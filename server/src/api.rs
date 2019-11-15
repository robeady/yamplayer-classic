pub mod search;

use crate::errors::Try;
use crate::file_completions::complete_file_path;
use crate::ids::{ExternalId, Id, LibraryId, Playlist};
use crate::library::{Library, Track};
use crate::model::LoadedTrack;
use crate::player::PlayerApp;
use crate::queue::CurrentTrack;
use crate::services::{Service, ServiceId};
use anyhow::Context;
use fstrings::{f, format_args_f};
use parking_lot::{Mutex, RwLock};
use serde_derive::{Deserialize, Serialize};
use slotmap::{DenseSlotMap, Key};
use std::collections::{BTreeMap, HashMap};
use std::convert::Into;
use std::fs;
use std::sync::Arc;

pub struct App {
    pub services: HashMap<ServiceId, Box<dyn Service>>,
    pub player: PlayerApp,
    // TODO: no need for a mutex any more
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
    AddTrackToPlaylist {
        track_id: String,
        playlist_id: String,
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
            GetPlaylist { ref id } => ok(&self
                .library
                .lock()
                .get_playlist(id.parse()?)
                .context("failed to get playlist")?),
            AddTrackToPlaylist {
                track_id,
                playlist_id,
            } => self.add_track_to_playlist(track_id, playlist_id),
            Search { ref query } => self.search(query),
        }
    }

    fn enqueue(&self, track_id: &str) -> Response {
        let track_id = track_id.parse()?;
        let track = self.load_track(&track_id)?;
        self.player.add_to_queue(track_id, track)?;
        done()
    }

    //    fn load_track(&self, track_id: &TrackId) -> Try<LoadedTrack> {
    //        match track_id {
    //            TrackId::Library(lib_track_id) => {
    //                let lib = self.library.lock();
    //                let track = lib
    //                    .get_track(*lib_track_id)?
    //                    .ok_or_else(|| anyhow!("Unknown track {}", track_id))?;
    //                if let Some(file_path) = track.track_info.file_path {
    //                    log::info!("loading track {} from {}", track_id, file_path);
    //                    Ok(LoadedTrack {
    //                        data: fs::read(&file_path)
    //                            .with_context(|| f!("failed to load track file {}", file_path))?,
    //                        duration_secs: track.track_info.duration_secs,
    //                    })
    //                } else {
    //                    Err(anyhow!("no file available for track {}", track_id))
    //                }
    //            }
    //            TrackId::External(ExternalTrackId {
    //                ref service_id,
    //                ref track_id,
    //            }) => {
    //                let svc = self
    //                    .services
    //                    .get(service_id)
    //                    .ok_or_else(|| anyhow!("unknown service for track ID {}", track_id))?;
    //                svc.fetch(track_id)
    //            }
    //        }
    //    }

    fn load_track(&self, track_id: &Id<crate::ids::Track>) -> Try<LoadedTrack> {
        match track_id {
            Id::Library(lib_track_id) => {
                let lib = self.library.lock();
                let track = lib
                    .get_track(*lib_track_id)?
                    .ok_or_else(|| anyhow!("Unknown track {}", track_id))?;
                if let Some(file_path) = track.track_info.file_path {
                    log::info!("loading track {} from {}", track_id, file_path);
                    Ok(LoadedTrack {
                        data: fs::read(&file_path)
                            .with_context(|| f!("failed to load track file {}", file_path))?,
                        duration_secs: track.track_info.duration_secs,
                    })
                } else {
                    Err(anyhow!("no file available for track {}", track_id))
                }
            }
            Id::External(ExternalId { service, id }) => {
                let svc = self
                    .services
                    .get(service)
                    .ok_or_else(|| anyhow!("unknown service for track ID {}", track_id))?;
                svc.fetch(id)
            }
        }
    }

    fn get_tracks(&self, track_ids: &[String]) -> Response {
        let lib = self.library.lock();
        let tracks: Result<BTreeMap<Id<crate::ids::Track>, Option<Track>>, anyhow::Error> =
            track_ids
                .iter()
                .map(|id| {
                    let id = id.parse()?;
                    let track = match id {
                        Id::Library(lib_id) => lib.get_track(lib_id)?,
                        Id::External(_) => None,
                    };
                    Ok((id, track))
                })
                .collect();
        ok(&tracks?)
    }

    fn list_library(&self) -> Response {
        let lib = self.library.lock();
        let tracks = lib
            .tracks()
            .context("error loading tracks")?
            .map(Into::into)
            .collect();
        ok(&LibraryListing { tracks })
    }

    fn add_to_library(&self, track_file_path: String) -> Response {
        self.library.lock().add_local_track(track_file_path)?;
        done()
    }

    fn get_playback_state(&self) -> Response {
        ok(&self.player.playback_state())
    }

    fn list_playlists(&self) -> Response {
        #[derive(Serialize)]
        struct Playlists {
            playlists: Vec<PlaylistInfo>,
        }
        #[derive(Serialize)]
        struct PlaylistInfo {
            id: LibraryId<Playlist>,
            name: String,
        }
        ok(&Playlists {
            playlists: self
                .library
                .lock()
                .playlists()
                .context("Error loading playlists")?
                .map(|p| PlaylistInfo {
                    id: p.id,
                    name: p.name,
                })
                .collect(),
        })
    }

    fn add_track_to_playlist(&self, track_id: &str, playlist_id: &str) -> Response {
        let track_id: Id<crate::ids::Track> = track_id.parse()?;
        let playlist_id: LibraryId<Playlist> = playlist_id.parse()?;
        match track_id {
            Id::Library(track_id) => self.add_library_track_to_playlist(track_id, playlist_id)?,
            Id::External(track_id) => {
                // verify that the playlist exists first
                if !self.library.lock().playlist_exists(playlist_id)? {
                    Err(anyhow!("non existent playlist {}", playlist_id.0))?
                }
                let track_id = self.add_external_track_to_library(track_id)?;
                self.add_library_track_to_playlist(track_id, playlist_id)?;
            }
        }
        done()
    }

    fn add_external_track_to_library(
        &self,
        track_id: ExternalId<crate::ids::Track>,
    ) -> Try<LibraryId<crate::ids::Track>> {
        let svc = self
            .services
            .get(&track_id.service)
            .ok_or_else(|| anyhow!("unrecognised service {}", track_id.id.0))?;
        //svc.
        todo!()
    }

    fn add_library_track_to_playlist(
        &self,
        track_id: LibraryId<crate::ids::Track>,
        playlist_id: LibraryId<Playlist>,
    ) -> Try<()> {
        self.library
            .lock()
            .add_track_to_playlist(track_id, playlist_id)
    }

    fn completions(&self, prefix: &str) -> Response {
        ok(&CompleteFilePathResp {
            completions: complete_file_path(prefix)?,
        })
    }

    fn search(&self, query: &str) -> Response {
        let service = self
            .services
            .values()
            .next()
            .ok_or_else(|| anyhow!("No search services registered"))?;
        let results = service.search(query)?;
        ok(&self
            .library
            .lock()
            .resolve(results)
            .context("failed to resolve search results")?)
    }
}

#[derive(Serialize)]
struct LibraryListing {
    tracks: Vec<Track>,
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

impl From<anyhow::Error> for Payload {
    fn from(e: anyhow::Error) -> Self {
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
