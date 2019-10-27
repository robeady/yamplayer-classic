use crate::api::Event::{PlaybackChanged, VolumeChanged};
use crate::api::{Event, EventSink};
use crate::errors::Try;
use crate::model::{LoadedTrack, TrackId};
use crate::playback;
use crate::queue::{Queue, QueueCallback};
use log;
use parking_lot::Mutex;
use rodio::decoder::Decoder;
use std::io::Cursor;
use std::sync::Arc;

pub struct PlayerApp {
    queue: Arc<Mutex<Queue<f32, QueueCallbackHandler>>>,
    event_sink: Arc<EventSink>,
}

struct QueueCallbackHandler {
    event_sink: Arc<EventSink>,
}

impl QueueCallback<f32> for QueueCallbackHandler {
    fn on_current_track_changed(&self, queue: &Queue<f32, Self>) {
        self.event_sink.broadcast(&Event::PlaybackChanged {
            paused: queue.controls.paused,
            current_track: queue.current_track(),
        });
    }
}

impl PlayerApp {
    pub fn new(event_sink: Arc<EventSink>) -> Try<PlayerApp> {
        let callback = QueueCallbackHandler {
            event_sink: Arc::clone(&event_sink),
        };
        let queue = playback::establish(callback);
        Ok(PlayerApp { queue, event_sink })
    }

    pub fn unmuted_volume(&self) -> f32 {
        self.queue.lock().controls.volume
    }

    pub fn muted(&self) -> bool {
        self.queue.lock().controls.muted
    }

    pub fn update_volume(&self, volume: Option<f32>, muted: Option<bool>) {
        let mut source = self.queue.lock();
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

    pub fn unpause(&self) {
        let mut queue = self.queue.lock();
        if queue.controls.paused {
            queue.controls.paused = false;
            self.event_sink.broadcast(&PlaybackChanged {
                paused: false,
                current_track: queue.current_track(),
            })
        }
    }

    pub fn pause(&self) {
        let mut source = self.queue.lock();
        if !source.controls.paused {
            source.controls.paused = true;
            self.event_sink.broadcast(&PlaybackChanged {
                paused: true,
                current_track: source.current_track(),
            })
        }
    }

    pub fn paused(&self) -> bool {
        self.queue.lock().controls.paused
    }

    pub fn skip_to_next(&self) {
        self.queue.lock().skip_current();
    }

    pub fn add_to_queue(&self, track_id: TrackId, track: LoadedTrack) -> Try<()> {
        let source: Decoder<_> = Decoder::new(Cursor::new(track.data))?;
        log::info!(
            "enqueuing track {} with length: {}:{:02}",
            track_id,
            track.duration_secs as i64 / 60,
            track.duration_secs as i64 % 60
        );
        self.queue
            .lock()
            .enqueue_last(track_id, track.duration_secs, Box::new(source));
        Ok(())
    }

    pub fn empty_queue(&self) {
        self.queue.lock().clear();
    }
}
