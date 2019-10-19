use serde_derive::Deserialize;
use url::Url;

#[derive(Deserialize)]
pub(super) struct DeezerList<T> {
    pub data: Vec<T>,
}

#[derive(Deserialize)]
pub(super) struct DeezerSearchedTrack {
    /// The track's Deezer id
    pub id: i64,
    /// true if the track is readable in the player for the current user
    pub readable: bool,
    /// The track's fulltitle
    pub title: String,
    /// The track's short title
    pub title_short: String,
    /// The track version, seemingly not always present
    // pub title_version: String,
    /// The url of the track on Deezer
    pub link: Url,
    /// The track's duration in seconds
    pub duration: i64,
    /// The track's Deezer rank
    pub rank: i64,
    /// Whether the track contains explicit lyrics
    pub explicit_lyrics: bool,
    /// The url of track's preview file. This file contains the first 30 seconds of the track
    pub preview: Url,
    /// artist object containing : id, name, link, picture, picture_small, picture_medium, picture_big, picture_xl
    pub artist: DeezerLinkedArtist,
    /// album object containing : id, title, cover, cover_small, cover_medium, cover_big, cover_xl
    pub album: DeezerLinkedAlbum,
}

#[derive(Deserialize)]
pub(super) struct DeezerLinkedArtist {
    /// The artist's Deezer id
    pub id: i64,
    /// The artist's name
    pub name: String,
    /// The url of the artist on Deezer
    pub link: Url,
    /// The url of the artist picture. Add 'size' parameter to the url to change size. Can be 'small', 'medium', 'big', 'xl'
    pub picture: Url,
    /// The url of the artist picture in size small.
    pub picture_small: Url,
    /// The url of the artist picture in size medium.
    pub picture_medium: Url,
    /// The url of the artist picture in size big.
    pub picture_big: Url,
    /// The url of the artist picture in size xl.
    pub picture_xl: Url,
}

#[derive(Deserialize)]
pub(super) struct DeezerLinkedAlbum {
    /// The Deezer album id
    pub id: i64,
    /// The album title
    pub title: String,
    /// The url of the album's cover. Add 'size' parameter to the url to change size. Can be 'small', 'medium', 'big', 'xl'
    pub cover: Url,
    /// The url of the album's cover in size small.
    pub cover_small: Url,
    /// The url of the album's cover in size medium.
    pub cover_medium: Url,
    /// The url of the album's cover in size big.
    pub cover_big: Url,
    /// The url of the album's cover in size xl.
    pub cover_xl: Url,
}

#[derive(Deserialize)]
pub(super) struct DeezerSearchedArtist {
    /// The artist's Deezer id
    pub id: i64,
    /// The artist's name
    pub name: String,
    /// The url of the artist on Deezer
    pub link: Url,
    /// The url of the artist picture. Add 'size' parameter to the url to change size. Can be 'small', 'medium', 'big', 'xl'
    pub picture: Url,
    /// The url of the artist picture in size small.
    pub picture_small: Url,
    /// The url of the artist picture in size medium.
    pub picture_medium: Url,
    /// The url of the artist picture in size big.
    pub picture_big: Url,
    /// The url of the artist picture in size xl.
    pub picture_xl: Url,
    /// The number of artist's albums
    pub nb_album: i64,
    /// The number of artist's fans
    pub nb_fan: i64,
    /// true if the artist has a smartradio
    pub radio: bool,
}

#[derive(Deserialize)]
pub(super) struct DeezerSearchedAlbum {
    /// The Deezer album id
    pub id: i64,
    /// The album title
    pub title: String,
    /// The url of the album on Deezer
    pub link: Url,
    /// The url of the album's cover. Add 'size' parameter to the url to change size. Can be 'small', 'medium', 'big', 'xl'
    pub cover: Url,
    /// The url of the album's cover in size small.
    pub cover_small: Url,
    /// The url of the album's cover in size medium.
    pub cover_medium: Url,
    /// The url of the album's cover in size big.
    pub cover_big: Url,
    /// The url of the album's cover in size xl.
    pub cover_xl: Url,
    /// The album's first genre id (You should use the genre list instead). NB : -1 for not found
    pub genre_id: i64,
    /// ?
    pub nb_tracks: i64,
    /// The record type of the album (EP / ALBUM / etc..)
    pub record_type: String,
    /// Whether the album contains explicit lyrics
    pub explicit_lyrics: bool,
    /// artist object containing : id, name, link, picture, picture_small, picture_medium, picture_big, picture_xl
    pub artist: DeezerLinkedArtist,
}
