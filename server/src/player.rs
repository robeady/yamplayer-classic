use crate::api::Event::{PlaybackPaused, PlaybackResumed, VolumeChanged};
use crate::api::EventSink;
use crate::errors::{string_err, Try};
use log;
use rodio::decoder::Decoder;
use rodio::{Device, Sink, Source};
use std::fs::File;
use std::io::{Cursor, Read};
use std::sync::Arc;

pub struct PlayerApp {
    device: Arc<Device>,
    sink: Sink,
    event_sink: Arc<EventSink>,
}

impl PlayerApp {
    pub fn new(event_sink: Arc<EventSink>) -> Try<PlayerApp> {
        let device =
            Arc::new(rodio::default_output_device().ok_or(string_err("no output device"))?);
        let sink = Sink::new(&device);
        Ok(PlayerApp {
            device,
            sink,
            event_sink,
        })
    }

    pub fn volume(&self) -> f32 {
        self.sink.volume()
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.sink.set_volume(volume);
        self.event_sink
            .broadcast(&VolumeChanged { new_volume: volume })
    }

    pub fn toggle_pause(&mut self) {
        if self.sink.is_paused() {
            self.sink.play();
            self.event_sink.broadcast(&PlaybackResumed)
        } else {
            self.sink.pause();
            self.event_sink.broadcast(&PlaybackPaused)
        }
    }

    pub fn add_to_queue(&mut self, track_file_path: &str) -> Try<()> {
        log::debug!("loading file");
        let buffer = load_file(track_file_path)?;
        log::debug!("file loaded into memory");
        let source: Decoder<_> = Decoder::new(Cursor::new(buffer))?;
        match source.total_duration().map(|d| d.as_secs()) {
            None => log::warn!("playing track with unknown length"),
            Some(duration_secs) => log::info!(
                "playing track with length: {}:{:02}",
                duration_secs / 60,
                duration_secs % 60
            ),
        }
        // for now we just replace what's currently playing
        self.empty_queue();
        self.sink.append(source);
        Ok(())
    }

    pub fn empty_queue(&mut self) {
        self.sink.stop();
        let new_sink = Sink::new(&self.device);
        new_sink.set_volume(self.sink.volume());
        self.sink = new_sink;
    }
}

fn load_file(path: &str) -> Try<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}
