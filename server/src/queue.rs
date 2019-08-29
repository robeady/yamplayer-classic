use crate::library::TrackId;
use cpal::Format;
use rodio::source::UniformSourceIterator;
use rodio::{Sample, Source};
use std::collections::VecDeque;

pub struct Queue<S> {
    tracks: VecDeque<EnqueuedTrack<S>>,
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

    pub fn push_back<T: Sample + Send + 'static>(&mut self, track: Track<T>) -> EntryMarker {
        let track = self.create_track(track);
        let entry_marker = track.entry_marker;
        self.tracks.push_back(track);
        entry_marker
    }

    pub fn push_front<T: Sample + Send + 'static>(&mut self, track: Track<T>) -> EntryMarker {
        let track = self.create_track(track);
        let entry_marker = track.entry_marker;
        self.tracks.push_front(track);
        entry_marker
    }

    fn create_track<T: Sample + Send + 'static>(&mut self, track: Track<T>) -> EnqueuedTrack<S> {
        let entry_marker = EntryMarker(self.next_entry_marker);
        self.next_entry_marker += 1;
        // converter that interpolates the input track samples producing the right output format
        let mixed_source = UniformSourceIterator::new(
            track.source,
            self.audio_format.channels,
            self.audio_format.sample_rate.0,
        );
        EnqueuedTrack {
            audio_source: Box::new(mixed_source),
            id: track.id,
            entry_marker,
        }
    }

    // returns true iff the item was in the queue
    pub fn remove(&mut self, marker: EntryMarker) -> bool {
        let length_before = self.tracks.len();
        self.tracks.retain(|t| t.entry_marker != marker);
        match length_before - self.tracks.len() {
            0 => false,
            1 => true,
            _ => panic!("more than one track with entry marker {:?}", marker),
        }
    }

    pub fn clear(&mut self) {
        self.tracks.clear();
    }

    fn tracks(&mut self) -> impl Iterator<Item = (EntryMarker, TrackId)> + '_ {
        self.tracks.iter().map(|t| (t.entry_marker, t.id))
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

pub struct Track<S> {
    pub id: TrackId,
    pub source: Box<dyn Source<Item = S> + Send>,
}

struct EnqueuedTrack<S> {
    id: TrackId,
    entry_marker: EntryMarker,
    audio_source: Box<dyn Source<Item = S> + Send>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct EntryMarker(u64);
