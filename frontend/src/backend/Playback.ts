import { observable } from "mobx"
import { createContext, useContext } from "react"
import { Track } from "../Model"
import { ServerApi, serverApi, ServerEvent } from "./ServerApi"

export class Playback {
    constructor(private serverApi: ServerApi) {
        serverApi.addHandler(this.handleEvent)
        this.initialiseState()
    }

    private initialiseState = async () => {
        const initialState = await this.serverApi.request("GetPlaybackState")
        this.volume = initialState.volume
        this.playing = initialState.playing
    }

    private handleEvent = (e: ServerEvent) => {
        switch (e.type) {
            case "VolumeChanged":
                this.volume = e.args.volume
                this.muted = e.args.muted
                return
            case "PlaybackPaused":
                this.playing = false
                return
            case "PlaybackResumed":
                this.playing = true
                return
            case "TrackChanged":
                this.currentTrack = e.args.track
                return
        }
    }

    @observable volume = 0.5
    @observable muted = false
    @observable playing = false
    @observable currentTrack: Track | null = null

    changeVolume = (muted: boolean, volume?: number) => this.serverApi.request("ChangeVolume", { muted, volume })

    togglePause = () => this.serverApi.request("TogglePause")

    stop = () => this.serverApi.request("Stop")

    enqueue = (trackId: string) => this.serverApi.request("Enqueue", { track_id: trackId })
}

export const playback = new Playback(serverApi)
const playbackContext = createContext(playback)
export const usePlayback = () => useContext(playbackContext)
