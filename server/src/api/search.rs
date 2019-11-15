use crate::ids::{Album, Artist, Entity, ExternalId, LibraryId, Track};
use crate::model::{AlbumInfo, ArtistInfo, TrackInfo};
use serde_derive::Serialize;

#[derive(Serialize)]
pub struct SearchResults {
    pub tracks: Vec<TrackSearchResult>,
    pub albums: Vec<SearchResult<Album, AlbumInfo>>,
    pub artists: Vec<SearchResult<Artist, ArtistInfo>>,
}

#[derive(Serialize)]
pub struct SearchResult<E: Entity, T> {
    pub library_id: Option<LibraryId<E>>,
    pub external_id: ExternalId<E>,
    pub info: T,
}

#[derive(Serialize)]
pub struct TrackSearchResult {
    pub track: SearchResult<Track, TrackInfo>,
    pub artist: SearchResult<Artist, ArtistInfo>,
    pub album: SearchResult<Album, AlbumInfo>,
}
