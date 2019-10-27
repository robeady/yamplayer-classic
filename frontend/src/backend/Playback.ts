import { observable, action, computed } from "mobx"
import { PlaybackTiming } from "../Model"
import { ServerApi, ServerEvent, CurrentTrack } from "./ServerApi"

export class Playback {
    constructor(private serverApi: ServerApi) {
        serverApi.addHandler(this.handleEvent)
        this.initialiseState()
    }

    private updateCurrentTrack(currentTrack: null | CurrentTrack, paused: boolean) {
        this.currentTrack =
            currentTrack === null
                ? null
                : {
                      trackId: currentTrack.track.id,
                      durationSecs: currentTrack.track.duration_secs,
                      playingSinceTimestamp: paused ? "paused" : performance.now(),
                      positionSecsAtTimestamp: currentTrack.position_secs,
                  }
    }

    private initialiseState = async () => {
        // TODO initialise all the state
        const { volume, muted, current_track, paused } = await this.serverApi.request("GetPlaybackState")
        this.volume = volume
        this.muted = muted
        this.updateCurrentTrack(current_track, paused)
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
                this.updateCurrentTrack(e.args.current_track, e.args.paused)
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
