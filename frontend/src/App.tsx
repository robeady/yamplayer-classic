import React from "react"
import PlayArrow from "@material-ui/icons/PlayArrow"
import Pause from "@material-ui/icons/Pause"
import SkipNext from "@material-ui/icons/SkipNext"
import SkipPrevious from "@material-ui/icons/SkipPrevious"
import VolumeDown from "@material-ui/icons/VolumeDown"
import VolumeUp from "@material-ui/icons/VolumeUp"
import VolumeMute from "@material-ui/icons/VolumeMute"
import Slider from "@material-ui/core/Slider"
import "./App.scss"
import { observer } from "mobx-react-lite"
import { Track } from "./Model"
import { useBackend } from "./backend/backend"
import { css } from "linaria"
import { NoPlayerProgress, PlayerProgress } from "./components/PlayerProgress"
import { configure } from "mobx"
configure({
    computedRequiresReaction: true,
    enforceActions: "observed",
})

const App = () => (
        <div className="player">
            <NowPlaying />
            <LeftNav />
            <Main />
        </div>
)

const Main = () => (
    <main className="main">
        <TrackList />
    </main>
)

const TrackList = observer(() => {
    const { library, playback } = useBackend()
    return (
        <div>
            <ol className="trackList">
                {Object.values(library.getLibrary() || {}).map(t => (
                    <TrackRow track={t} enqueue={() => playback.enqueue(t.id)} />
                ))}
            </ol>
        </div>
    )
})

function TrackRow(props: { track: Track; enqueue: () => void }) {
    return (
        <li className="trackListRow" key={props.track.id}>
            <div className="title clickable" onClick={props.enqueue} title="Click to enqueue">
                {props.track.title}
            </div>
            <div className="artist">{props.track.artist}</div>
            <div className="album">{props.track.album}</div>
        </li>
    )
}

const LeftNav = () => <nav className="leftNav">navigation links go here</nav>

const NowPlaying = observer(() => {
    const { playback: pb, library } = useBackend()
    const track = pb.currentTrack && library.getTrack(pb.currentTrack.trackId)
    return (
        <header className="nowPlaying">
            <div className="controls">
                <TrackSummary
                    art="https://i.scdn.co/image/93852b7922b792c49e9198e09314c6b885eb1ed2"
                    artist={(track && track.artist) || ""}
                    track={(track && track.title) || ""}
                />
                <PlaybackControls
                    playing={pb.playing}
                    onUnpause={() => pb.unpause()}
                    onPause={() => pb.pause()}
                    onPrev={() => {}}
                    onNext={() => pb.skipToNext()}
                />
                {pb.currentTrack === null ? (
                    <NoPlayerProgress />
                ) : (
                    <PlayerProgress
                        durationSecs={pb.currentTrack.durationSecs}
                        playingSinceTimestamp={pb.currentTrack.playingSinceTimestamp}
                        positionSecsAtTimestamp={pb.currentTrack.positionSecsAtTimestamp}
                    />
                )}
                <VolumeControl muted={pb.muted} volume={pb.volume} setVolume={pb.changeVolume} />
                <QueueControls />
            </div>
        </header>
    )
})

const TrackSummary = (props: { track: string; artist: string; art: string }) => (
    <div className="trackSummary">
        <img src={props.art} height={70} alt="" />
        <div className="trackDescription">
            <span>{props.track}</span>
            <span>{props.artist}</span>
        </div>
    </div>
)

const PlaybackControls = (props: {
    playing: boolean
    onUnpause: () => void
    onPause: () => void
    onPrev: () => void
    onNext: () => void
}) => (
    <div className="playbackControls">
        <SkipPrevious className="prevButton" />
        {props.playing ? (
            <Pause onClick={props.onPause} className="playPauseButton" />
        ) : (
            <PlayArrow onClick={props.onUnpause} className="playPauseButton" />
        )}
        <SkipNext onClick={props.onNext} className="nextButton" />
    </div>
)

const VolumeControl = (props: {
    muted: boolean
    volume: number
    setVolume: (muted: boolean, volume?: number) => void
}) => {
    return (
        <div className="volume">
            <div className="volumeButton">
                {props.volume <= 0 || props.muted ? (
                    <VolumeMute onClick={() => props.setVolume(false)} />
                ) : props.volume <= 0.5 ? (
                    <VolumeDown onClick={() => props.setVolume(true)} />
                ) : (
                    <VolumeUp onClick={() => props.setVolume(true)} />
                )}
            </div>
            <div className="volumeSlider">
                <Slider
                    min={0}
                    max={1}
                    step={0.01}
                    value={props.muted ? 0 : props.volume}
                    onChange={(_, v) => props.setVolume(false, v as number)}
                />
            </div>
        </div>
    )
}
const QueueControls = () => <div>QueueControls</div>

export default App
