mod database;
mod schema;
mod tables;

pub use database::Library;

use crate::ids::{Album, Artist, LibraryId};
use crate::model::{AlbumInfo, ArtistInfo, TrackInfo};

use serde_derive::Serialize;

#[derive(Serialize, Clone)]
pub struct Track {
    pub track_id: LibraryId<crate::ids::Track>,
    // pub external_ids: Vec<ExternalTrackId>,
    pub track_info: TrackInfo,
    pub artist_id: LibraryId<Artist>,
    pub artist_info: ArtistInfo,
    pub album_id: LibraryId<Album>,
    pub album_info: AlbumInfo,
}

#[derive(Serialize, Clone)]
pub struct Playlist {
    pub id: LibraryId<crate::ids::Playlist>,
    pub name: String,
    track_ids: Vec<LibraryId<crate::ids::Track>>,
}

impl Playlist {
    fn tracks(&self) -> impl Iterator<Item = LibraryId<crate::ids::Track>> + '_ {
        self.track_ids.iter().copied()
    }
}
