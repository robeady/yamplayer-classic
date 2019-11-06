mod database;
mod inmemory;

pub use database::DbLibrary;
pub use inmemory::InMemoryLibrary;

use crate::api::search::SearchResults;
use crate::errors::Try;
use crate::model::{
    AlbumId, AlbumInfo, ArtistId, ArtistInfo, ExternalTrackId, LibraryTrackId, PlaylistId, TrackId,
    TrackInfo,
};
use id3::Tag;
use serde_derive::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;

pub trait Library: Send + Sync + 'static {
    type Error: Send + Sync + Error;

    fn create_track(
        &self,
        track: TrackInfo,
        artist: ArtistId,
        album: AlbumId,
    ) -> Result<LibraryTrackId, Self::Error>;

    fn create_album(&self, album: AlbumInfo, artist: ArtistId) -> Result<AlbumId, Self::Error>;

    fn create_artist(&self, artist: ArtistInfo) -> Result<ArtistId, Self::Error>;

    fn tracks(&self) -> Result<Box<dyn Iterator<Item = Track> + '_>, Self::Error>;

    fn get_track(&self, id: &LibraryTrackId) -> Result<Option<Track>, Self::Error>;

    fn create_playlist(&mut self, name: String) -> Result<PlaylistId, Self::Error>;

    fn get_playlist(&self, id: PlaylistId) -> Result<Option<Playlist>, Self::Error>;

    fn playlists(&self) -> Result<Box<dyn Iterator<Item = Playlist> + '_>, Self::Error>;

    fn add_track_to_playlist(
        &mut self,
        track_id: LibraryTrackId,
        playlist_id: PlaylistId,
    ) -> Result<bool, Self::Error>;

    fn resolve(&self, search_results: SearchResults) -> Result<SearchResults, Self::Error>;

    fn add_local_track(&self, file_path: String) -> Try<LibraryTrackId> {
        let (track, album, artist) = if file_path.ends_with(".mp3") {
            self.read_mp3(file_path)
        } else if file_path.ends_with(".flac") {
            self.read_flac(file_path)
        } else {
            return Err(anyhow!(f!("unsupported file type {file_path}")));
        };
        Ok(
            if let Some((album_id, _, artist_id)) = self.find_album(&album.title)? {
                self.create_track(track, artist_id, album_id)?
            } else if let Some((artist_id, _)) = self.find_artist(&artist.name)? {
                let album_id = self.create_album(album, artist_id)?;
                self.create_track(track, artist_id, album_id)?
            } else {
                let artist_id = self.create_artist(artist)?;
                let album_id = self.create_album(album, artist_id)?;
                self.create_track(track, artist_id, album_id)?
            },
        )
    }

    fn read_mp3(&self, file_path: String) -> (TrackInfo, AlbumInfo, ArtistInfo) {
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
        (
            TrackInfo {
                title: tag("TITLE", mp3_tags.title()),
                isrc: None,
                duration_secs,
                file_path: Some(file_path),
            },
            AlbumInfo {
                title: tag("ALBUM", mp3_tags.album()),
                cover_image_url: None,
                release_date: None,
            },
            ArtistInfo {
                name: tag("ARTIST", mp3_tags.artist()),
                image_url: None,
            },
        )
    }

    fn read_flac(&self, file_path: String) -> (TrackInfo, AlbumInfo, ArtistInfo) {
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
        (
            TrackInfo {
                title: tag("TITLE"),
                isrc: None,
                duration_secs,
                file_path: Some(file_path),
            },
            AlbumInfo {
                title: tag("ALBUM"),
                cover_image_url: None,
                release_date: None,
            },
            ArtistInfo {
                name: tag("ARTIST"),
                image_url: None,
            },
        )
    }

    fn find_album(
        &self,
        title: &str,
    ) -> Result<Option<(AlbumId, AlbumInfo, ArtistId)>, Self::Error>;

    fn find_artist(&self, name: &str) -> Result<Option<(ArtistId, ArtistInfo)>, Self::Error>;
}

#[derive(Serialize, Clone)]
pub struct Track {
    pub track_id: LibraryTrackId,
    // pub external_ids: Vec<ExternalTrackId>,
    pub track_info: TrackInfo,
    pub file_path: Option<String>,
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
