use crate::api::search::SearchResults;
use crate::errors::Try;
use crate::model::{
    Album, AlbumInfo, Artist, ArtistInfo, ExternalId, IdString, LoadedTrack, Track, TrackInfo,
};

#[derive(Eq, PartialEq, Hash, Ord, PartialOrd, Clone)]
pub struct ServiceId(pub String);

pub trait Service: Send + Sync {
    fn id(&self) -> ServiceId;
    fn search(&self, query: &str) -> Try<SearchResults>;
    fn fetch(&self, track_id: &IdString<Track>) -> Try<LoadedTrack>;
    fn track_info(&self, track_id: &IdString<Track>) -> Try<ExternalTrack>;
}

pub struct ExternalTrack {
    pub track_id: ExternalId<Track>,
    pub track_info: TrackInfo,
    pub artist_id: ExternalId<Artist>,
    pub artist_info: ArtistInfo,
    pub album_id: ExternalId<Album>,
    pub album_info: AlbumInfo,
}
