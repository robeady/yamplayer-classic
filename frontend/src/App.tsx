import React from "react"
import PlayArrow from "@material-ui/icons/PlayArrow"
import Pause from "@material-ui/icons/Pause"
import SkipNext from "@material-ui/icons/SkipNext"
import SkipPrevious from "@material-ui/icons/SkipPrevious"
import VolumeDown from "@material-ui/icons/VolumeDown"
import VolumeUp from "@material-ui/icons/VolumeUp"
import VolumeMute from "@material-ui/icons/VolumeMute"
import Slider from "@material-ui/core/Slider"
import { observer } from "mobx-react-lite"
import { Track } from "./Model"
import { useBackend } from "./backend/backend"
import { css } from "linaria"
import { NoPlayerProgress, PlayerProgress } from "./components/PlayerProgress"
import { configure } from "mobx"
import { FlexDiv } from "./components/common"
configure({
    computedRequiresReaction: true,
    enforceActions: "observed",
})

const App = () => (
    <div
        className={css`
            height: 100vh;
            display: grid;
            grid-template-areas:
                "nowPlaying nowPlaying"
                "leftNav main";
            grid-template-columns: 200px auto;
            grid-template-rows: 100px auto;
        `}>
        <div
            className={css`
                grid-area: nowPlaying;
                background: #ddd;
            `}>
            <NowPlaying />
        </div>
        <nav
            className={css`
                grid-area: leftNav;
                background: #eee;
            `}>
            navigation links go here
        </nav>
        <main
            className={css`
                grid-area: main;
                padding: 2rem;
            `}>
            <TrackList />
        </main>
    </div>
)

const TrackList = observer(() => {
    const { library, playback } = useBackend()
    return (
        <div>
            <ol
                className={css`
                    margin: 0;
                    padding-left: 0;
                `}>
                {Object.values(library.getLibrary() || {}).map(t => (
                    <TrackRow track={t} enqueue={() => playback.enqueue(t.id)} />
                ))}
            </ol>
        </div>
    )
})

function TrackRow(props: { track: Track; enqueue: () => void }) {
    return (
        <li
            className={css`
                padding: 0.2rem;
                display: flex;
                text-align: left;
            `}
            key={props.track.id}>
            <div
                className={css`
                    flex-basis: 40%;
                    cursor: pointer;
                `}
                onClick={props.enqueue}
                title="Click to enqueue">
                {props.track.title}
            </div>
            <div
                className={css`
                    flex-basis: 30%;
                `}>
                {props.track.artist}
            </div>
            <div
                className={css`
                    flex-basis: 30%;
                `}>
                {props.track.album}
            </div>
        </li>
    )
}

const NowPlaying = observer(() => {
    const { playback: pb, library } = useBackend()
    const track = pb.currentTrack && library.getTrack(pb.currentTrack.trackId)
    return (
        <header>
            <div
                className={css`
                    display: flex;
                    padding: 1rem;
                    justify-content: space-between;
                `}>
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
    <FlexDiv>
        <img src={props.art} height={70} alt="" />
        <div
            className={css`
                display: flex;
                flex-direction: column;
                padding-left: 0.5rem;
            `}>
            <span>{props.track}</span>
            <span>{props.artist}</span>
        </div>
    </FlexDiv>
)

const PlaybackControls = (props: {
    playing: boolean
    onUnpause: () => void
    onPause: () => void
    onPrev: () => void
    onNext: () => void
}) => {
    return (
        <div>
            <SkipPrevious
                className={css`
                    font-size: 2rem;
                `}
            />
            {props.playing ? (
                <Pause
                    onClick={props.onPause}
                    className={css`
                        font-size: 3rem;
                    `}
                />
            ) : (
                <PlayArrow
                    onClick={props.onUnpause}
                    className={css`
                        font-size: 3rem;
                    `}
                />
            )}
            <SkipNext
                onClick={props.onNext}
                className={css`
                    font-size: 2rem;
                `}
            />
        </div>
    )
}

const VolumeControl = (props: {
    muted: boolean
    volume: number
    setVolume: (muted: boolean, volume?: number) => void
}) => {
    return (
        <FlexDiv>
            <div className="volumeButton">
                {props.volume <= 0 || props.muted ? (
                    <VolumeMute onClick={() => props.setVolume(false)} />
                ) : props.volume <= 0.5 ? (
                    <VolumeDown onClick={() => props.setVolume(true)} />
                ) : (
                    <VolumeUp onClick={() => props.setVolume(true)} />
                )}
            </div>
            <div
                className={css`
                    width: 6rem;
                    margin-left: 1rem;
                `}>
                <Slider
                    min={0}
                    max={1}
                    step={0.01}
                    value={props.muted ? 0 : props.volume}
                    onChange={(_, v) => props.setVolume(false, v as number)}
                />
            </div>
        </FlexDiv>
    )
}
const QueueControls = () => <div>QueueControls</div>

export default App
