use super::{Playlist, Track};
use crate::api::search::SearchResults;
use crate::api::EventSink;
use crate::errors::Try;
use crate::library::Library;
use crate::model::{LibraryTrackId, PlaylistId, TrackId};
use anyhow::anyhow;
use fstrings::{f, format_args_f};
use id3::Tag;
use log;
use serde_derive::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::convert::Infallible;
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;

pub struct InMemoryLibrary {
    tracks: BTreeMap<LibraryTrackId, Track>,
    playlists: BTreeMap<PlaylistId, Playlist>,
    event_sink: Arc<EventSink>,
}

type Success<T> = Result<T, Infallible>;

impl Library for InMemoryLibrary {
    type Error = Infallible;

    fn tracks(&self) -> Result<Box<dyn Iterator<Item = Track> + '_>, Self::Error> {
        Ok(Box::new(self.tracks.values().cloned()))
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn get_track(&self, id: &LibraryTrackId) -> Result<Option<Track>, Self::Error> {
        Ok(self.tracks.get(id).cloned())
    }

    fn create_playlist(&mut self, name: String) -> Result<PlaylistId, Self::Error> {
        let playlist_id = self.next_playlist_id();
        self.playlists.insert(
            playlist_id,
            Playlist {
                id: playlist_id,
                name,
                track_ids: Vec::new(),
            },
        );
        Ok(playlist_id)
    }

    fn add_track_to_playlist(
        &mut self,
        track_id: LibraryTrackId,
        playlist_id: PlaylistId,
    ) -> Result<bool, Self::Error> {
        //        self.tracks
        //            .get(&track_id)
        //            .ok_or_else(|| anyhow!("Unknown track {}", track_id.0))?;
        //        self.playlists
        //            .get_mut(&playlist_id)
        //            .ok_or_else(|| anyhow!("Unknown playlist {}", playlist_id.0))?
        //            .track_ids
        //            .push(track_id);
        //        Ok(())
        // TODO: we need a better way of conveying this error,
        // probably a new error enum for this method
        if let None = self.tracks.get(&track_id) {
            return Ok(false);
        }
        let p = self.playlists.get_mut(&playlist_id);
        if let Some(playlist) = p {
            playlist.track_ids.push(track_id);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn get_playlist(&self, id: PlaylistId) -> Result<Option<Playlist>, Self::Error> {
        Ok(self.playlists.get(&id).cloned())
    }

    fn playlists(&self) -> Result<Box<dyn Iterator<Item = Playlist> + '_>, Self::Error> {
        Ok(Box::new(self.playlists.values().cloned()))
    }

    fn resolve(&self, mut search_results: SearchResults) -> Result<SearchResults, Self::Error> {
        //        for track_result in &mut search_results.tracks {
        //            if let Some((id, _)) = self
        //                .tracks
        //                .iter()
        //                .find(|(_, t)| t.external_id.as_ref() == Some(&track_result.track.external_id))
        //            {
        //                track_result.track.library_id = Some(TrackId::Library(*id))
        //            }
        //        }
        // TODO: tracks and artists
        Ok(search_results)
    }

    fn create_track(&self) -> Result<(), Self::Error> {
        unimplemented!()
    }
}

impl InMemoryLibrary {
    pub fn new(event_sink: Arc<EventSink>) -> Self {
        InMemoryLibrary {
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

    fn add_track_mp3(&mut self, file_path: String) -> Try<LibraryTrackId> {
        self.tracks.insert(track_id, track);
        Ok(track_id)
    }

    fn add_track_flac(&mut self, file_path: String) -> Try<LibraryTrackId> {
        self.tracks.insert(track_id, track);
        Ok(track_id)
    }
}
