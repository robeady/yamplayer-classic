table! {
    albums (album_id) {
        album_id -> Nullable<BigInt>,
        title -> Text,
        cover_image_url -> Nullable<Text>,
        release_date -> Nullable<Text>,
    }
}

table! {
    artists (artist_id) {
        artist_id -> Nullable<BigInt>,
        name -> Text,
        image_url -> Nullable<Text>,
    }
}

table! {
    external_albums (_id) {
        _id -> Nullable<BigInt>,
        album_id -> BigInt,
        service_id -> Text,
        external_id -> Text,
    }
}

table! {
    external_artists (_id) {
        _id -> Nullable<BigInt>,
        artist_id -> BigInt,
        service_id -> Text,
        external_id -> Text,
    }
}

table! {
    external_tracks (_id) {
        _id -> Nullable<BigInt>,
        track_id -> BigInt,
        service_id -> Text,
        external_id -> Text,
    }
}

table! {
    playlist_tracks (_id) {
        _id -> Nullable<BigInt>,
        playlist_id -> BigInt,
        track_id -> BigInt,
    }
}

table! {
    playlists (playlist_id) {
        playlist_id -> Nullable<BigInt>,
        name -> Text,
    }
}

table! {
    tracks (track_id) {
        track_id -> Nullable<BigInt>,
        album_id -> BigInt,
        artist_id -> BigInt,
        title -> Text,
        isrc -> Nullable<Text>,
        duration_secs -> Float,
        file_path -> Nullable<Text>,
    }
}

joinable!(external_albums -> albums (album_id));
joinable!(external_artists -> artists (artist_id));
joinable!(external_tracks -> tracks (track_id));
joinable!(playlist_tracks -> playlists (playlist_id));
joinable!(playlist_tracks -> tracks (track_id));
joinable!(tracks -> albums (album_id));
joinable!(tracks -> artists (artist_id));

allow_tables_to_appear_in_same_query!(
    albums,
    artists,
    external_albums,
    external_artists,
    external_tracks,
    playlist_tracks,
    playlists,
    tracks,
);
