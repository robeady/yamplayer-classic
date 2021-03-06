import { RPCWebSocket, Payload } from "../websocket"
import { Track, AlbumInfo, ArtistInfo, TrackInfo } from "../Model"

interface ServerRPCApi {
    (type: "Enqueue", args: { track_id: string }): Promise<void>
    (type: "Stop"): Promise<void>
    (type: "Pause"): Promise<void>
    (type: "Unpause"): Promise<void>
    (type: "SkipToNext"): Promise<void>
    (type: "ChangeVolume", args: { muted?: boolean; volume?: number }): Promise<void>
    (type: "CompleteFilePath", args: { prefix: string }): Promise<void>
    (type: "GetTracks", args: { track_ids: string[] }): Promise<Record<string, Track | null>>
    (type: "GetLibrary"): Promise<{ tracks: Track[] }>
    (type: "AddToLibrary", args: { path: string }): Promise<void>
    (type: "GetPlaybackState"): Promise<PlaybackState>
    (type: "ListPlaylists"): Promise<{ playlists: { id: string; name: string }[] }>
    (type: "GetPlaylist", args: { id: string }): Promise<{ name: string; track_ids: string[] } | null>
    (type: "AddTrackToPlaylist", args: { track_id: string; playlist_id: string }): Promise<void>
    (type: "Search", args: { query: string }): Promise<SearchResults>
}

export interface PlaybackState {
    muted: boolean
    volume: number
    paused: boolean
    current_track: CurrentTrack | null
}

export interface SearchResults {
    tracks: TrackSearchResult[]
    albums: SearchResult<AlbumInfo>[]
    artists: SearchResult<ArtistInfo>[]
}

export interface SearchResult<T> {
    library_id: string
    external_id: string
    info: T
}

export interface TrackSearchResult {
    track: SearchResult<TrackInfo>
    album: SearchResult<AlbumInfo>
    artist: SearchResult<ArtistInfo>
}

type ServerEventHandler = (e: ServerEvent) => void

export interface EnqueuedTrack {
    id: string
    duration_secs: number
    entry_marker: string
}

export interface CurrentTrack {
    track: EnqueuedTrack
    position_secs: number
}

export type ServerEvent =
    | { type: "VolumeChanged"; args: { muted: boolean; volume: number } }
    | { type: "PlaybackChanged"; args: { paused: boolean; current_track: CurrentTrack | null } }
    | { type: "TrackAddedToLibrary"; args: Track }
    | { type: "TrackAddedToPlaylist"; args: { track_id: string; playlist_id: string } }

export class ServerApi {
    private handleEvent = (payload: Payload) => {
        this.handlers.forEach(handle => handle(payload as ServerEvent))
    }

    private ws = new RPCWebSocket("ws://127.0.0.1:8080/ws", this.handleEvent)
    private handlers: ServerEventHandler[] = []

    addHandler = (handler: ServerEventHandler) => {
        this.handlers.push(handler)
    }

    request: ServerRPCApi = async (type: string, args?: {}) => {
        return (await this.ws.query({ type, args })) as any
    }
}
