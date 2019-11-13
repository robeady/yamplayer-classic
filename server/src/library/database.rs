use super::schema::{albums, artists, playlist_tracks, playlists, tracks};
use super::tables;
use crate::api::search::SearchResults;
use crate::api::EventSink;
use crate::errors::Try;
use crate::file_formats;
use crate::library::{Playlist, Track};
use crate::model::{
    AlbumId, AlbumInfo, ArtistId, ArtistInfo, LibraryTrackId, PlaylistId, TrackInfo,
};
use anyhow::anyhow;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel::{insert_into, select};
use log;
use std::collections::HashMap;
use std::sync::Arc;
use thread_local::CachedThreadLocal;

pub struct Library {
    connection: CachedThreadLocal<SqliteConnection>,
    file_path: String,
}

impl Library {
    pub fn tracks(&self) -> Try<impl Iterator<Item = Track>> {
        let rows: Vec<(tables::Track, tables::Album, tables::Artist)> = tracks::table
            .inner_join(albums::table)
            .inner_join(artists::table)
            .load(self.connection()?)?;
        Ok(rows.into_iter().map(into_track))
    }

    pub fn get_track(&self, id: LibraryTrackId) -> Try<Option<Track>> {
        let row: Option<(tables::Track, tables::Album, tables::Artist)> = tracks::table
            .find(id.0)
            .inner_join(albums::table)
            .inner_join(artists::table)
            .first(self.connection()?)
            .optional()?;
        Ok(row.map(into_track))
    }

    // TODO: special error enum including track/playlist/artist/album not found
    pub fn create_track(
        &self,
        track: TrackInfo,
        album: AlbumId,
        artist: ArtistId,
    ) -> Try<LibraryTrackId> {
        let c = self.connection()?;
        insert_into(tracks::table)
            .values(tables::Track {
                track_id: None,
                album_id: album.0,
                artist_id: artist.0,
                title: track.title,
                isrc: track.isrc,
                duration_secs: track.duration_secs,
                file_path: track.file_path,
            })
            .execute(c)?;
        Ok(LibraryTrackId(last_id(c)?))
    }

    pub fn create_album(&self, album: AlbumInfo) -> Try<AlbumId> {
        let c = self.connection()?;
        insert_into(albums::table)
            .values(tables::Album {
                album_id: None,
                title: album.title,
                cover_image_url: album.cover_image_url.map(|u| u.into_string()),
                release_date: album.release_date.map(|d| d.to_string()),
            })
            .execute(c)?;
        Ok(AlbumId(last_id(c)?))
    }

    pub fn find_albums(&self, title: &str) -> Try<Vec<(AlbumId, AlbumInfo)>> {
        let albums: Vec<tables::Album> = albums::table
            .filter(albums::title.eq(title))
            .load(self.connection()?)?;
        Ok(albums
            .into_iter()
            .map(|a| {
                (
                    AlbumId(a.album_id.unwrap()),
                    AlbumInfo {
                        title: a.title,
                        cover_image_url: a.cover_image_url.map(|u| u.parse().unwrap()),
                        release_date: a.release_date.map(|d| d.parse().unwrap()),
                    },
                )
            })
            .collect())
    }

    pub fn create_artist(&self, artist: ArtistInfo) -> Try<ArtistId> {
        let c = self.connection()?;
        insert_into(artists::table)
            .values(tables::Artist {
                artist_id: None,
                name: artist.name,
                image_url: artist.image_url.map(|u| u.into_string()),
            })
            .execute(c)?;
        Ok(ArtistId(last_id(c)?))
    }

    pub fn find_artists(&self, name: &str) -> Try<Vec<(ArtistId, ArtistInfo)>> {
        let artists: Vec<tables::Artist> = artists::table
            .filter(artists::name.eq(name))
            .load(self.connection()?)?;
        Ok(artists
            .into_iter()
            .map(|a| {
                (
                    ArtistId(a.artist_id.unwrap()),
                    ArtistInfo {
                        name: a.name,
                        image_url: a.image_url.map(|u| u.parse().unwrap()),
                    },
                )
            })
            .collect())
    }

    pub fn playlists(&self) -> Try<impl Iterator<Item = Playlist>> {
        let rows: Vec<(tables::Playlist, tables::PlaylistTrack)> = playlists::table
            .inner_join(playlist_tracks::table)
            .load(self.connection()?)?;
        let mut name_and_track_ids_by_playlist_id = HashMap::new();
        for (playlist, playlist_track) in rows {
            name_and_track_ids_by_playlist_id
                .entry(PlaylistId(playlist_track.playlist_id))
                .or_insert((playlist.name, Vec::new()))
                .1
                .push(LibraryTrackId(playlist_track.track_id))
        }
        Ok(name_and_track_ids_by_playlist_id
            .into_iter()
            .map(|(id, (name, track_ids))| Playlist {
                id,
                name,
                track_ids,
            }))
    }

