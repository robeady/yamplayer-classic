import { observable, action, computed } from "mobx"
import { PlaybackTiming } from "../Model"
import { ServerApi, ServerEvent } from "./ServerApi"

export class Playback {
    constructor(private serverApi: ServerApi) {
        serverApi.addHandler(this.handleEvent)
        this.initialiseState()
    }

    private initialiseState = async () => {
        // TODO initialise all the state
        const initialState = await this.serverApi.request("GetPlaybackState")
        this.setVolume(initialState.volume)
    }

    @action setVolume = (v: number) => (this.volume = v)

    @action
    private handleEvent = (e: ServerEvent) => {
        switch (e.type) {
            case "VolumeChanged":
                this.volume = e.args.volume
                this.muted = e.args.muted
                return
            case "PlaybackChanged":
                this.currentTrack =
                    e.args.current_track === null
                        ? null
                        : {
                              trackId: e.args.current_track.track.id,
                              durationSecs: e.args.current_track.track.duration_secs,
                              playingSinceTimestamp: e.args.paused ? "paused" : performance.now(),
                              positionSecsAtTimestamp: e.args.current_track.position_secs,
                          }
                return
        }
    }

    @observable volume = 0.5
    @observable muted = false

    @observable
    currentTrack: PlaybackTiming & { trackId: string } | null = null

    @computed
    get playing() {
        return this.currentTrack !== null && this.currentTrack.playingSinceTimestamp !== "paused"
    }

    changeVolume = (muted: boolean, volume?: number) => this.serverApi.request("ChangeVolume", { muted, volume })

    pause = () => this.serverApi.request("Pause")

    unpause = () => this.serverApi.request("Unpause")

    skipToNext = () => this.serverApi.request("SkipToNext")

    stop = () => this.serverApi.request("Stop")

    enqueue = (trackId: string) => this.serverApi.request("Enqueue", { track_id: trackId })
}
