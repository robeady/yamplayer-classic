import { observable } from "mobx"
import { createContext, useContext } from "react"
import { Track } from "../Model"
import { ServerApi, serverApi, ServerEvent } from "./ServerApi"
import { keyBy } from "lodash"

export class Library {
    constructor(private serverApi: ServerApi) {
        serverApi.addHandler(this.handleEvent)
        serverApi.request("GetLibrary").then(({ tracks }) => (this.tracks = keyBy(tracks, t => t.id)))
    }

    private handleEvent = (e: ServerEvent) => {}

    @observable tracks: { [trackId: string]: Track } | null = null
}

export const library = new Library(serverApi)
const libraryContext = createContext(library)
export const useLibrary = () => useContext(libraryContext)
