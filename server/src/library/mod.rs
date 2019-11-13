mod database;
mod schema;
mod tables;

pub use database::Library;

use crate::model::{
    AlbumId, AlbumInfo, ArtistId, ArtistInfo, LibraryTrackId, PlaylistId, TrackInfo,
};

use serde_derive::Serialize;

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
