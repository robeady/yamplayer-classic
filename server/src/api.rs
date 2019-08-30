use crate::errors::{string_err, Erro, Try};
use crate::file_completions::complete_file_path;
use crate::library::{self, Library, TrackId};
use crate::player::PlayerApp;
use parking_lot::{Mutex, RwLock};
use serde_derive::{Deserialize, Serialize};
use slotmap::{DenseSlotMap, Key};
use std::convert::Into;
use std::sync::Arc;

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
    TogglePause,
    SkipToNext,
    ChangeVolume {
        volume: Option<f32>,
        muted: Option<bool>,
    },
    CompleteFilePath {
        prefix: String,
    },
    GetTrack {
        track_id: String,
    },
    GetLibrary,
    AddToLibrary {
        path: String,
    },
    GetPlaybackState,
}

impl App {
    pub fn handle_request(&self, request: &Request) -> Response {
        use Request::*;
        match request {
            Enqueue { track_id } => self.enqueue(track_id),
            Stop => self.player.empty_queue().and_done(),
            TogglePause => self.player.toggle_pause().and_done(),
            SkipToNext => self.player.skip_to_next().and_done(),
            ChangeVolume { volume, muted } => self.player.update_volume(*volume, *muted).and_done(),
            CompleteFilePath { prefix } => self.completions(prefix),
            GetTrack { track_id } => self.get_track(track_id),
            GetLibrary => self.list_library(),
            AddToLibrary { path } => self.add_to_library(path.clone()),
            GetPlaybackState => self.get_playback_state(),
        }
    }

    fn enqueue(&self, track_id: &str) -> Response {
        let track_id = parse_track_id(track_id)?;
        let lib = self.library.lock();
        let track = lib
            .get_track(track_id)
            .ok_or_else(|| string_err(format!("Unknown track {}", track_id.0)))?;
        log::info!("enqueueing track {} from {}", track_id.0, track.file_path);
        self.player.add_to_queue(track_id, &track.file_path)?;
        // TODO: this event should come from somewhere else
        //        self.event_sink.broadcast(&Event::TrackChanged {
        //            track: (track_id, track).into(),
        //        });
        done()
    }

    fn get_track(&self, track_id: &str) -> Response {
        let track_id = parse_track_id(track_id)?;
        let lib = self.library.lock();
        let track = lib.get_track(track_id);
        ok(&track)
    }

    fn list_library(&self) -> Response {
        let lib = self.library.lock();
        let tracks = lib.list_tracks().map(Into::into).collect();
        ok(&LibraryListing { tracks })
    }

    fn add_to_library(&self, track_file_path: String) -> Response {
        self.library.lock().add_track(track_file_path)?;
        done()
    }

    fn get_playback_state(&self) -> Response {
        ok(&PlaybackState {
            playing: !self.player.paused(),
            volume: self.player.unmuted_volume(),
            muted: self.player.muted(),
        })
    }

    fn completions(&self, prefix: &str) -> Response {
        ok(&CompleteFilePathResp {
            completions: complete_file_path(prefix)?,
        })
    }
}

fn parse_track_id(track_id: &str) -> Try<TrackId> {
    Ok(TrackId(track_id.parse().map_err(|_| {
        string_err(format!("Invalid track ID {}", track_id))
    })?))
}

#[derive(Debug, Serialize)]
struct PlaybackState {
    playing: bool,
    volume: f32,
    muted: bool,
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

impl From<Erro> for Payload {
    fn from(e: Erro) -> Self {
        match e {
            Erro::StringError(s) => payload(&s),
        }
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
    VolumeChanged { muted: bool, volume: f32 },
    PlaybackPaused,
    PlaybackResumed,
    TrackChanged { track_id: Option<TrackId> },
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
