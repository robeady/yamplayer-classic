import { observable, action } from "mobx"
import { Track, PlaybackProgress } from "../Model"
import { ServerApi, ServerEvent } from "./ServerApi"

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

    @action
    private handleEvent = (e: ServerEvent) => {
        switch (e.type) {
            case "VolumeChanged":
                this.volume = e.args.volume
                this.muted = e.args.muted
                return
            case "PlaybackPaused":
                if (this.playingTrack !== null) {
                    this.playingTrack.progress = {
                        positionSecs: e.args.position_secs,
                        timestampOffsetMillis: null,
                    }
                }
                this.playing = false
                return
            case "PlaybackResumed":
                if (this.playingTrack !== null) {
                    this.playingTrack.progress = {
                        positionSecs: e.args.position_secs,
                        timestampOffsetMillis: performance.now(),
                    }
                }
                this.playing = true
                return
            case "PlayingTrackChanged":
                if (e.args === null) {
                    this.playingTrack = null
                } else {
                    this.serverApi.request("GetTrack", { track_id: e.args.id }).then(track => {
                        if (track === null) throw Error("unknown track " + e.args!.id)
                        this.playingTrack = {
                            track,
                            durationSecs: e.args!.duration_secs,
                            progress: { positionSecs: 0, timestampOffsetMillis: performance.now() },
                        }
                    })
                }
                return
        }
    }

    @observable volume = 0.5
    @observable muted = false
    @observable playing = false
    @observable
    playingTrack: {
        track: Track
        durationSecs: number
        progress: PlaybackProgress
    } | null = null

    changeVolume = (muted: boolean, volume?: number) => this.serverApi.request("ChangeVolume", { muted, volume })

    togglePause = () => this.serverApi.request("TogglePause")

    skipToNext = () => this.serverApi.request("SkipToNext")

    stop = () => this.serverApi.request("Stop")

    enqueue = (trackId: string) => this.serverApi.request("Enqueue", { track_id: trackId })
}
