// Generated by diesel_ext

#![allow(unused)]
#![allow(clippy::all)]

use super::schema::{albums, artists, tracks, playlists, playlist_tracks};


#[derive(Identifiable, Queryable, Insertable)]
#[primary_key(album_id)]
pub struct Album {
    pub album_id: Option<i64>,
    pub title: String,
    pub cover_image_url: Option<String>,
    pub release_date: Option<String>,
}

#[derive(Identifiable, Queryable, Insertable)]
#[primary_key(artist_id)]
pub struct Artist {
    pub artist_id: Option<i64>,
    pub name: String,
    pub image_url: Option<String>,
}

#[derive(Identifiable, Queryable, Insertable)]
#[primary_key(_id)]
pub struct PlaylistTrack {
    pub _id: Option<i64>,
    pub playlist_id: i64,
    pub track_id: i64,
}

#[derive(Identifiable, Queryable, Insertable)]
#[primary_key(playlist_id)]
pub struct Playlist {
    pub playlist_id: Option<i64>,
    pub name: String,
}

#[derive(Identifiable, Queryable, Insertable)]
#[primary_key(track_id)]
pub struct Track {
    pub track_id: Option<i64>,
    pub album_id: i64,
    pub artist_id: i64,
    pub title: String,
    pub isrc: Option<String>,
    pub duration_secs: f32,
    pub file_path: Option<String>,
}
