use super::schema::{
    albums, artists, external_albums, external_artists, external_tracks, playlist_tracks,
    playlists, tracks,
};
use super::tables;
use crate::api::search::SearchResults;
use crate::api::Event;
use crate::api::EventSink;
use crate::errors::Try;
use crate::file_formats;
use crate::ids::{Album, Artist, ExternalId, Id, IdString, LibraryId};
use crate::library::{Playlist, Track};
use crate::model::{AlbumInfo, ArtistInfo, TrackInfo};
use crate::services::ServiceId;
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
    event_sink: Arc<EventSink>,
}

impl Library {
    pub fn tracks(&self) -> Try<impl Iterator<Item = Track>> {
        let rows: Vec<(tables::Track, tables::Album, tables::Artist)> = tracks::table
            .inner_join(albums::table)
            .inner_join(artists::table)
            .log()
            .load(self.connection()?)?;
        // TODO: external IDs
        Ok(rows.into_iter().map(|row| into_track(row, vec![])))
    }

    pub fn get_track(&self, id: LibraryId<crate::ids::Track>) -> Try<Option<Track>> {
        let track_row: Option<(tables::Track, tables::Album, tables::Artist)> = tracks::table
            .find(id.0)
            .inner_join(albums::table)
            .inner_join(artists::table)
            .log()
            .first(self.connection()?)
            .optional()?;
        Ok(if let Some(row) = track_row {
            let external_ids: Vec<(tables::ExternalTrack)> = external_tracks::table
                .filter(external_tracks::track_id.eq(id.0))
                .log()
                .load(self.connection()?)?;
            Some(into_track(row, external_ids))
        } else {
            None
        })
    }

    // TODO: special error enum including track/playlist/artist/album not found
    pub fn create_track(
        &self,
        track: TrackInfo,
        album: LibraryId<Album>,
        artist: LibraryId<Artist>,
        external_id: Option<ExternalId<crate::ids::Track>>,
    ) -> Try<LibraryId<crate::ids::Track>> {
        let id = self.in_transaction(|c| {
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
            let track_id = last_id(c)?;
            if let Some(external_id) = external_id {
                insert_into(external_tracks::table)
                    .values(tables::ExternalTrack {
                        _id: None,
                        track_id,
                        service_id: external_id.service.0,
                        external_id: external_id.id.0,
                    })
                    .log()
                    .execute(c)?;
            }
            Ok(LibraryId::new(track_id))
        })?;
        // TODO: don't re-query
        self.event_sink
            .broadcast(&Event::TrackAddedToLibrary(self.get_track(id)?.unwrap()));
        Ok(id)
    }

    pub fn create_album(
        &self,
        album: AlbumInfo,
        external_id: Option<ExternalId<Album>>,
    ) -> Try<LibraryId<Album>> {
        self.in_transaction(|c| {
            insert_into(albums::table)
                .values(tables::Album {
                    album_id: None,
                    title: album.title,
                    cover_image_url: album.cover_image_url.map(|u| u.into_string()),
                    release_date: album.release_date.map(|d| d.to_string()),
                })
                .log()
                .execute(c)?;
            let album_id = last_id(c)?;
            if let Some(external_id) = external_id {
                insert_into(external_albums::table)
                    .values(tables::ExternalAlbum {
                        _id: None,
                        album_id,
                        service_id: external_id.service.0,
                        external_id: external_id.id.0,
                    })
                    .log()
                    .execute(c)?;
            }
            Ok(LibraryId::new(album_id))
        })
    }

    pub fn find_albums_by_name(&self, title: &str) -> Try<Vec<(LibraryId<Album>, AlbumInfo)>> {
        let albums: Vec<tables::Album> = albums::table
            .filter(albums::title.eq(title))
            .log()
            .load(self.connection()?)?;
        Ok(albums.into_iter().map(into_album).collect())
    }

