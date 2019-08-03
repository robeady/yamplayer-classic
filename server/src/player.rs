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
    unmuted_volume: f32,
    muted: bool,
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
            unmuted_volume: 0.5,
            muted: false,
        })
    }

    pub fn unmuted_volume(&self) -> f32 {
        self.unmuted_volume
    }

    pub fn muted(&self) -> bool {
        self.muted
    }

    pub fn update_volume(&mut self, volume: Option<f32>, muted: Option<bool>) {
        if let Some(new_volume) = volume {
            self.unmuted_volume = new_volume;
        }
        if let Some(new_muted) = muted {
            self.muted = new_muted;
        }
        self.update_sink_volume();
        self.event_sink.broadcast(&VolumeChanged {
            muted: self.muted,
            volume: self.unmuted_volume,
        })
    }

    fn update_sink_volume(&mut self) {
        if self.muted {
            self.sink.set_volume(0.0)
        } else {
            self.sink.set_volume(self.unmuted_volume)
        }
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

    pub fn paused(&self) -> bool {
        self.sink.is_paused()
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
