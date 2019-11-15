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
