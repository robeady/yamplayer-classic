use crate::library::TrackId;
use cpal::Format;
use rodio::source::UniformSourceIterator;
use rodio::{Sample, Source};
use std::collections::VecDeque;

pub struct Queue<S> {
    tracks: VecDeque<QueueItem<S>>,
    next_entry_marker: u64,
    audio_format: Format,
}

impl<S: Sample + Send + 'static> Queue<S> {
    pub fn new(audio_format: Format) -> Self {
        Queue {
            tracks: VecDeque::new(),
            next_entry_marker: 0,
            audio_format,
        }
    }

    pub fn push_back<T: Sample + Send + 'static>(
        &mut self,
        id: TrackId,
        source: Box<dyn Source<Item = T> + Send>,
    ) -> EntryMarker {
        let t = self.create_track(id, source);
        let entry_marker = t.track.entry_marker;
        self.tracks.push_back(t);
        entry_marker
    }

    pub fn push_front<T: Sample + Send + 'static>(
        &mut self,
        id: TrackId,
        source: Box<dyn Source<Item = T> + Send>,
    ) -> EntryMarker {
        let t = self.create_track(id, source);
        let entry_marker = t.track.entry_marker;
        self.tracks.push_front(t);
        entry_marker
    }

    fn create_track<T: Sample + Send + 'static>(
        &mut self,
        id: TrackId,
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
            track: EnqueuedTrack { id, entry_marker },
            audio_source: Box::new(mixed_source),
        }
    }

    pub fn pop_front(&mut self) -> Option<EnqueuedTrack> {
        self.tracks.pop_front().map(|t| t.track)
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
}

impl<S: Sample> Iterator for Queue<S> {
    type Item = S;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(track) = self.tracks.get_mut(0) {
            if let Some(sample) = track.audio_source.next() {
                Some(sample)
            } else {
                // current source is over, advance to next
                self.tracks.pop_front();
                // recurse now that current_source is updated
                self.next()
            }
        } else {
            // no current source, play silence
            Some(Sample::zero_value())
        }
    }
}

struct QueueItem<S> {
    track: EnqueuedTrack,
    audio_source: Box<dyn Source<Item = S> + Send>,
}

pub struct EnqueuedTrack {
    pub id: TrackId,
    pub entry_marker: EntryMarker,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct EntryMarker(u64);
