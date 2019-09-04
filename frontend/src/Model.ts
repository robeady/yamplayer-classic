export interface Track {
    id: string
    file_path: string
    title: string
    artist: string
    album: string
}

export interface PlaybackProgress {
    /** The playback position of the current song in seconds */
    positionSecs: number
    /**
     * If the current song is playing, the timestamp offset (from performance.now)
     * at which the position in seconds was correct. Else if the current song is paused, null.
     */
    timestampOffsetMillis: number | null
}
