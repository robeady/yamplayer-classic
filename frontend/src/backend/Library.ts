import { observable, action } from "mobx"
import { Track } from "../Model"
import { ServerApi, ServerEvent } from "./ServerApi"
import { iterate } from "iterare"

export class Library {
    constructor(private serverApi: ServerApi) {
        serverApi.addHandler(this.handleEvent)
        serverApi.request("GetLibrary").then(this.populateLibrary)
    }

    private handleEvent = (e: ServerEvent) => {}

    @observable tracks = new Map<string, Track | null>()

    @observable library: Set<string> | null = null

    getTrack = (trackId: string): Track | null => {
        const t = this.tracks.get(trackId)
        if (t === undefined) {
            this.serverApi.request("GetTrack", { track_id: trackId }).then(t => this.tracks.set(trackId, t))
            return null
        } else {
            return t
        }
    }

    getLibrary = (): Track[] | null => {
        if (this.library === null) {
            return null
        } else {
            // if library is non-null, tracks must contain everything in library
            return iterate(this.library)
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
        this.library = iterate(library.tracks)
            .map(track => track.id)
            .toSet()
    }
}
