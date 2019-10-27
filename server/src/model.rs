use crate::errors::{Erro, Try};
use crate::queue::CurrentTrack;
use crate::serde::string;
use crate::server::ServiceId;
use crate::{deserialize_with_parse, serialize_with_display};
use anyhow::anyhow;
use chrono::NaiveDate;
use fstrings::{f, format_args_f};
use serde::export::fmt::Error;
use serde::export::Formatter;
use serde_derive::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;
use url::Url;

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone)]
pub enum TrackId {
    Library(LibraryTrackId),
    External(ExternalTrackId),
}

serialize_with_display!(TrackId);
deserialize_with_parse!(TrackId);

impl FromStr for TrackId {
    type Err = Erro;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.splitn(2, ':').collect();
        match parts.as_slice() {
            [p] => Ok(Self::Library(LibraryTrackId(p.parse()?))),
            [service, id] => Ok(Self::External(ExternalTrackId {
                service_id: ServiceId((*service).to_owned()),
                track_id: (*id).to_owned(),
            })),
            _ => Err(anyhow!("invalid track ID {}", s)),
        }
    }
}

impl Display for TrackId {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            TrackId::Library(library_id) => library_id.0.fmt(f),
            TrackId::External(ExternalTrackId {
                service_id,
                track_id,
            }) => write!(f, "{}:{}", service_id.0, track_id),
        }
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
pub struct LibraryTrackId(#[serde(with = "string")] pub u64);

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct ExternalTrackId {
    pub service_id: ServiceId,
    pub track_id: String,
}

//#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
//pub struct TrackId(#[serde(with = "string")] pub u64);
//
//impl FromStr for TrackId {
//    type Err = Erro;
//
//    fn from_str(s: &str) -> Result<Self, Self::Err> {
//        Ok(TrackId(parse_id(s, "track")?))
//    }
//}

fn parse_id<T: FromStr>(s: &str, type_name: &str) -> Try<T> {
    s.parse()
        .map_err(|_| anyhow!(f!("Invalid {type_name} ID {s}")))
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
pub struct AlbumId(#[serde(with = "string")] pub u64);

impl FromStr for AlbumId {
    type Err = Erro;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(AlbumId(parse_id(s, "album")?))
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
pub struct ArtistId(#[serde(with = "string")] pub u32);

impl FromStr for ArtistId {
    type Err = Erro;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ArtistId(parse_id(s, "artist")?))
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
pub struct PlaylistId(#[serde(with = "string")] pub u64);

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

pub struct LoadedTrack {
    pub data: Vec<u8>,
    pub duration_secs: f32,
}

#[derive(Serialize)]
pub struct PlaybackState {
    pub muted: bool,
    pub volume: f32,
    pub paused: bool,
    pub current_track: Option<CurrentTrack>,
}
