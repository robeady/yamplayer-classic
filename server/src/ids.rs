use crate::errors::{string_err, Erro};
use crate::serde::number_string;
use serde_derive::{Deserialize, Serialize};
use std::str::FromStr;

fn parse_id<T: FromStr>(s: &str, type_name: &str) -> Result<T, Erro> {
    s.parse()
        .map_err(|_| string_err(format!("Invalid {} ID {}", type_name, s)))
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
pub struct PlaylistId(#[serde(with = "number_string")] pub u64);

impl FromStr for PlaylistId {
    type Err = Erro;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(PlaylistId(parse_id(s, "playlist")?))
    }
}
