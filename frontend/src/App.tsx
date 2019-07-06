import React, { useState, useEffect } from "react"
import Autosuggest from "react-autosuggest"
import "./App.scss"

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
        <div className="tracksTable">
            <ol>{tracks.map(t => <li>{JSON.stringify(t)}</li>)}</ol>
        </div>
    )
}

interface Track {
    album: string
    artist: string
    title: string
}

async function getTracksInLibrary() {
    const response = await fetch("/library")
    if (response.status === 200) {
        const body = await response.json()
        return body.tracks as Track[]
    } else {
        return []
    }
}

async function serverSetVolume(volume: number) {
    const response = await fetch("/player/volume", {
        method: "POST",
        body: JSON.stringify({ volume }),
        headers: {
            "Content-Type": "application/json",
        },
    })
    const body = await response.text()
    return { status: response.status, body }
}

async function play(track: string) {
    const response = await fetch("/player/play", {
        method: "POST",
        body: JSON.stringify({ path: track }),
        headers: {
            "Content-Type": "application/json",
        },
    })
    const body = await response.text()
    return { status: response.status, body }
}

async function togglePause() {
    const response = await fetch("/player/toggle-pause", {
        method: "POST",
    })
    const body = await response.text()
    return { status: response.status, body }
}

async function getSuggestions(prefix: string) {
    const response = await fetch("/completions/file-path", {
        method: "POST",
        body: JSON.stringify({ prefix }),
        headers: {
            "Content-Type": "application/json",
        },
    })
    if (response.status === 200) {
        const body = await response.json()
        return body.completions as string[]
    } else {
        return []
    }
}

export default App
