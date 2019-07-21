use crate::api::EventSink;
use crate::errors::{string_err, Try};
use serde_derive::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;

pub struct Library {
    tracks: BTreeMap<TrackId, Track>,
    event_sink: Arc<EventSink>,
}

impl Library {
    pub fn new(event_sink: Arc<EventSink>) -> Library {
        Library {
            tracks: BTreeMap::new(),
            event_sink,
        }
    }

    fn next_track_id(&self) -> TrackId {
        TrackId(
            self.tracks
                .iter()
                .next_back()
                .map(|(id, _)| id.0 + 1)
                .unwrap_or(0),
        )
    }

    pub fn add_track(&mut self, file_path: String) -> Try<TrackId> {
        let flac = claxon::FlacReader::open(&file_path)?;
        let tag = |name: &str| {
            flac.get_tag(name)
                .next()
                .ok_or(string_err(format!("no {} tag in {}", name, file_path)))
                .map(|t| t.to_string())
        };
        let title = tag("TITLE")?;
        let artist = tag("ARTIST")?;
        let album = tag("ALBUM")?;
        let track_id = self.next_track_id();
        let track = Track {
            title,
            artist,
            album,
            file_path,
        };

        self.tracks.insert(track_id, track);

        Ok(track_id)
    }

    pub fn list_tracks(&self) -> impl Iterator<Item = (TrackId, &Track)> {
        self.tracks.iter().map(|(id, t)| (*id, t))
    }

    pub fn get_track(&self, id: TrackId) -> Option<&Track> {
        self.tracks.get(&id)
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct TrackId(pub u64);

#[derive(Debug, Serialize, Deserialize)]
pub struct Track {
    pub file_path: String,
    pub title: String,
    pub artist: String,
    pub album: String,
}