    pub fn find_external_album(
        &self,
        external_id: &ExternalId<Album>,
    ) -> Try<Option<(LibraryId<Album>, AlbumInfo)>> {
        let album: Option<(tables::ExternalAlbum, tables::Album)> = external_albums::table
            .filter(external_albums::service_id.eq(&external_id.service.0))
            .filter(external_albums::external_id.eq(&external_id.id.0))
            .inner_join(albums::table)
            .log()
            .first(self.connection()?)
            .optional()?;
        Ok(album.map(|(_, a)| into_album(a)))
    }

    pub fn create_artist(
        &self,
        artist: ArtistInfo,
        external_id: Option<ExternalId<Artist>>,
    ) -> Try<LibraryId<Artist>> {
        self.in_transaction(|c| {
            insert_into(artists::table)
                .values(tables::Artist {
                    artist_id: None,
                    name: artist.name,
                    image_url: artist.image_url.map(|u| u.into_string()),
                })
                .log()
                .execute(c)?;
            let artist_id = last_id(c)?;
            if let Some(external_id) = external_id {
                insert_into(external_artists::table)
                    .values(tables::ExternalArtist {
                        _id: None,
                        artist_id,
                        service_id: external_id.service.0,
                        external_id: external_id.id.0,
                    })
                    .log()
                    .execute(c)?;
            }
            Ok(LibraryId::new(artist_id))
        })
    }

    pub fn find_artists_by_name(&self, name: &str) -> Try<Vec<(LibraryId<Artist>, ArtistInfo)>> {
        let artists: Vec<tables::Artist> = artists::table
            .filter(artists::name.eq(name))
            .log()
            .load(self.connection()?)?;
        Ok(artists.into_iter().map(into_artist).collect())
    }

    pub fn find_external_artist(
        &self,
        external_id: &ExternalId<Artist>,
    ) -> Try<Option<(LibraryId<Artist>, ArtistInfo)>> {
        let artist: Option<(tables::ExternalArtist, tables::Artist)> = external_artists::table
            .filter(external_artists::service_id.eq(&external_id.service.0))
            .filter(external_artists::external_id.eq(&external_id.id.0))
            .inner_join(artists::table)
            .log()
            .first(self.connection()?)
            .optional()?;
        Ok(artist.map(|(_, a)| into_artist(a)))
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
        self.event_sink.broadcast(&Event::TrackAddedToPlaylist {
            track_id,
            playlist_id,
        });
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
            event_sink,
        };
        db.setup()?;
        Ok(db)
    }

    pub fn in_transaction<T>(&self, f: impl FnOnce(&SqliteConnection) -> Try<T>) -> Try<T> {
        let c = self.connection()?;
        c.transaction(|| f(c))
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
            .find_albums_by_name(&album.title)
            .map(|a| a.first().map(|(id, _)| *id))
            .transpose()
            .unwrap_or_else(|| self.create_album(album, None))?;
        let artist_id = self
            .find_artists_by_name(&artist.name)
            .map(|a| a.first().map(|(id, _)| *id))
            .transpose()
            .unwrap_or_else(|| self.create_artist(artist, None))?;
        Ok(self.create_track(track, album_id, artist_id, None)?)
    }
}

fn into_track(
    (track, album, artist): (tables::Track, tables::Album, tables::Artist),
    external_ids: Vec<tables::ExternalTrack>,
) -> Track {
    Track {
        track_id: LibraryId::new(track.track_id.unwrap()),
        external_ids: external_ids
            .into_iter()
            .map(|row| ExternalId {
                service: ServiceId(row.service_id),
                id: IdString::new(row.external_id),
            })
            .collect(),
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

fn into_album(a: tables::Album) -> (LibraryId<Album>, AlbumInfo) {
    (
        LibraryId::new(a.album_id.unwrap()),
        AlbumInfo {
            title: a.title,
            cover_image_url: a.cover_image_url.map(|u| u.parse().unwrap()),
            release_date: a.release_date.map(|d| d.parse().unwrap()),
        },
    )
}

fn into_artist(a: tables::Artist) -> (LibraryId<Artist>, ArtistInfo) {
    (
        LibraryId::new(a.artist_id.unwrap()),
        ArtistInfo {
            name: a.name,
            image_url: a.image_url.map(|u| u.parse().unwrap()),
        },
    )
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
