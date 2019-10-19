use crate::model::TrackId;
use crate::serde::number_string;
use cpal::Format;
use rodio::source::UniformSourceIterator;
use rodio::{Sample, Source};
use serde_derive::Serialize;
use std::collections::VecDeque;

pub struct Queue<S, C> {
    tracks: VecDeque<QueueItem<S>>,
    next_entry_marker: u64,
    audio_format: Format,
    callback: C,
    pub controls: PlaybackControls,
}

pub struct PlaybackControls {
    pub paused: bool,
    pub muted: bool,
    pub volume: f32,
}

pub trait QueueCallback<S>
where
    Self: Sized,
{
    fn on_current_track_changed(&self, queue: &Queue<S, Self>);
}

impl<S, C> Queue<S, C>
where
    S: Sample + Send + 'static,
    C: QueueCallback<S>,
{
    pub fn new(initial_volume: f32, audio_format: Format, callback: C) -> Self {
        Queue {
            tracks: VecDeque::new(),
            next_entry_marker: 0,
            audio_format,
            callback,
            controls: PlaybackControls {
                paused: false,
                muted: false,
                volume: initial_volume,
            },
        }
    }

    pub fn enqueue_last<T: Sample + Send + 'static>(
        &mut self,
        id: TrackId,
        duration_secs: f32,
        source: Box<dyn Source<Item = T> + Send>,
    ) -> EntryMarker {
        let t = self.create_track(id, duration_secs, source);
        let entry_marker = t.track.entry_marker;
        self.tracks.push_back(t);
        if self.tracks.len() == 1 {
            self.raise_track_changed();
        }
        entry_marker
    }

    pub fn enqueue_next<T: Sample + Send + 'static>(
        &mut self,
        id: TrackId,
        duration_secs: f32,
        source: Box<dyn Source<Item = T> + Send>,
    ) -> EntryMarker {
        let t = self.create_track(id, duration_secs, source);
        let entry_marker = t.track.entry_marker;
        if self.tracks.is_empty() {
            self.tracks.push_front(t);
            self.raise_track_changed();
        } else {
            self.tracks.insert(1, t);
        }
        entry_marker
    }

    fn create_track<T: Sample + Send + 'static>(
        &mut self,
        id: TrackId,
        duration_secs: f32,
        source: Box<dyn Source<Item = T> + Send>,
    ) -> QueueItem<S> {
        let entry_marker = EntryMarker(self.next_entry_marker);
        self.next_entry_marker += 1;
        // converter that interpolates the input track samples producing the right output format
        let mixed_source = UniformSourceIterator::new(
            source,
            self.audio_format.channels,
            self.audio_format.sample_rate.0,
        );
        QueueItem {
            track: EnqueuedTrack {
                id,
                duration_secs,
                entry_marker,
            },
            audio_source: CountedSource::new(Box::new(mixed_source)),
        }
    }

    pub fn skip_current(&mut self) -> Option<EnqueuedTrack> {
        let popped = self.tracks.pop_front().map(|t| t.track);
        self.raise_track_changed();
        popped
    }

    fn raise_track_changed(&self) {
        self.callback.on_current_track_changed(self);
    }

    // returns true iff the item was in the queue
    pub fn remove(&mut self, marker: EntryMarker) -> bool {
        let length_before = self.tracks.len();
        self.tracks.retain(|t| t.track.entry_marker != marker);
        match length_before - self.tracks.len() {
            0 => false,
            1 => true,
            _ => panic!("more than one track with entry marker {:?}", marker),
        }
    }

    pub fn clear(&mut self) {
        self.tracks.clear();
    }

    fn tracks(&mut self) -> impl Iterator<Item = &EnqueuedTrack> + '_ {
        self.tracks.iter().map(|t| &t.track)
    }

    pub fn current_track(&self) -> Option<CurrentTrack> {
        self.tracks.get(0).map(|t| CurrentTrack {
            track: t.track,
            position_secs: t.audio_source.samples_played as f32
                / self.audio_format.channels as f32
                / self.audio_format.sample_rate.0 as f32,
        })
    }

    fn next_sample(&mut self) -> Option<S> {
        if let Some(track) = self.tracks.get_mut(0) {
            if let Some(sample) = track.audio_source.next() {
                Some(sample)
            } else {
                // current source is over, advance to next
                self.tracks.pop_front();
                self.raise_track_changed();
                // recurse now that current_source is updated
                self.next_sample()
            }
        } else {
            // no current source, play silence
            Some(Sample::zero_value())
        }
    }
}

#[derive(Serialize)]
pub struct CurrentTrack {
    track: EnqueuedTrack,
    position_secs: f32,
}

impl<S, C> Iterator for Queue<S, C>
where
    S: Sample + Send + 'static,
    C: QueueCallback<S>,
{
    type Item = S;

    fn next(&mut self) -> Option<S> {
        if self.controls.paused {
            Some(Sample::zero_value())
        } else if self.controls.muted {
            self.next_sample();
            Some(Sample::zero_value())
        } else {
            self.next_sample()
                .map(|sample| sample.amplify(self.controls.volume))
        }
    }
}

struct QueueItem<S> {
    track: EnqueuedTrack,
    audio_source: CountedSource<S>,
}

#[derive(Copy, Clone, Serialize)]
pub struct EnqueuedTrack {
    pub id: TrackId,
    pub duration_secs: f32,
    pub entry_marker: EntryMarker,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct EntryMarker(#[serde(with = "number_string")] u64);

struct CountedSource<S> {
    samples_played: u64,
    inner: Box<dyn Source<Item = S> + Send>,
}

impl<S> CountedSource<S> {
    fn new(source: Box<dyn Source<Item = S> + Send>) -> Self {
        CountedSource {
            samples_played: 0,
            inner: source,
        }
    }
}

impl<S> Iterator for CountedSource<S> {
    type Item = S;

    fn next(&mut self) -> Option<S> {
        let sample = self.inner.next();
        if sample.is_some() {
            self.samples_played += 1;
        }
        sample
    }
}
