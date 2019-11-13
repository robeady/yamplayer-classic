use super::schema::{albums, artists, playlist_tracks, playlists, tracks};
use super::tables;
use crate::api::search::SearchResults;
use crate::api::EventSink;
use crate::errors::Try;
use crate::library::{Playlist, Track};
use crate::model::{
    AlbumId, AlbumInfo, ArtistId, ArtistInfo, LibraryTrackId, PlaylistId, TrackInfo,
};
use anyhow::anyhow;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel::{insert_into, select};
use fstrings::{f, format_args_f};
use id3::Tag;
use log;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
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
            self.read_mp3(file_path)?
        } else if file_path.ends_with(".flac") {
            self.read_flac(file_path)?
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

    fn read_mp3(&self, file_path: String) -> Try<(TrackInfo, AlbumInfo, ArtistInfo)> {
        // TODO: move mp3 handling code to separate module
        let mp3_tags = Tag::read_from_path(&file_path)?;
        let tag = |name: &str, value: Option<&str>| {
            value.map(|t| t.to_string()).unwrap_or_else(|| {
                log::warn!("no {} tag in {}", name, file_path);
                f!("UNKNOWN {name}")
            })
        };
        let mut mp3 = minimp3::Decoder::new(BufReader::new(File::open(&file_path)?));
        let mut duration_secs = 0_f32;
        loop {
            let frame_result = mp3.next_frame();
            let frame = match frame_result {
                Ok(frame) => frame,
                // An error caused by some IO operation required during decoding.
                Err(minimp3::Error::Io(e)) => return Err(e.into()),
                // The decoder tried to parse a frame from its internal buffer, but there was not enough.
                Err(minimp3::Error::InsufficientData) => {
                    panic!("not enough data in encoder buffer??? {}", file_path)
                }
                // The decoder encountered data which was not a frame (ie, ID3 data), and skipped it.
                Err(minimp3::Error::SkippedData) => continue,
                // The decoder has reached the end of the provided reader.
                Err(minimp3::Error::Eof) => break,
            };
            let seconds_of_audio =
                (frame.data.len() / frame.channels) as f32 / frame.sample_rate as f32;
            duration_secs += seconds_of_audio;
        }
        let track_title = tag("TITLE", mp3_tags.title());
        let album_title = tag("ALBUM", mp3_tags.album());
        let artist_name = tag("ARTIST", mp3_tags.artist());
        Ok((
            TrackInfo {
                title: track_title,
                isrc: None,
                duration_secs,
                file_path: Some(file_path),
            },
            AlbumInfo {
                title: album_title,
                cover_image_url: None,
                release_date: None,
            },
            ArtistInfo {
                name: artist_name,
                image_url: None,
            },
        ))
    }

    fn read_flac(&self, file_path: String) -> Try<(TrackInfo, AlbumInfo, ArtistInfo)> {
        let flac = claxon::FlacReader::open(&file_path)?;
        // TODO: move flac handling code to separate module
        let tag = |name: &str| {
            flac.get_tag(name)
                .next()
                .map(|t| t.to_string())
                .unwrap_or_else(|| {
                    log::warn!("no {} tag in {}", name, file_path);
                    f!("UNKNOWN {name}")
                })
        };
        let duration_secs = (flac
            .streaminfo()
            .samples
            .unwrap_or_else(|| panic!("no stream info in {}", file_path))
            as f32)
            / flac.streaminfo().sample_rate as f32;
        let track_title = tag("TITLE");
        let album_title = tag("ALBUM");
        let artist_name = tag("ARTIST");
        Ok((
            TrackInfo {
                title: track_title,
                isrc: None,
                duration_secs,
                file_path: Some(file_path),
            },
            AlbumInfo {
                title: album_title,
                cover_image_url: None,
                release_date: None,
            },
            ArtistInfo {
                name: artist_name,
                image_url: None,
            },
        ))
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

/// returns the rowid of the last row inserted by this database connection
fn last_id(con: &SqliteConnection) -> diesel::result::QueryResult<i64> {
    select(last_insert_rowid).first(con)
}

#[cfg(test)]
mod tests {
    use crate::library::{Library, Library};
    use thread_local::CachedThreadLocal;

    #[test]
    fn foo() {
        println!("wd: {}", std::env::current_dir().unwrap().to_str().unwrap());
        println!("hello world");
        let mut d = Library {
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
