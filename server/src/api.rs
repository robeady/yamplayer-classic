pub mod search;

use crate::errors::Try;
use crate::file_completions::complete_file_path;
use crate::ids::{ExternalId, Id, LibraryId, Playlist, Track};
use crate::library::{Library, TrackSummary};
use crate::model::LoadedTrack;
use crate::player::PlayerApp;
use crate::queue::CurrentTrack;
use crate::services::{ExternalTrack, Service, ServiceId};
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
    pub library: Library,
    // TODO: do we need this here?
    pub event_sink: Arc<EventSink>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "args")]
pub enum Request {
    GetPlaybackState,
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
    ListAlbums,
    ListArtists,
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
            GetPlaybackState => self.get_playback_state(),
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
            ListAlbums => self.list_albums(),
            ListArtists => self.list_artists(),
            ListPlaylists => self.list_playlists(),
            GetPlaylist { ref id } => ok(&self
                .library
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

    fn load_track(&self, track_id: &Id<Track>) -> Try<LoadedTrack> {
        match track_id {
            Id::Library(lib_track_id) => {
                let track = self
                    .library
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
                    for ext_id in track.external_ids {
                        if let Some(service) = self.services.get(&ext_id.service) {
                            log::info!("fetching track {} from {}", track_id, ext_id);
                            return service.fetch(&ext_id.id);
                        }
                    }
                    Err(anyhow!(
                        "no file or external source available for track {}",
                        track_id
                    ))
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
        let tracks: Result<BTreeMap<Id<Track>, Option<TrackSummary>>, anyhow::Error> = track_ids
            .iter()
            .map(|id| {
                let id = id.parse()?;
                let track = match id {
                    Id::Library(lib_id) => self.library.get_track(lib_id)?,
                    Id::External(_) => None,
                };
                Ok((id, track))
            })
            .collect();
        ok(&tracks?)
    }

    fn list_library(&self) -> Response {
        let tracks = self
            .library
            .tracks()
            .context("error loading tracks")?
            .map(Into::into)
            .collect();
        ok(&LibraryListing { tracks })
    }

    fn add_to_library(&self, track_file_path: String) -> Response {
        self.library.add_local_track(track_file_path)?;
        done()
    }

    fn get_playback_state(&self) -> Response {
        ok(&self.player.playback_state())
    }

    fn list_albums(&self) -> Response {
        ok(&self.library.albums()?.collect::<Vec<_>>())
    }

    fn list_artists(&self) -> Response {
        ok(&self.library.artists()?.collect::<Vec<_>>())
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
        let track_id: Id<Track> = track_id.parse()?;
        let playlist_id: LibraryId<Playlist> = playlist_id.parse()?;
        match track_id {
            Id::Library(track_id) => self.add_library_track_to_playlist(track_id, playlist_id)?,
            Id::External(track_id) => {
                // verify that the playlist exists first
                if !self.library.playlist_exists(playlist_id)? {
                    Err(anyhow!("non existent playlist {}", playlist_id.0))?
                }
                let track_id = self.add_external_track_to_library(track_id)?;
                self.add_library_track_to_playlist(track_id, playlist_id)?;
            }
        }
        done()
    }

    fn add_external_track_to_library(&self, track_id: ExternalId<Track>) -> Try<LibraryId<Track>> {
        let svc = self
            .services
            .get(&track_id.service)
            .ok_or_else(|| anyhow!("unrecognised service {}", track_id.id.0))?;
        let ExternalTrack {
            track_id,
            track_info,
            artist_id,
            artist_info,
            album_id,
            album_info,
        } = svc.track_info(&track_id.id)?;
        let album_id = self
            .library
            .find_external_album(&album_id)
            .map(|a| a.map(|(id, _)| id))
            .transpose()
            .unwrap_or_else(|| self.library.create_album(album_info, Some(album_id)))?;
        let artist_id = self
            .library
            .find_external_artist(&artist_id)
            .map(|a| a.map(|(id, _)| id))
            .transpose()
            .unwrap_or_else(|| self.library.create_artist(artist_info, Some(artist_id)))?;
        Ok(self
            .library
            .create_track(track_info, album_id, artist_id, Some(track_id))?)
    }

    fn add_library_track_to_playlist(
        &self,
        track_id: LibraryId<Track>,
        playlist_id: LibraryId<Playlist>,
    ) -> Try<()> {
        self.library.add_track_to_playlist(track_id, playlist_id)
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
            .resolve(results)
            .context("failed to resolve search results")?)
    }
}

#[derive(Serialize)]
struct LibraryListing {
    tracks: Vec<TrackSummary>,
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
    TrackAddedToLibrary(TrackSummary),
    TrackAddedToPlaylist {
        track_id: LibraryId<Track>,
        playlist_id: LibraryId<Playlist>,
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
