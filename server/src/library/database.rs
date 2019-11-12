use super::schema::{albums, artists, playlist_tracks, playlists, tracks};
use super::tables;
use crate::api::search::SearchResults;
use crate::errors::Try;
use crate::library::{Library, Playlist, Track};
use crate::model::{
    AlbumId, AlbumInfo, ArtistId, ArtistInfo, LibraryTrackId, PlaylistId, TrackInfo,
};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel::{insert_into, select};
use std::collections::HashMap;
use thread_local::CachedThreadLocal;

pub struct DbLibrary {
    connection: CachedThreadLocal<SqliteConnection>,
    file_path: String,
}

impl Library for DbLibrary {
    fn tracks(&self) -> Try<Box<dyn Iterator<Item = Track>>> {
        let rows: Vec<(tables::Track, tables::Album, tables::Artist)> = tracks::table
            .inner_join(albums::table)
            .inner_join(artists::table)
            .load(self.connection()?)?;
        Ok(Box::new(rows.into_iter().map(into_track)))
    }

    fn get_track(&self, id: LibraryTrackId) -> Try<Option<Track>> {
        let con = establish_connection();
        // let track: Option<tables::Track> = tracks::table.find(id.0).first(&con).optional()?;
        let row: Option<(tables::Track, tables::Album, tables::Artist)> = tracks::table
            .inner_join(albums::table)
            .inner_join(artists::table)
            .filter(tracks::track_id.eq(Some(id.0)))
            .first(&con)
            .optional()?;
        Ok(row.map(into_track))
    }

    // TODO: special error enum including track/playlist/artist/album not found
    fn create_track(
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

    fn create_album(&self, album: AlbumInfo) -> Try<AlbumId> {
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

    fn find_albums(&self, title: &str) -> Try<Vec<(AlbumId, AlbumInfo)>> {
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

    fn create_artist(&self, artist: ArtistInfo) -> Try<ArtistId> {
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

    fn find_artists(&self, name: &str) -> Try<Vec<(ArtistId, ArtistInfo)>> {
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

    fn playlists(&self) -> Try<Box<dyn Iterator<Item = Playlist>>> {
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
        Ok(Box::new(name_and_track_ids_by_playlist_id.into_iter().map(
            |(id, (name, track_ids))| Playlist {
                id,
                name,
                track_ids,
            },
        )))
    }

    fn create_playlist(&mut self, name: String) -> Try<PlaylistId> {
        let c = self.connection()?;
        insert_into(playlists::table)
            .values(tables::Playlist {
                playlist_id: None,
                name,
            })
            .execute(c)?;
        Ok(PlaylistId(last_id(c)?))
    }

    fn get_playlist(&self, id: PlaylistId) -> Try<Option<Playlist>> {
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

    fn add_track_to_playlist(
        &mut self,
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

    fn resolve(&self, mut search_results: SearchResults) -> Try<SearchResults> {
        // TODO: search
        Ok(search_results)
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

fn establish_connection() -> SqliteConnection {
    SqliteConnection::establish("../../database.sqlite").expect("error connecting to db")
}

// const SCHEMA: &str = include_str!("schema.sql");

impl DbLibrary {
    //    pub fn connect_or_create(file_name: String) -> Try<DbLibrary> {
    //        let db = DbLibrary {
    //            connection: CachedThreadLocal::new(),
    //            file_name,
    //        };
    //        db.setup()?;
    //        Ok(db)
    //    }
    //
    //    fn setup(&self) -> Try<()> {
    //        Ok(self.connection()?.execute_batch(SCHEMA)?)
    //    }
    //
    //    fn connection(&self) -> Result<&Connection, rusqlite::Error> {
    //        self.connection
    //            .get_or_try(|| Ok(Connection::open(&self.file_name)?))
    //    }

    fn connection(&self) -> diesel::ConnectionResult<&SqliteConnection> {
        // TODO: create if not open yet?
        // TODO: enforce foreign key constraints
        self.connection
            .get_or_try(|| SqliteConnection::establish(&self.file_path))
    }
}

//fn connect(file_name: &str) -> Result<Connection, rusqlite::Error> {
//    Connection::open(file_name)
//}

no_arg_sql_function!(last_insert_rowid, diesel::sql_types::BigInt);

/// returns the rowid of the last row inserted by this database connection
fn last_id(con: &SqliteConnection) -> diesel::result::QueryResult<i64> {
    select(last_insert_rowid).first(con)
}

#[cfg(test)]
mod tests {
    use crate::library::{DbLibrary, Library};
    use thread_local::CachedThreadLocal;

    #[test]
    fn foo() {
        println!("wd: {}", std::env::current_dir().unwrap().to_str().unwrap());
        println!("hello world");
        let mut d = DbLibrary {
            connection: CachedThreadLocal::new(),
            file_path: "cxz ".to_string(),
        };
        println!("id was {}", d.create_playlist("foo".to_string()).unwrap().0);
        //        println!(
        //            "{:?}",
        //            diesel::debug_query::<Sqlite, _>(&insert_into(playlists::table).values(
        //                tables::Playlist {
        //                    playlist_id: None,
        //                    name: "steve".to_string(),
        //                }
        //            ))
        //        );
        assert!(false)
    }
}
