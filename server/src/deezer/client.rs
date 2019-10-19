use super::types::{DeezerList, DeezerSearchedTrack};
use crate::api::search::{SearchResult, SearchResults, TrackSearchResult};
use crate::deezer::types::{DeezerSearchedAlbum, DeezerSearchedArtist};
use crate::errors::Try;
use crate::model::{AlbumId, AlbumInfo, ArtistId, ArtistInfo, TrackInfo};
use anyhow::Context;
use fstrings::{f, format_args_f};
use reqwest::Client;
use serde::de::DeserializeOwned;

const BASE_DEEZER_API: &str = "https://api.deezer.com/";

pub struct DeezerClient {
    client: Client,
}

impl DeezerClient {
    pub fn new() -> Self {
        DeezerClient {
            client: Client::new(),
        }
    }

    pub fn search(&self, search_query: &str) -> Try<SearchResults> {
        // TODO: call in parallel
        Ok(SearchResults {
            tracks: self.search_tracks(search_query)?,
            albums: self.search_albums(search_query)?,
            artists: self.search_artists(search_query)?,
        })
    }

    pub fn search_tracks(&self, search_query: &str) -> Try<Vec<TrackSearchResult>> {
        self.fetch::<DeezerList<DeezerSearchedTrack>>(&f!("search/track?q={search_query}"))?
            .data
            .into_iter()
            .map(|t| {
                Ok(TrackSearchResult {
                    track: SearchResult {
                        library_id: None,
                        external_id: t.id.to_string(),
                        info: TrackInfo {
                            title: t.title,
                            isrc: None,
                            duration_secs: t.duration as f32,
                        },
                    },
                    artist: SearchResult {
                        library_id: None,
                        external_id: t.artist.id.to_string(),
                        info: ArtistInfo {
                            name: t.artist.name,
                            image_url: t.artist.picture_medium,
                        },
                    },
                    album: SearchResult {
                        library_id: None,
                        external_id: t.album.id.to_string(),
                        info: AlbumInfo {
                            title: t.album.title,
                            cover_image_url: t.album.cover_medium,
                            release_date: None,
                        },
                    },
                })
            })
            .collect()
    }

    pub fn search_albums(&self, search_query: &str) -> Try<Vec<SearchResult<AlbumId, AlbumInfo>>> {
        self.fetch::<DeezerList<DeezerSearchedAlbum>>(&f!("search/album?q={search_query}"))?
            .data
            .into_iter()
            .map(|a| {
                Ok(SearchResult {
                    library_id: None,
                    external_id: a.id.to_string(),
                    info: AlbumInfo {
                        title: a.title,
                        cover_image_url: a.cover_medium,
                        release_date: None,
                    },
                })
            })
            .collect()
    }

    pub fn search_artists(
        &self,
        search_query: &str,
    ) -> Try<Vec<SearchResult<ArtistId, ArtistInfo>>> {
        self.fetch::<DeezerList<DeezerSearchedArtist>>(&f!("search/artist?q={search_query}"))?
            .data
            .into_iter()
            .map(|a| {
                Ok(SearchResult {
                    library_id: None,
                    external_id: a.id.to_string(),
                    info: ArtistInfo {
                        name: a.name,
                        image_url: a.picture_medium,
                    },
                })
            })
            .collect()
    }

    fn fetch<T: DeserializeOwned>(&self, path: &str) -> Try<T> {
        let response_text = self
            .client
            .get(&f!("{BASE_DEEZER_API}/{path}"))
            .send()?
            .text()?;
        serde_json::from_str::<T>(&response_text).context(response_text)
    }
}
