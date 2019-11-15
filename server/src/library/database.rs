use super::schema::{albums, artists, playlist_tracks, playlists, tracks};
use super::tables;
use crate::api::search::SearchResults;
use crate::api::EventSink;
use crate::errors::Try;
use crate::file_formats;
use crate::ids::{Album, Artist, Id, LibraryId};
use crate::library::{Playlist, Track};
use crate::model::{AlbumInfo, ArtistInfo, TrackInfo};
use diesel::dsl::exists;
use diesel::prelude::*;
use diesel::query_builder::QueryFragment;
use diesel::sqlite::{Sqlite, SqliteConnection};
use diesel::{debug_query, insert_into, select, sql_types};
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
            .log()
            .load(self.connection()?)?;
        Ok(rows.into_iter().map(into_track))
    }

    pub fn get_track(&self, id: LibraryId<crate::ids::Track>) -> Try<Option<Track>> {
        let row: Option<(tables::Track, tables::Album, tables::Artist)> = tracks::table
            .find(id.0)
            .inner_join(albums::table)
            .inner_join(artists::table)
            .log()
            .first(self.connection()?)
            .optional()?;
        Ok(row.map(into_track))
    }

    // TODO: special error enum including track/playlist/artist/album not found
    pub fn create_track(
        &self,
        track: TrackInfo,
        album: LibraryId<Album>,
        artist: LibraryId<Artist>,
    ) -> Try<LibraryId<crate::ids::Track>> {
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
            .log()
            .execute(c)?;
        Ok(LibraryId::new(last_id(c)?))
    }

    pub fn create_album(&self, album: AlbumInfo) -> Try<LibraryId<Album>> {
        let c = self.connection()?;
        insert_into(albums::table)
            .values(tables::Album {
                album_id: None,
                title: album.title,
                cover_image_url: album.cover_image_url.map(|u| u.into_string()),
                release_date: album.release_date.map(|d| d.to_string()),
            })
            .log()
            .execute(c)?;
        Ok(LibraryId::new(last_id(c)?))
    }

    pub fn find_albums(&self, title: &str) -> Try<Vec<(LibraryId<Album>, AlbumInfo)>> {
        let albums: Vec<tables::Album> = albums::table
            .filter(albums::title.eq(title))
            .log()
            .load(self.connection()?)?;
        Ok(albums
            .into_iter()
            .map(|a| {
                (
                    LibraryId::new(a.album_id.unwrap()),
                    AlbumInfo {
                        title: a.title,
                        cover_image_url: a.cover_image_url.map(|u| u.parse().unwrap()),
                        release_date: a.release_date.map(|d| d.parse().unwrap()),
                    },
                )
            })
            .collect())
    }

    pub fn create_artist(&self, artist: ArtistInfo) -> Try<LibraryId<Artist>> {
        let c = self.connection()?;
        insert_into(artists::table)
            .values(tables::Artist {
                artist_id: None,
                name: artist.name,
                image_url: artist.image_url.map(|u| u.into_string()),
            })
            .log()
            .execute(c)?;
        Ok(LibraryId::new(last_id(c)?))
    }

    pub fn find_artists(&self, name: &str) -> Try<Vec<(LibraryId<Artist>, ArtistInfo)>> {
        let artists: Vec<tables::Artist> = artists::table
            .filter(artists::name.eq(name))
            .log()
            .load(self.connection()?)?;
        Ok(artists
            .into_iter()
            .map(|a| {
                (
                    LibraryId::new(a.artist_id.unwrap()),
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
            .log()
            .load(self.connection()?)?;
        let mut name_and_track_ids_by_playlist_id = HashMap::new();
        for (playlist, playlist_track) in rows {
            name_and_track_ids_by_playlist_id
                .entry(LibraryId::new(playlist_track.playlist_id))
                .or_insert((playlist.name, Vec::new()))
                .1
                .push(LibraryId::new(playlist_track.track_id))
        }
        Ok(name_and_track_ids_by_playlist_id
            .into_iter()
            .map(|(id, (name, track_ids))| Playlist {
                id,
                name,
                track_ids,
            }))
    }

    pub fn create_playlist(&self, name: String) -> Try<LibraryId<crate::ids::Playlist>> {
        let c = self.connection()?;
        insert_into(playlists::table)
            .values(tables::Playlist {
                playlist_id: None,
                name,
            })
            .log()
            .execute(c)?;
        let id = last_id(c)?;
        Ok(LibraryId::new(id))
    }

    pub fn get_playlist(&self, id: LibraryId<crate::ids::Playlist>) -> Try<Option<Playlist>> {
        let rows: Vec<(tables::Playlist, tables::PlaylistTrack)> = playlists::table
            .find(id.0)
            .inner_join(playlist_tracks::table)
            .log()
            .load(self.connection()?)?;
        Ok(if rows.is_empty() {
            None
        } else {
            let track_ids = rows
                .iter()
                .map(|(_, t)| LibraryId::new(t.track_id))
                .collect();
            let (tables::Playlist { playlist_id, name }, _) = rows.into_iter().nth(0).unwrap();
            Some(Playlist {
                id: LibraryId::new(playlist_id.unwrap()),
                name,
                track_ids,
            })
        })
    }

    pub fn playlist_exists(&self, id: LibraryId<crate::ids::Playlist>) -> Try<bool> {
        Ok(select(exists(playlists::table.find(id.0)))
            .log()
            .get_result(self.connection()?)?)
    }

    pub fn add_track_to_playlist(
        &self,
        track_id: LibraryId<crate::ids::Track>,
        playlist_id: LibraryId<crate::ids::Playlist>,
    ) -> Try<()> {
        let c = self.connection()?;
        insert_into(playlist_tracks::table)
            .values(tables::PlaylistTrack {
                _id: None,
                playlist_id: playlist_id.0,
                track_id: track_id.0,
            })
            .log()
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

    fn connection(&self) -> ConnectionResult<&SqliteConnection> {
        // TODO: create if not open yet?
        // TODO: enforce foreign key constraints
        self.connection
            .get_or_try(|| SqliteConnection::establish(&self.file_path))
    }

    pub fn add_local_track(&self, file_path: String) -> Try<LibraryId<crate::ids::Track>> {
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
        track_id: LibraryId::new(track.track_id.unwrap()),
        track_info: TrackInfo {
            title: track.title,
            isrc: track.isrc,
            duration_secs: track.duration_secs,
            file_path: track.file_path,
        },
        artist_id: LibraryId::new(track.artist_id),
        artist_info: ArtistInfo {
            name: artist.name,
            image_url: artist.image_url.map(|u| u.parse().unwrap()),
        },
        album_id: LibraryId::new(track.album_id),
        album_info: AlbumInfo {
            title: album.title,
            cover_image_url: album.cover_image_url.map(|u| u.parse().unwrap()),
            release_date: album.release_date.map(|d| d.parse().unwrap()),
        },
    }
}

trait QueryFragmentLogExt {
    fn log(self) -> Self;
}

impl<Q: QueryFragment<Sqlite>> QueryFragmentLogExt for Q {
    fn log(self) -> Self {
        log::info!("{}", debug_query::<Sqlite, _>(&self));
        self
    }
}

embed_migrations!();

no_arg_sql_function!(last_insert_rowid, sql_types::BigInt);

/// Returns the rowid of the last row inserted by this database connection.
fn last_id(con: &SqliteConnection) -> QueryResult<i64> {
    select(last_insert_rowid).first(con)
}
