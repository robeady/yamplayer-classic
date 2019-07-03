import React, { useState } from "react"
import "./App.scss"

const App = () => {
    const [trackFilePath, setTrackFilePath] = useState("")
    const [serverResponse, setServerResponse] = useState({ status: 0, body: "" })
    const [volume, setVolume] = useState(1.0)
    return (
        <>
            <div className="App">
                <h1>Music Player</h1>
                <div>
                    <label>
                        <span>Track file path: </span>
                        <input value={trackFilePath} onChange={e => setTrackFilePath(e.target.value)} />
                    </label>
                    <button onClick={() => play(trackFilePath).then(setServerResponse)}>Play</button>
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
            </div>
        </>
    )
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

export default App
