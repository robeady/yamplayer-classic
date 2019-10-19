use self::search::SearchResults;
use crate::deezer::client::DeezerClient;
use crate::errors::Erro;
use crate::file_completions::complete_file_path;
use crate::library::{self, Library};
use crate::model::{PlaylistId, TrackId};
use crate::player::PlayerApp;
use crate::queue::CurrentTrack;
use anyhow::anyhow;
use fstrings::{f, format_args_f};
use parking_lot::{Mutex, RwLock};
use serde_derive::{Deserialize, Serialize};
use slotmap::{DenseSlotMap, Key};
use std::collections::BTreeMap;
use std::convert::Into;
use std::sync::Arc;

pub mod search;

pub struct App {
    pub player: PlayerApp,
    pub library: Mutex<Library>,
    pub event_sink: Arc<EventSink>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "args")]
pub enum Request {
    Enqueue {
        track_id: String,
    },
    Stop,
    Pause,
    Unpause,
    SkipToNext,
    ChangeVolume {
        volume: Option<f32>,
        muted: Option<bool>,
    },
    CompleteFilePath {
        prefix: String,
    },
    GetTracks {
        track_ids: Vec<String>,
    },
    GetLibrary,
    AddToLibrary {
        path: String,
    },
    GetPlaybackState,
    ListPlaylists,
    GetPlaylist {
        id: String,
    },
    Search {
        query: String,
    },
}

impl App {
    pub fn handle_request(&self, request: &Request) -> Response {
        use Request::*;
        #[allow(clippy::unit_arg)]
        match request {
            Enqueue { track_id } => self.enqueue(track_id),
            Stop => self.player.empty_queue().and_done(),
            Pause => self.player.pause().and_done(),
            Unpause => self.player.unpause().and_done(),
            SkipToNext => self.player.skip_to_next().and_done(),
            ChangeVolume { volume, muted } => self.player.update_volume(*volume, *muted).and_done(),
            CompleteFilePath { prefix } => self.completions(prefix),
            GetTracks { track_ids } => self.get_tracks(track_ids),
            GetLibrary => self.list_library(),
            AddToLibrary { path } => self.add_to_library(path.clone()),
            GetPlaybackState => self.get_playback_state(),
            ListPlaylists => self.list_playlists(),
            GetPlaylist { ref id } => ok(&self.library.lock().get_playlist(id.parse()?)),
            Search { ref query } => self.search(query),
        }
    }

    fn enqueue(&self, track_id: &str) -> Response {
        let track_id = track_id.parse()?;
        let lib = self.library.lock();
        let track = lib
            .get_track(track_id)
            .ok_or_else(|| anyhow!("Unknown track {}", track_id.0))?;
        log::info!("enqueueing track {} from {}", track_id.0, track.file_path);
        self.player.add_to_queue(track_id, &track)?;
        done()
    }

    fn get_tracks(&self, track_ids: &[String]) -> Response {
        let lib = self.library.lock();
        let tracks: Result<BTreeMap<TrackId, Option<Track>>, Erro> = track_ids
            .iter()
            .map(|id| {
                let id = id.parse()?;
                Ok((id, lib.get_track(id).map(|t| (id, t).into())))
            })
            .collect();
        ok(&tracks?)
    }

    fn list_library(&self) -> Response {
        let lib = self.library.lock();
        let tracks = lib.tracks().map(Into::into).collect();
        ok(&LibraryListing { tracks })
    }

    fn add_to_library(&self, track_file_path: String) -> Response {
        self.library.lock().add_track(track_file_path)?;
        done()
    }

    fn get_playback_state(&self) -> Response {
        #[derive(Debug, Serialize)]
        struct PlaybackState {
            playing: bool,
            volume: f32,
            muted: bool,
        }
        ok(&PlaybackState {
            playing: !self.player.paused(),
            volume: self.player.unmuted_volume(),
            muted: self.player.muted(),
        })
    }

