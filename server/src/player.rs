use crate::api::Event::{PlaybackPaused, PlaybackResumed, VolumeChanged};
use crate::api::{Event, EventSink};
use crate::errors::Try;
use crate::library::{Track, TrackId};
use crate::playback;
use crate::playback::PlaybackSource;
use crate::queue::{QueueEvent, QueueEventSink};
use log;
use parking_lot::Mutex;
use rodio::decoder::Decoder;
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
            QueueEvent::TrackChanged(t) => Event::PlayingTrackChanged(t),
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
        // TODO: is zero a sensible default?
        let position_secs = source
            .queue
            .current_track_played_duration_secs()
            .unwrap_or(0.0);
        if source.controls.paused {
            source.controls.paused = false;
            self.event_sink
                .broadcast(&PlaybackResumed { position_secs })
        } else {
            source.controls.paused = true;
            self.event_sink.broadcast(&PlaybackPaused { position_secs })
        }
    }

    pub fn paused(&self) -> bool {
        self.source.lock().controls.paused
    }

    pub fn skip_to_next(&self) {
        self.source.lock().queue.skip_current();
    }

    pub fn add_to_queue(&self, track_id: TrackId, track: &Track) -> Try<()> {
        log::debug!("loading file");
        let buffer = load_file(&track.file_path)?;
        log::debug!("file loaded into memory");
        let source: Decoder<_> = Decoder::new(Cursor::new(buffer))?;
        log::info!(
            "enqueuing track with length: {}:{:02}",
            track.duration_secs as i64 / 60,
            track.duration_secs as i64 % 60
        );
        self.source
            .lock()
            .queue
            .enqueue_last(track_id, track.duration_secs, Box::new(source));
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
