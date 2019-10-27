use crate::api::search::SearchResults;
use crate::api::EventSink;
use crate::errors::Try;
use crate::model::{LibraryTrackId, PlaylistId, TrackId};
use anyhow::anyhow;
use fstrings::{f, format_args_f};
use id3::Tag;
use log;
use serde_derive::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;

pub struct Library {
    tracks: BTreeMap<LibraryTrackId, Track>,
    playlists: BTreeMap<PlaylistId, Playlist>,
    event_sink: Arc<EventSink>,
}

impl Library {
    pub fn new(event_sink: Arc<EventSink>) -> Library {
        Library {
            tracks: BTreeMap::new(),
            playlists: BTreeMap::new(),
            event_sink,
        }
    }

    fn next_track_id(&self) -> LibraryTrackId {
        LibraryTrackId(
            self.tracks
                .iter()
                .next_back()
                .map(|(id, _)| id.0 + 1)
                .unwrap_or(0),
        )
    }

    fn next_playlist_id(&self) -> PlaylistId {
        PlaylistId(
            self.playlists
                .iter()
                .next_back()
                .map(|(id, _)| id.0 + 1)
                .unwrap_or(0),
        )
    }

    pub fn add_track(&mut self, file_path: String) -> Try<LibraryTrackId> {
        if file_path.ends_with(".mp3") {
            self.add_track_mp3(file_path)
        } else if file_path.ends_with(".flac") {
            self.add_track_flac(file_path)
        } else {
            Err(anyhow!(f!("unsupported file type {file_path}")))
        }
    }

    fn add_track_mp3(&mut self, file_path: String) -> Try<LibraryTrackId> {
        let mp3_tags = Tag::read_from_path(&file_path)?;
        let tag = |name: &str, value: Option<&str>| {
            value.map(|t| t.to_string()).unwrap_or_else(|| {
                log::warn!("no {} tag in {}", name, file_path);
                f!("UNKNOWN {name}")
            })
        };
        let mut mp3 = minimp3::Decoder::new(BufReader::new(File::open(&file_path)?));
        let mut duration_secs = 0_f32;
        loop {
            let frame_result = mp3.next_frame();
            let frame = match frame_result {
                Ok(frame) => frame,
                // An error caused by some IO operation required during decoding.
                Err(minimp3::Error::Io(e)) => return Err(e.into()),
                // The decoder tried to parse a frame from its internal buffer, but there was not enough.
                Err(minimp3::Error::InsufficientData) => {
                    panic!("not enough data in encoder buffer??? {}", file_path)
                }
                // The decoder encountered data which was not a frame (ie, ID3 data), and skipped it.
                Err(minimp3::Error::SkippedData) => continue,
                // The decoder has reached the end of the provided reader.
                Err(minimp3::Error::Eof) => break,
            };
            let seconds_of_audio =
                (frame.data.len() / frame.channels) as f32 / frame.sample_rate as f32;
            duration_secs += seconds_of_audio;
        }
        let track = Track {
            title: tag("TITLE", mp3_tags.title()),
            artist: tag("ARTIST", mp3_tags.artist()),
            album: tag("ALBUM", mp3_tags.album()),
            duration_secs,
            file_path,
            external_id: None,
        };
        let track_id = self.next_track_id();
        self.tracks.insert(track_id, track);
        Ok(track_id)
    }

    fn add_track_flac(&mut self, file_path: String) -> Try<LibraryTrackId> {
        let flac = claxon::FlacReader::open(&file_path)?;
        let tag = |name: &str| {
            flac.get_tag(name)
                .next()
                .map(|t| t.to_string())
                .unwrap_or_else(|| {
                    log::warn!("no {} tag in {}", name, file_path);
                    f!("UNKNOWN {name}")
                })
        };
        let track = Track {
            title: tag("TITLE"),
            artist: tag("ARTIST"),
            album: tag("ALBUM"),
            duration_secs: (flac
                .streaminfo()
                .samples
                .unwrap_or_else(|| panic!("no stream info in {}", file_path))
                as f32)
                / flac.streaminfo().sample_rate as f32,
            file_path,
            external_id: None,
        };
        let track_id = self.next_track_id();
        self.tracks.insert(track_id, track);
        Ok(track_id)
    }

    pub fn tracks(&self) -> impl Iterator<Item = (LibraryTrackId, &Track)> {
        self.tracks.iter().map(|(id, t)| (*id, t))
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn get_track(&self, id: &LibraryTrackId) -> Option<&Track> {
        self.tracks.get(id)
    }

    pub fn create_playlist(&mut self, name: String) -> PlaylistId {
        let playlist_id = self.next_playlist_id();
        self.playlists.insert(
            playlist_id,
            Playlist {
                name,
                track_ids: Vec::new(),
            },
        );
        playlist_id
    }

    pub fn add_track_to_playlist(
        &mut self,
        track_id: LibraryTrackId,
        playlist_id: PlaylistId,
    ) -> Try<()> {
        self.tracks
            .get(&track_id)
            .ok_or_else(|| anyhow!("Unknown track {}", track_id.0))?;
        self.playlists
            .get_mut(&playlist_id)
            .ok_or_else(|| anyhow!("Unknown playlist {}", playlist_id.0))?
            .track_ids
            .push(track_id);
        Ok(())
    }

    pub fn get_playlist(&self, id: PlaylistId) -> Option<&Playlist> {
        self.playlists.get(&id)
    }

    pub fn playlists(&self) -> impl Iterator<Item = (PlaylistId, &Playlist)> {
        self.playlists.iter().map(|(id, p)| (*id, p))
    }

    pub fn resolve(&self, mut search_results: SearchResults) -> SearchResults {
        for track_result in &mut search_results.tracks {
            if let Some((id, _)) = self
                .tracks
                .iter()
                .find(|(_, t)| t.external_id.as_ref() == Some(&track_result.track.external_id))
            {
                track_result.track.library_id = Some(TrackId::Library(*id))
            }
        }
        // TODO: tracks and artists
        search_results
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Track {
    pub file_path: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration_secs: f32,
    pub external_id: Option<String>,
}

#[derive(Serialize)]
pub struct Playlist {
    pub name: String,
    track_ids: Vec<LibraryTrackId>,
}

impl Playlist {
    fn tracks(&self) -> impl Iterator<Item = LibraryTrackId> + '_ {
        self.track_ids.iter().copied()
    }
}
