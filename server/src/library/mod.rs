mod database;
mod schema;
mod tables;

pub use database::Library;

use crate::ids::{Album, Artist, ExternalId, LibraryId, Track};
use crate::model::{AlbumInfo, ArtistInfo, TrackInfo};

use serde_derive::Serialize;

#[derive(Serialize, Clone)]
pub struct TrackSummary {
    pub track_id: LibraryId<Track>,
    pub external_ids: Vec<ExternalId<Track>>,
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
    track_ids: Vec<LibraryId<Track>>,
}

impl Playlist {
    fn tracks(&self) -> impl Iterator<Item = LibraryId<Track>> + '_ {
        self.track_ids.iter().copied()
    }
}
