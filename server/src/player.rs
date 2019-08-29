use crate::api::Event::{PlaybackPaused, PlaybackResumed, VolumeChanged};
use crate::api::{Event, EventSink};
use crate::errors::Try;
use crate::library::TrackId;
use crate::playback;
use crate::playback::PlaybackSource;
use crate::queue::{QueueEvent, QueueEventSink};
use log;
use parking_lot::Mutex;
use rodio::decoder::Decoder;
use rodio::Source;
use std::fs::File;
use std::io::{Cursor, Read};
use std::sync::Arc;

pub struct PlayerApp {
    source: Arc<Mutex<PlaybackSource<EventAdapter>>>,
    event_sink: Arc<EventSink>,
}

struct EventAdapter {
    event_sink: Arc<EventSink>,
}

impl QueueEventSink for EventAdapter {
    fn accept(&self, event: QueueEvent) {
        self.event_sink.broadcast(&match event {
            QueueEvent::TrackChanged(t) => Event::TrackChanged {
                track_id: t.map(|t| t.id),
            },
        })
    }
}

impl PlayerApp {
    pub fn new(event_sink: Arc<EventSink>) -> Try<PlayerApp> {
        let adapter = EventAdapter {
            event_sink: Arc::clone(&event_sink),
        };
        let source = playback::establish(adapter);
        Ok(PlayerApp { source, event_sink })
    }

    pub fn unmuted_volume(&self) -> f32 {
        self.source.lock().controls.volume
    }

    pub fn muted(&self) -> bool {
        self.source.lock().controls.muted
    }

    pub fn update_volume(&self, volume: Option<f32>, muted: Option<bool>) {
        let mut source = self.source.lock();
        if let Some(new_volume) = volume {
            source.controls.volume = new_volume;
        }
        if let Some(new_muted) = muted {
            source.controls.muted = new_muted;
        }
        self.event_sink.broadcast(&VolumeChanged {
            muted: source.controls.muted,
            volume: source.controls.volume,
        })
    }

    pub fn toggle_pause(&self) {
        let mut source = self.source.lock();
        if source.controls.paused {
            source.controls.paused = false;
            self.event_sink.broadcast(&PlaybackResumed)
        } else {
            source.controls.paused = true;
            self.event_sink.broadcast(&PlaybackPaused)
        }
    }

    pub fn paused(&self) -> bool {
        self.source.lock().controls.paused
    }

    pub fn skip_to_next(&self) {
        self.source.lock().queue.pop_front();
    }

    pub fn add_to_queue(&self, track_id: TrackId, track_file_path: &str) -> Try<()> {
        log::debug!("loading file");
        let buffer = load_file(track_file_path)?;
        log::debug!("file loaded into memory");
        let source: Decoder<_> = Decoder::new(Cursor::new(buffer))?;
        match source.total_duration().map(|d| d.as_secs()) {
            None => log::warn!("enqueuing track with unknown length"),
            Some(duration_secs) => log::info!(
                "enqueuing track with length: {}:{:02}",
                duration_secs / 60,
                duration_secs % 60
            ),
        }
        self.source
            .lock()
            .queue
            .push_back(track_id, Box::new(source));
        Ok(())
    }

    pub fn empty_queue(&self) {
        self.source.lock().queue.clear();
    }
}

fn load_file(path: &str) -> Try<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}
