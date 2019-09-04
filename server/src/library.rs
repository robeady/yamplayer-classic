use crate::api::EventSink;
use crate::errors::{string_err, Try};
use crate::serde::number_string;
use id3::Tag;
use log;
use serde_derive::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufReader;
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
        if file_path.ends_with(".mp3") {
            self.add_track_mp3(file_path)
        } else if file_path.ends_with(".flac") {
            self.add_track_flac(file_path)
        } else {
            Err(string_err(format!("unsupported file type {}", file_path)))
        }
    }

    fn add_track_mp3(&mut self, file_path: String) -> Try<TrackId> {
        let mp3_tags = Tag::read_from_path(&file_path)?;
        let tag = |name: &str, value: Option<&str>| {
            value.map(|t| t.to_string()).unwrap_or_else(|| {
                log::warn!("no {} tag in {}", name, file_path);
                format!("UNKNOWN {}", name)
            })
        };
        let mut mp3 = minimp3::Decoder::new(BufReader::new(File::open(&file_path)?));
        let mut duration_secs = 0_f32;
        loop {
            let frame_result = mp3.next_frame();
            let frame = match frame_result {
                Ok(frame) => frame,
                // An error caused by some IO operation required during decoding.
                Err(minimp3::Error::Io(e)) => Err(e)?,
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
        };
        let track_id = self.next_track_id();
        self.tracks.insert(track_id, track);
        Ok(track_id)
    }

    fn add_track_flac(&mut self, file_path: String) -> Try<TrackId> {
        let flac = claxon::FlacReader::open(&file_path)?;
        let tag = |name: &str| {
            flac.get_tag(name)
                .next()
                .map(|t| t.to_string())
                .unwrap_or_else(|| {
                    log::warn!("no {} tag in {}", name, file_path);
                    format!("UNKNOWN {}", name)
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
        };
        let track_id = self.next_track_id();
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

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
pub struct TrackId(#[serde(with = "number_string")] pub u64);

#[derive(Debug, Serialize, Deserialize)]
pub struct Track {
    pub file_path: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration_secs: f32,
}
