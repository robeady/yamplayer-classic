use super::{Playlist, Track};
use crate::api::search::SearchResults;
use crate::api::EventSink;
use crate::errors::Try;
use crate::library::Library;
use crate::model::{
    AlbumId, AlbumInfo, ArtistId, ArtistInfo, LibraryTrackId, PlaylistId, TrackInfo,
};
use std::collections::BTreeMap;
use std::sync::Arc;

pub struct InMemoryLibrary {
    tracks: BTreeMap<LibraryTrackId, Track>,
    playlists: BTreeMap<PlaylistId, Playlist>,
    event_sink: Arc<EventSink>,
}

impl Library for InMemoryLibrary {
    fn tracks(&self) -> Try<Box<dyn Iterator<Item = Track> + '_>> {
        Ok(Box::new(self.tracks.values().cloned()))
    }

    fn get_track(&self, id: LibraryTrackId) -> Try<Option<Track>> {
        Ok(self.tracks.get(&id).cloned())
    }

    fn create_track(
        &self,
        track: TrackInfo,
        album: AlbumId,
        artist: ArtistId,
    ) -> Try<LibraryTrackId> {
        todo!()
    }

    fn create_album(&self, album: AlbumInfo) -> Try<AlbumId> {
        todo!()
    }

    fn find_albums(&self, title: &str) -> Try<Vec<(AlbumId, AlbumInfo)>> {
        Ok(Vec::new())
    }

    fn create_artist(&self, artist: ArtistInfo) -> Try<ArtistId> {
        todo!()
    }

    fn find_artists(&self, name: &str) -> Try<Vec<(ArtistId, ArtistInfo)>> {
        Ok(Vec::new())
    }

    fn playlists(&self) -> Try<Box<dyn Iterator<Item = Playlist> + '_>> {
        Ok(Box::new(self.playlists.values().cloned()))
    }

    fn create_playlist(&mut self, name: String) -> Try<PlaylistId> {
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

    fn get_playlist(&self, id: PlaylistId) -> Try<Option<Playlist>> {
        Ok(self.playlists.get(&id).cloned())
    }

    fn add_track_to_playlist(
        &mut self,
        track_id: LibraryTrackId,
        playlist_id: PlaylistId,
    ) -> Try<()> {
        //        self.tracks
        //            .get(&track_id)
        //            .ok_or_else(|| anyhow!("Unknown track {}", track_id.0))?;
        //        self.playlists
        //            .get_mut(&playlist_id)
        //            .ok_or_else(|| anyhow!("Unknown playlist {}", playlist_id.0))?
        //            .track_ids
        //            .push(track_id);
        //        Ok(())
        // TODO: we need a better way of conveying these errors...
        // probably a new error enum
        if self.tracks.get(&track_id).is_none() {
            // return Ok(false);
            todo!("no such track");
        }
        let p = self.playlists.get_mut(&playlist_id);
        if let Some(playlist) = p {
            playlist.track_ids.push(track_id);
            Ok(())
        } else {
            todo!("no such playlist");
        }
    }

    fn resolve(&self, mut search_results: SearchResults) -> Try<SearchResults> {
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

    //    fn add_track_mp3(&mut self, file_path: String) -> Try<LibraryTrackId> {
    //        self.tracks.insert(track_id, track);
    //        Ok(track_id)
    //    }
    //
    //    fn add_track_flac(&mut self, file_path: String) -> Try<LibraryTrackId> {
    //        self.tracks.insert(track_id, track);
    //        Ok(track_id)
    //    }
}