    fn list_playlists(&self) -> Response {
        #[derive(Serialize)]
        struct Playlists<'a> {
            playlists: Vec<PlaylistInfo<'a>>,
        }
        #[derive(Serialize)]
        struct PlaylistInfo<'a> {
            id: PlaylistId,
            name: &'a str,
        }
        ok(&Playlists {
            playlists: self
                .library
                .lock()
                .playlists()
                .map(|(id, p)| PlaylistInfo { id, name: &p.name })
                .collect(),
        })
    }

    fn completions(&self, prefix: &str) -> Response {
        ok(&CompleteFilePathResp {
            completions: complete_file_path(prefix)?,
        })
    }

    fn search(&self, query: &str) -> Response {
        let results = DeezerClient::new().search(query)?;
        ok(&self.library.lock().resolve(results))
    }
}

impl<'a> From<(TrackId, &'a library::Track)> for Track<'a> {
    fn from(t: (TrackId, &'a library::Track)) -> Self {
        let (id, track) = t;
        Track {
            id: id.0.to_string(),
            file_path: &track.file_path,
            title: &track.title,
            artist: &track.artist,
            album: &track.album,
            duration_secs: track.duration_secs,
        }
    }
}

#[derive(Debug, Serialize)]
struct LibraryListing<'a> {
    tracks: Vec<Track<'a>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Track<'a> {
    pub id: String,
    pub file_path: &'a str,
    pub title: &'a str,
    pub artist: &'a str,
    pub album: &'a str,
    pub duration_secs: f32,
}

#[derive(Debug, Clone)]
pub struct Payload {
    pub json: String,
}

fn payload(data: &impl serde::Serialize) -> Payload {
    Payload {
        json: serde_json::to_string(data).expect("payload serialization failed"),
    }
}

//#[derive(Serialize)]
//struct ErrorPayload {
//    message: String,
//    trace: Backt,
//}

impl From<Erro> for Payload {
    fn from(e: Erro) -> Self {
        // debug print the error
        payload(&f!("{e:?}"))
    }
}

pub type Response = Result<Payload, Payload>;

fn ok(data: &impl serde::Serialize) -> Response {
    Ok(payload(data))
}

fn done() -> Response {
    ok(&())
}

trait AndDoneExt {
    fn and_done(&self) -> Response {
        done()
    }
}
impl<T> AndDoneExt for T {}

#[derive(Debug, Serialize, Deserialize)]
struct CompleteFilePathResp {
    completions: Vec<String>,
}

#[derive(Serialize)]
#[serde(tag = "type", content = "args")]
pub enum Event {
    VolumeChanged {
        muted: bool,
        volume: f32,
    },
    PlaybackChanged {
        paused: bool,
        current_track: Option<CurrentTrack>,
    },
}

pub trait EventDestination: Send + Sync {
    fn send_event(&self, payload: &Payload);
}

pub trait ResponseDestination {
    fn send_response(&self, request_id: String, response: Response);
}

struct EventCollector {
    payloads: Mutex<Vec<Payload>>,
}

impl EventDestination for EventCollector {
    fn send_event(&self, payload: &Payload) {
        self.payloads.lock().push(payload.clone())
    }
}

impl<F: Fn(&Payload) -> () + Send + Sync> EventDestination for F {
    fn send_event(&self, payload: &Payload) {
        self(payload)
    }
}

pub struct EventSink {
    destinations: RwLock<DenseSlotMap<Box<dyn EventDestination>>>,
}

impl EventSink {
    pub fn empty() -> EventSink {
        EventSink {
            destinations: RwLock::new(DenseSlotMap::new()),
        }
    }

    pub fn broadcast(&self, event: &Event) {
        let payload = payload(event);
        for (_, dest) in self.destinations.read().iter() {
            dest.send_event(&payload)
        }
    }

    pub fn add_destination(&self, destination: Box<dyn EventDestination>) -> DestinationKey {
        DestinationKey(self.destinations.write().insert(destination))
    }

    pub fn remove_destination(&self, key: DestinationKey) -> Option<Box<dyn EventDestination>> {
        self.destinations.write().remove(key.0)
    }
}

pub struct DestinationKey(Key);
