mod database;
mod inmemory;
mod schema;
mod tables;

pub use database::DbLibrary;
pub use inmemory::InMemoryLibrary;

use crate::api::search::SearchResults;
use crate::errors::Try;
use crate::model::{
    AlbumId, AlbumInfo, ArtistId, ArtistInfo, LibraryTrackId, PlaylistId, TrackInfo,
};
use anyhow::anyhow;
use fstrings::{f, format_args_f};
use id3::Tag;
use serde_derive::Serialize;
use std::fs::File;
use std::io::BufReader;

pub trait Library: Send + Sync + 'static {
    fn tracks(&self) -> Try<Box<dyn Iterator<Item = Track> + '_>>;

    fn get_track(&self, id: LibraryTrackId) -> Try<Option<Track>>;

    fn create_track(
        &self,
        track: TrackInfo,
        album: AlbumId,
        artist: ArtistId,
    ) -> Try<LibraryTrackId>;

    fn create_album(&self, album: AlbumInfo) -> Try<AlbumId>;

    fn find_albums(&self, title: &str) -> Try<Vec<(AlbumId, AlbumInfo)>>;

    fn create_artist(&self, artist: ArtistInfo) -> Try<ArtistId>;

    fn find_artists(&self, name: &str) -> Try<Vec<(ArtistId, ArtistInfo)>>;

    fn playlists(&self) -> Try<Box<dyn Iterator<Item = Playlist> + '_>>;

    fn create_playlist(&mut self, name: String) -> Try<PlaylistId>;

    fn get_playlist(&self, id: PlaylistId) -> Try<Option<Playlist>>;

    fn add_track_to_playlist(
        &mut self,
        track_id: LibraryTrackId,
        playlist_id: PlaylistId,
    ) -> Try<()>;

    fn resolve(&self, search_results: SearchResults) -> Try<SearchResults>;

    fn add_local_track(&self, file_path: String) -> Try<LibraryTrackId> {
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

#[derive(Serialize, Clone)]
pub struct Track {
    pub track_id: LibraryTrackId,
    // pub external_ids: Vec<ExternalTrackId>,
    pub track_info: TrackInfo,
    pub artist_id: ArtistId,
    pub artist_info: ArtistInfo,
    pub album_id: AlbumId,
    pub album_info: AlbumInfo,
}

//#[derive(Debug, Serialize, Deserialize)]
//pub struct Track {
//    pub file_path: String,
//    pub title: String,
//    pub artist: String,
//    pub album: String,
//    pub duration_secs: f32,
//    pub external_id: Option<String>,
//}

#[derive(Serialize, Clone)]
pub struct Playlist {
    pub id: PlaylistId,
    pub name: String,
    track_ids: Vec<LibraryTrackId>,
}

impl Playlist {
    fn tracks(&self) -> impl Iterator<Item = LibraryTrackId> + '_ {
        self.track_ids.iter().copied()
    }
}
