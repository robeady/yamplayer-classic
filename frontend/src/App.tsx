import React, { ReactElement } from "react"
import PlayArrow from "@material-ui/icons/PlayArrow"
import Pause from "@material-ui/icons/Pause"
import SkipNext from "@material-ui/icons/SkipNext"
import SkipPrevious from "@material-ui/icons/SkipPrevious"
import VolumeDown from "@material-ui/icons/VolumeDown"
import VolumeUp from "@material-ui/icons/VolumeUp"
import QueueMusic from "@material-ui/icons/QueueMusic"
import VolumeMute from "@material-ui/icons/VolumeMute"
import Slider from "@material-ui/core/Slider"
import { observer } from "mobx-react-lite"
import { Track } from "./Model"
import { useBackend } from "./backend/backend"
import { css } from "linaria"
import { styled } from "linaria/react"
import { NoPlayerProgress, PlayerProgress } from "./components/PlayerProgress"
import { FlexDiv } from "./components/common"
import { BrowserRouter, Route } from "react-router-dom"
import { NavLink } from "./components/links"
import { SearchBox, SearchResultsScreen } from "./components/search"
import { ThemeProvider } from "emotion-theming"
import { THEME } from "./styling"

const App = () => (
    <ThemeProvider theme={THEME}>
        <BrowserRouter>
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
                        padding: 1.5rem;
                    `}>
                    <SearchBox />
                    <Navigation />
                </nav>
                <main
                    className={css`
                        grid-area: main;
                        padding: 2rem;
                    `}>
                    <Route path="/" exact render={() => <TrackList listing="library" />} />
                    <Route
                        path="/playlists/:id"
                        render={props => <TrackList listing={{ playlistId: props.match.params.id }} />}
                    />
                    <Route
                        path="/search/:query"
                        render={props => <SearchResultsScreen query={props.match.params.query} />}
                    />
                </main>
            </div>
        </BrowserRouter>
    </ThemeProvider>
)

const Navigation = observer(() => {
    const { library } = useBackend()
    const NavList = styled.ol`
        list-style: none;
        margin: 0;
        padding-left: 0;
    `
    return (
        <>
            <h3>Playlists</h3>
            <NavList>
                <NavListItem name="Library" id="library" linkTo="/" icon={<QueueMusic />} />
                {(library.listPlaylists() || []).map(p => (
                    <NavListItem name={p.name} id={p.id} linkTo={`/playlists/${p.id}`} icon={<QueueMusic />} />
                ))}
            </NavList>
        </>
    )
})

const NavListItem = (props: { id: string; linkTo: string; name: string; icon: ReactElement }) => {
    return (
        <li
            className={css`
                display: flex;
                padding: 0.25rem 0;
            `}
            key={props.id}>
            {props.icon}
            <NavLink
                className={css`
                    margin-left: 0.3rem;
                `}
                to={props.linkTo}>
                {props.name}
            </NavLink>
        </li>
    )
}

const TrackList = observer((props: { listing: "library" | { playlistId: string } }) => {
    const { library, playback } = useBackend()
    const tracks =
        props.listing === "library" ? library.getLibrary() : library.listPlaylistTracks(props.listing.playlistId)
    return (
        <div>
            <ol
                className={css`
                    margin: 0;
                    padding-left: 0;
                `}>
                {(tracks || []).map(t => (
                    <TrackRow track={t} enqueue={() => playback.enqueue(t.track_id)} />
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
            key={props.track.track_id}>
            <div
                className={css`
                    flex-basis: 40%;
                    cursor: pointer;
                `}
                onClick={props.enqueue}
                title="Click to enqueue">
                {props.track.track_info.title}
            </div>
            <div
                className={css`
                    flex-basis: 30%;
                `}>
                {props.track.artist_info.name}
            </div>
            <div
                className={css`
                    flex-basis: 30%;
                `}>
                {props.track.album_info.title}
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
                    artist={(track && track.artist_info.name) || ""}
                    track={(track && track.track_info.title) || ""}
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
                    font-size: 2rem !important;
                `}
            />
            {props.playing ? (
                <Pause
                    onClick={props.onPause}
                    className={css`
                        font-size: 3rem !important;
                    `}
                />
            ) : (
                <PlayArrow
                    onClick={props.onUnpause}
                    className={css`
                        font-size: 3rem !important;
                    `}
                />
            )}
            <SkipNext
                onClick={props.onNext}
                className={css`
                    font-size: 2rem !important;
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
