export interface Track {
    id: string
    file_path: string
    title: string
    artist: string
    album: string
}

export interface PlaybackTiming {
    durationSecs: number
    playingSinceTimestamp: number | "paused"
    positionSecsAtTimestamp: number
}
