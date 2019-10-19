import { observable, action } from "mobx"
import { Track } from "../Model"
import { ServerApi, ServerEvent, SearchResults } from "./ServerApi"
import { iterate } from "iterare"
import { sortBy } from "lodash"

export class Library {
    constructor(private serverApi: ServerApi) {
        serverApi.addHandler(this.handleEvent)
        serverApi.request("GetLibrary").then(this.populateLibrary)
    }

    private handleEvent = (e: ServerEvent) => {
        // TODO: handle events like new playlists and tracks, tracks edited, etc
    }

    @observable private tracks = new Map<string, Track | null>()

    @observable private libraryTrackIds: Set<string> | null = null

    @observable private playlists: Array<{ id: string; name: string }> | null = null

    @observable private playlistTrackIds = new Map<string, string[]>()

    getTrack = (trackId: string): Track | null => {
        const t = this.tracks.get(trackId)
        if (t === undefined) {
            this.serverApi.request("GetTracks", { track_ids: [trackId] }).then(this.populateTracks)
            return null
        } else {
            return t
        }
    }

    @action
    private populateTracks = (response: Record<string, Track | null>) => {
        Object.entries(response).forEach(([id, track]) => this.tracks.set(id, track))
    }

    getLibrary = (): Track[] | null => {
        if (this.libraryTrackIds === null) {
            return null
        } else {
            // if library is non-null, every item in the library must be in tracks
            return iterate(this.libraryTrackIds)
                .map(tid => this.tracks.get(tid)!)
                .toArray()
        }
    }

    @action
    private populateLibrary = (library: { tracks: Track[] }) => {
        for (const track of library.tracks) {
            if (!this.tracks.has(track.id)) {
                this.tracks.set(track.id, track)
            }
        }
        this.libraryTrackIds = iterate(library.tracks)
            .map(track => track.id)
            .toSet()
    }

    listPlaylists = () => {
        if (this.playlists === null) {
            this.serverApi.request("ListPlaylists").then(this.populatePlaylists)
            return null
        } else {
            return this.playlists
        }
    }

    @action
    private populatePlaylists = (response: { playlists: { id: string; name: string }[] }) => {
        this.playlists = sortBy(response.playlists, p => p.name)
    }

    listPlaylistTracks = (playlistId: string): Track[] | null => {
        const trackIds = this.playlistTrackIds.get(playlistId)
        if (trackIds === undefined) {
            this.serverApi.request("GetPlaylist", { id: playlistId }).then(playlist => {
                if (playlist === null) {
                    // if the playlist doesn't exist do nothing
                } else {
                    // also get any new tracks from the playlist
                    const unknownTrackIds = playlist.track_ids.filter(tid => !this.tracks.has(tid))
                    this.serverApi
                        .request("GetTracks", { track_ids: unknownTrackIds })
                        .then(newTracksById =>
                            this.populatePlaylistTracks(playlistId, playlist.track_ids, newTracksById),
                        )
                }
            })
            return null
        } else {
            // for now we are discarding any tracks in a playlist that didn't exist
            return iterate(trackIds)
                .map(tid => this.tracks.get(tid))
                .filter<Track>((track): track is Track => !!track)
                .toArray()
        }
    }

    @action
    private populatePlaylistTracks = (
        playlistId: string,
        trackIds: string[],
        newTracksById: Record<string, Track | null>,
    ) => {
        this.playlistTrackIds.set(playlistId, trackIds)
        for (const [tid, track] of Object.entries(newTracksById)) {
            this.tracks.set(tid, track)
        }
    }

    @observable private searchResults = new Map<string, SearchResults | "loading">()

    getSearchResults = (query: string): SearchResults | "loading" => {
        if (this.searchResults.get(query) === undefined) {
            // TODO: don't keep all search results ever in memory.
            // this should be an expring cache or hold only the last search
            this.searchResults.set(query, "loading")
            this.serverApi.request("Search", { query }).then(r => this.searchResults.set(query, r))
        }
        return this.searchResults.get(query)!
    }
}