    pub fn create_playlist(&self, name: String) -> Try<PlaylistId> {
        let c = self.connection()?;
        log::info!("inserting playlist");
        insert_into(playlists::table)
            .values(tables::Playlist {
                playlist_id: None,
                name,
            })
            .execute(c)?;
        log::info!("finding id");
        let id = last_id(c)?;
        log::info!("got it");
        Ok(PlaylistId(id))
    }

    pub fn get_playlist(&self, id: PlaylistId) -> Try<Option<Playlist>> {
        let rows: Vec<(tables::Playlist, tables::PlaylistTrack)> = playlists::table
            .inner_join(playlist_tracks::table)
            .filter(playlists::playlist_id.eq(Some(id.0)))
            .load(self.connection()?)?;
        Ok(if rows.is_empty() {
            None
        } else {
            let track_ids = rows
                .iter()
                .map(|(_, t)| LibraryTrackId(t.track_id))
                .collect();
            let (tables::Playlist { playlist_id, name }, _) = rows.into_iter().nth(0).unwrap();
            Some(Playlist {
                id: PlaylistId(playlist_id.unwrap()),
                name,
                track_ids,
            })
        })
    }

    pub fn add_track_to_playlist(
        &self,
        track_id: LibraryTrackId,
        playlist_id: PlaylistId,
    ) -> Try<()> {
        let c = self.connection()?;
        insert_into(playlist_tracks::table)
            .values(tables::PlaylistTrack {
                _id: None,
                playlist_id: playlist_id.0,
                track_id: track_id.0,
            })
            .execute(c)?;
        // TODO: what if track id or playlist id don't exist (causes foreign key error)
        Ok(())
    }

    pub fn resolve(&self, mut search_results: SearchResults) -> Try<SearchResults> {
        // TODO: search
        Ok(search_results)
    }

    pub fn new(file_path: String, event_sink: Arc<EventSink>) -> Try<Library> {
        let db = Library {
            connection: CachedThreadLocal::new(),
            file_path,
        };
        db.setup()?;
        Ok(db)
    }

    pub fn in_transaction<T>(&self, f: impl FnOnce() -> Try<T>) -> Try<T> {
        self.connection()?.transaction(f)
    }

    fn setup(&self) -> Try<()> {
        Ok(embedded_migrations::run(self.connection()?)?)
    }

    fn connection(&self) -> diesel::ConnectionResult<&SqliteConnection> {
        // TODO: create if not open yet?
        // TODO: enforce foreign key constraints
        self.connection
            .get_or_try(|| SqliteConnection::establish(&self.file_path))
    }

    pub fn add_local_track(&self, file_path: String) -> Try<LibraryTrackId> {
        let (track, album, artist) = if file_path.ends_with(".mp3") {
            file_formats::mp3::read_metadata(file_path)?
        } else if file_path.ends_with(".flac") {
            file_formats::flac::read_metadata(file_path)?
        } else {
            return Err(anyhow!("unsupported file type {}", file_path));
        };
        let album_id = self
            .find_albums(&album.title)?
            .first()
            .map(|(id, _)| *id)
            .unwrap_or(self.create_album(album)?);
        let artist_id = self
            .find_artists(&artist.name)?
            .first()
            .map(|(id, _)| *id)
            .unwrap_or(self.create_artist(artist)?);
        Ok(self.create_track(track, album_id, artist_id)?)
    }
}

fn into_track((track, album, artist): (tables::Track, tables::Album, tables::Artist)) -> Track {
    Track {
        track_id: LibraryTrackId(track.track_id.unwrap()),
        track_info: TrackInfo {
            title: track.title,
            isrc: track.isrc,
            duration_secs: track.duration_secs,
            file_path: track.file_path,
        },
        artist_id: ArtistId(track.artist_id),
        artist_info: ArtistInfo {
            name: artist.name,
            image_url: artist.image_url.map(|u| u.parse().unwrap()),
        },
        album_id: AlbumId(track.album_id),
        album_info: AlbumInfo {
            title: album.title,
            cover_image_url: album.cover_image_url.map(|u| u.parse().unwrap()),
            release_date: album.release_date.map(|d| d.parse().unwrap()),
        },
    }
}

embed_migrations!();

no_arg_sql_function!(last_insert_rowid, diesel::sql_types::BigInt);

/// Returns the rowid of the last row inserted by this database connection.
fn last_id(con: &SqliteConnection) -> diesel::result::QueryResult<i64> {
    select(last_insert_rowid).first(con)
}
