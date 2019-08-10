import { observable } from "mobx"
import { Track } from "../Model"
import { ServerApi, ServerEvent } from "./ServerApi"
import { keyBy } from "lodash"

export class Library {
    constructor(private serverApi: ServerApi) {
        serverApi.addHandler(this.handleEvent)
        serverApi.request("GetLibrary").then(({ tracks }) => (this.tracks = keyBy(tracks, t => t.id)))
    }

    private handleEvent = (e: ServerEvent) => {}

    @observable tracks: { [trackId: string]: Track } | null = null
}
