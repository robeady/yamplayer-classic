use crate::errors::Try;
use crate::queue::CurrentTrack;
use crate::serde::string;
use crate::services::ServiceId;
use crate::{deserialize_with_parse, serialize_with_display};
use chrono::NaiveDate;
use fstrings::{f, format_args_f};
use serde::export::fmt::Error;
use serde::export::{Formatter, PhantomData};
use serde_derive::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;
use url::Url;

pub trait IdType {}

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Copy)]
pub struct Track {}
#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Copy)]
pub struct Album {}
#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Copy)]
pub struct Artist {}
#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Copy)]
pub struct Playlist {}

impl IdType for Track {}
impl IdType for Album {}
impl IdType for Artist {}
impl IdType for Playlist {}

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone)]
pub enum Id<T: IdType> {
    Library(LibraryId<T>),
    External(ExternalId<T>),
}

impl<T: IdType> serde::Serialize for Id<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        crate::serde::string::serialize(self, serializer)
    }
}
impl<'de, T: IdType> serde::Deserialize<'de> for Id<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        crate::serde::string::deserialize(deserializer)
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
pub struct LibraryId<T: IdType>(pub i64, PhantomData<T>);

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct ExternalId<T: IdType> {
    pub service: ServiceId,
    pub id: IdString<T>,
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct IdString<T: IdType>(pub Box<str>, PhantomData<T>);

impl<T: IdType> FromStr for Id<T> {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.splitn(2, ':').collect();
        match parts.as_slice() {
            [p] => Ok(Self::Library(LibraryId(p.parse()?, PhantomData))),
            [service, id] => Ok(Self::External(ExternalId {
                service: ServiceId((*service).to_owned()),
                id: IdString(Box::from(*id), PhantomData),
            })),
            _ => Err(anyhow!("invalid ID {}", s)),
        }
    }
}

impl<T: IdType> Display for Id<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Self::Library(library_id) => library_id.0.fmt(f),
            Self::External(ExternalId { service, id, .. }) => write!(f, "{}:{}", service.0, id.0),
        }
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone)]
pub enum TrackId {
    Library(LibraryTrackId),
    External(ExternalTrackId),
}

serialize_with_display!(TrackId);
deserialize_with_parse!(TrackId);

impl FromStr for TrackId {
    type Err = anyhow::Error;

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
pub struct LibraryTrackId(#[serde(with = "string")] pub i64);

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct ExternalTrackId {
    pub service_id: ServiceId,
    pub track_id: String,
}

fn parse_id<T: FromStr>(s: &str, type_name: &str) -> Try<T> {
    s.parse()
        .map_err(|_| anyhow!(f!("Invalid {type_name} ID {s}")))
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
pub struct AlbumId(#[serde(with = "string")] pub i64);

impl FromStr for AlbumId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(AlbumId(parse_id(s, "album")?))
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
pub struct ArtistId(#[serde(with = "string")] pub i64);

impl FromStr for ArtistId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ArtistId(parse_id(s, "artist")?))
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize, Hash)]
pub struct PlaylistId(#[serde(with = "string")] pub i64);

impl FromStr for PlaylistId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(PlaylistId(parse_id(s, "playlist")?))
    }
}

#[derive(Serialize, Clone)]
pub struct TrackInfo {
    pub title: String,
    pub isrc: Option<String>,
    pub duration_secs: f32,
    pub file_path: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct ArtistInfo {
    pub name: String,
    pub image_url: Option<Url>,
}

#[derive(Serialize, Clone)]
pub struct AlbumInfo {
    pub title: String,
    pub cover_image_url: Option<Url>,
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
