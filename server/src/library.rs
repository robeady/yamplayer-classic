use crate::Try;
use serde_derive::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub struct Library {
    tracks: BTreeMap<TrackId, Track>,
}

impl Library {
    pub fn new() -> Library {
        Library {
            tracks: BTreeMap::new(),
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
                .ok_or(format!("no {} tag in {}", name, file_path))
                .map(|t| t.to_string())
        };
        let title = tag("TITLE")?;
        let artist = tag("ARTIST")?;
        let album = tag("ALBUM")?;
        let track_id = self.next_track_id();
        self.tracks.insert(
            track_id,
            Track {
                title,
                artist,
                album,
                file_path,
            },
        );
        Ok(track_id)
    }

    pub fn list_tracks(&self) -> impl Iterator<Item = (TrackId, &Track)> {
        self.tracks.iter().map(|(id, t)| (*id, t))
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct TrackId(u64);

#[derive(Debug, Serialize, Deserialize)]
pub struct Track {
    file_path: String,
    title: String,
    artist: String,
    album: String,
}
