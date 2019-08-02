use crate::errors::{string_err, Erro};
use crate::file_completions::complete_file_path;
use crate::library::{self, Library, TrackId};
use crate::player::PlayerApp;
use parking_lot::{Mutex, RwLock};
use serde_derive::{Deserialize, Serialize};
use slotmap::{DenseSlotMap, Key};
use std::convert::Into;
use std::sync::Arc;

pub struct App {
    pub player: Mutex<PlayerApp>,
    pub library: Mutex<Library>,
    pub event_sink: Arc<EventSink>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "args")]
pub enum Request {
    Enqueue { track_id: String },
    Stop,
    TogglePause,
    ChangeVolume { volume: f32 },
    CompleteFilePath { prefix: String },
    GetLibrary,
    AddToLibrary { path: String },
    GetPlaybackState,
}

impl App {
    pub fn handle_request(&self, request: &Request) -> Response {
        use Request::*;
        match request {
            Enqueue { track_id } => self.enqueue(track_id),
            Stop => self.stop(),
            TogglePause => self.toggle_pause(),
            ChangeVolume { volume } => self.set_volume(*volume),
            CompleteFilePath { prefix } => self.completions(prefix),
            GetLibrary => self.list_library(),
            AddToLibrary { path } => self.add_to_library(path.clone()),
            GetPlaybackState => self.get_playback_state(),
        }
    }

    fn enqueue(&self, track_id: &str) -> Response {
        let track_id = TrackId(
            track_id
                .parse()
                .map_err(|_| string_err(format!("Invalid track ID {}", track_id)))?,
        );
        let lib = self.library.lock();
        let track = lib
            .get_track(track_id)
            .ok_or_else(|| string_err(format!("Unknown track {}", track_id.0)))?;
        log::info!("enqueueing track {} from {}", track_id.0, track.file_path);
        let mut player = self.player.lock();
        player.add_to_queue(&track.file_path)?;
        // TODO: this event should come from somewhere else
        self.event_sink.broadcast(&Event::TrackChanged {
            track: (track_id, track).into(),
        });
        done()
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

    fn stop(&self) -> Response {
        self.player.lock().empty_queue();
        done()
    }

    fn set_volume(&self, volume: f32) -> Response {
        let mut player = self.player.lock();
        player.set_volume(volume);
        done()
    }

    fn toggle_pause(&self) -> Response {
        let mut player = self.player.lock();
        player.toggle_pause();
        done()
    }

    fn get_playback_state(&self) -> Response {
        let player = self.player.lock();
        ok(&PlaybackState {
            playing: !player.paused(),
            volume: player.volume(),
        })
    }

    fn completions(&self, prefix: &str) -> Response {
        ok(&CompleteFilePathResp {
            completions: complete_file_path(prefix)?,
        })
    }
}

#[derive(Debug, Serialize)]
struct PlaybackState {
    playing: bool,
    volume: f32,
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

#[derive(Debug, Serialize, Deserialize)]
struct CompleteFilePathResp {
    completions: Vec<String>,
}

#[derive(Serialize)]
#[serde(tag = "type", content = "args")]
pub enum Event<'a> {
    VolumeChanged { new_volume: f32 },
    PlaybackPaused,
    PlaybackResumed,
    TrackChanged { track: Track<'a> },
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
