use crate::api::search::SearchResults;
use crate::errors::Try;
use crate::library::{Library, Playlist, Track};
use crate::model::{
    AlbumId, AlbumInfo, ArtistId, ArtistInfo, LibraryTrackId, PlaylistId, TrackInfo,
};
use anyhow::Error;
use rusqlite::{params, Connection, Row};
use std::ops::Index;
use thread_local::CachedThreadLocal;

pub struct DbLibrary {
    connection: CachedThreadLocal<Connection>,
    file_name: String,
}

impl Library for DbLibrary {
    type Error = rusqlite::Error;

    fn add_track(&mut self, file_path: String) -> Result<LibraryTrackId, Error> {
        unimplemented!()
    }

    fn create_track(&self) -> Result<(), Self::Error> {
        unimplemented!()
    }

    fn tracks(&self) -> Result<Box<dyn Iterator<Item = Track>>, Self::Error> {
        unimplemented!()
    }

    fn get_track(&self, id: &LibraryTrackId) -> Result<Option<Track>, Self::Error> {
        self.connection()?.query_row(
            "SELECT \
             /* 0 */ tracks.track_id, \
             /* 1 */ tracks.album_id, \
             /* 2 */ tracks.artist_id, \
             /* 3 */ tracks.title, \
             /* 4 */ tracks.isrc, \
             /* 5 */ tracks.duration_secs, \
             /* 6 */ tracks.file_path, \
             /* 7 */ albums.title, \
             /* 8 */ albums.cover_image_url, \
             /* 9 */ albums.release_date, \
             /* 10 */ artists.name, \
             /* 11 */ artists.image_url \
             FROM tracks \
             JOIN albums USING album_id \
             JOIN artists USING artist_id \
             WHERE tracks.track_id = ?1",
            params![id.0],
            |row| {
                Ok(Some(Track {
                    track_id: LibraryTrackId(row.get(0)?),
                    album_id: AlbumId(row.get(2)?),
                    artist_id: ArtistId(row.get(1)?),
                    track_info: TrackInfo {
                        title: row.get(3)?,
                        isrc: row.get(4)?,
                        duration_secs: row.get::<_, f64>(5)? as f32,
                    },
                    file_path: row.get(6)?,
                    album_info: AlbumInfo {
                        title: row.get(7)?,
                        cover_image_url: row.get(8)?,
                        release_date: row.get(9)?,
                    },
                    artist_info: ArtistInfo {
                        name: row.get(10)?,
                        image_url: row.get(11)?,
                    },
                }))
            },
        )
    }

    fn create_playlist(&mut self, name: String) -> Result<PlaylistId, Self::Error> {
        unimplemented!()
    }

    fn get_playlist(&self, id: PlaylistId) -> Result<Option<Playlist>, Self::Error> {
        unimplemented!()
    }

    fn playlists(&self) -> Result<Box<dyn Iterator<Item = Playlist>>, Self::Error> {
        unimplemented!()
    }

    fn add_track_to_playlist(
        &mut self,
        track_id: LibraryTrackId,
        playlist_id: PlaylistId,
    ) -> Result<bool, Self::Error> {
        unimplemented!()
    }

    fn resolve(&self, mut search_results: SearchResults) -> Result<SearchResults, Self::Error> {
        unimplemented!()
    }
}

const SCHEMA: &str = include_str!("schema.sql");

impl DbLibrary {
    pub fn connect_or_create(file_name: String) -> Try<DbLibrary> {
        let db = DbLibrary {
            connection: CachedThreadLocal::new(),
            file_name,
        };
        db.setup()?;
        Ok(db)
    }

    fn setup(&self) -> Try<()> {
        Ok(self.connection()?.execute_batch(SCHEMA)?)
    }

    fn connection(&self) -> Result<&Connection, rusqlite::Error> {
        self.connection
            .get_or_try(|| Ok(Connection::open(&self.file_name)?))
    }
}

fn connect(file_name: &str) -> Result<Connection, rusqlite::Error> {
    Connection::open(file_name)
}
