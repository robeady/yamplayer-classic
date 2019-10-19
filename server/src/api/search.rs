use crate::model::{AlbumId, AlbumInfo, ArtistId, ArtistInfo, TrackId, TrackInfo};
use serde_derive::Serialize;

#[derive(Serialize)]
pub struct SearchResults {
    pub tracks: Vec<TrackSearchResult>,
    pub albums: Vec<SearchResult<AlbumId, AlbumInfo>>,
    pub artists: Vec<SearchResult<ArtistId, ArtistInfo>>,
}

#[derive(Serialize)]
pub struct SearchResult<I, T> {
    pub library_id: Option<I>,
    pub external_id: String,
    pub info: T,
}

#[derive(Serialize)]
pub struct TrackSearchResult {
    pub track: SearchResult<TrackId, TrackInfo>,
    pub artist: SearchResult<ArtistId, ArtistInfo>,
    pub album: SearchResult<AlbumId, AlbumInfo>,
}
