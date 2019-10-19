use crate::errors::{Erro, Try};
use crate::serde::number_string;
use anyhow::anyhow;
use chrono::NaiveDate;
use fstrings::{f, format_args_f};
use serde_derive::{Deserialize, Serialize};
use std::str::FromStr;
use url::Url;

fn parse_id<T: FromStr>(s: &str, type_name: &str) -> Try<T> {
    s.parse()
        .map_err(|_| anyhow!(f!("Invalid {type_name} ID {s}")))
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
pub struct TrackId(#[serde(with = "number_string")] pub u64);

impl FromStr for TrackId {
    type Err = Erro;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(TrackId(parse_id(s, "track")?))
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
pub struct AlbumId(#[serde(with = "number_string")] pub u64);

impl FromStr for AlbumId {
    type Err = Erro;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(AlbumId(parse_id(s, "album")?))
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
pub struct ArtistId(#[serde(with = "number_string")] pub u32);

impl FromStr for ArtistId {
    type Err = Erro;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ArtistId(parse_id(s, "artist")?))
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
pub struct PlaylistId(#[serde(with = "number_string")] pub u64);

impl FromStr for PlaylistId {
    type Err = Erro;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(PlaylistId(parse_id(s, "playlist")?))
    }
}

#[derive(Serialize)]
pub struct TrackInfo {
    pub title: String,
    pub isrc: Option<String>,
    pub duration_secs: f32,
}

#[derive(Serialize)]
pub struct ArtistInfo {
    pub name: String,
    pub image_url: Url,
}

#[derive(Serialize)]
pub struct AlbumInfo {
    pub title: String,
    pub cover_image_url: Url,
    pub release_date: Option<NaiveDate>,
}
