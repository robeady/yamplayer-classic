import React, { useState, useEffect } from "react"
import Autosuggest from "react-autosuggest"
import "./App.scss"
import { RPCWebSocket } from "./websocket"

const App = () => {
    const [trackFilePath, setTrackFilePath] = useState("")
    const [serverResponse, setServerResponse] = useState({ status: 0, body: "" })
    const [volume, setVolume] = useState(1.0)
    const [suggestions, setSuggestions] = useState([] as string[])
    return (
        <>
            <div className="App">
                <h1>Music Player</h1>
                <div>
                    <label>
                        <span>Track file path: </span>
                        <Autosuggest
                            suggestions={suggestions}
                            getSuggestionValue={x => x}
                            inputProps={{
                                onChange: (_, e) => setTrackFilePath(e.newValue),
                                value: trackFilePath,
                            }}
                            onSuggestionsFetchRequested={({ value }) => getSuggestions(value).then(setSuggestions)}
                            onSuggestionsClearRequested={() => setSuggestions([])}
                            renderSuggestion={s => <div>{s}</div>}
                        />
                    </label>
                    <button onClick={() => play(trackFilePath).then(setServerResponse)}>Play</button>
                    <button onClick={() => togglePause().then(setServerResponse)}>Toggle Pause</button>
                </div>
                <div>
                    <label>
                        <span>Volume: </span>
                        <input
                            type="number"
                            onChange={e => {
                                setVolume(e.target.valueAsNumber)
                                serverSetVolume(e.target.valueAsNumber).then(setServerResponse)
                            }}
                            min="0.0"
                            max="1.0"
                            step="0.1"
                            value={volume}
                        />
                    </label>
                </div>
                <div>
                    <span>
                        Last server response: {serverResponse.status || ""} {serverResponse.body}
                    </span>
                </div>
                <Library />
            </div>
        </>
    )
}

function Library() {
    const [tracks, setTracks] = useState([] as Track[])
    useEffect(() => {
        getTracksInLibrary().then(setTracks)
    }, [])
    return (
        <div>
            <h2>Library</h2>
            <ol className="trackList">{tracks.map(t => <Track track={t} />)}</ol>
        </div>
    )
}

function Track(props: { track: Track }) {
    return (
        <li className="trackListRow">
            <div className="title">{props.track.title}</div>
            <div className="artist">{props.track.artist}</div>
            <div className="album">{props.track.album}</div>
        </li>
    )
}

interface Track {
    album: string
    artist: string
    title: string
}

async function callApi(method: string, params?: unknown) {
    const response = await fetch("/api", {
        method: "POST",
        body: JSON.stringify({ method, params }),
        headers: {
            "Content-Type": "application/json",
        },
    })
    const body = await response.json()
    return { status: response.status, body }
}

const ws = new RPCWebSocket("ws://127.0.0.1:8080/ws")

async function callWsApi(method: string, params?: unknown) {
    const response = await ws.query({ method, params })
    return { status: 200, body: response as any }
}

async function getTracksInLibrary() {
    const response = await callWsApi("GetLibrary")
    if (response.status === 200) {
        return response.body.tracks as Track[]
    } else {
        return []
    }
}

async function serverSetVolume(volume: number) {
    return await callWsApi("ChangeVolume", { volume })
}

async function play(track: string) {
    return await callWsApi("Play", { path: track })
}

async function togglePause() {
    return await callWsApi("TogglePause")
}

async function getSuggestions(prefix: string) {
    const response = await callWsApi("CompleteFilePath", { prefix })
    if (response.status === 200) {
        return response.body.completions as string[]
    } else {
        return []
    }
}

export default App
