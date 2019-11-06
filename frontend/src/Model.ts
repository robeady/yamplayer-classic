// export interface Track {
//     id: string
//     file_path: string
//     title: string
//     artist: string
//     album: string
//     duration_secs: number
// }

export interface Track {
    track_id: string
    // external_ids: Vec<ExternalTrackId>,
    track_info: TrackInfo
    file_path: string | null
    artist_id: string
    artist_info: ArtistInfo
    album_id: string
    album_info: AlbumInfo
}

export interface PlaybackTiming {
    durationSecs: number
    playingSinceTimestamp: number | "paused"
    positionSecsAtTimestamp: number
}

export interface TrackInfo {
    title: string
    isrc: string | null
    duration_secs: number
}

export interface ArtistInfo {
    name: string
    image_url: string
}

export interface AlbumInfo {
    title: string
    cover_image_url: string
    release_date: string | null
}
