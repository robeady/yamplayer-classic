import React, { useState } from "react"
import "./App.scss"

const App = () => {
    const [trackFilePath, setTrackFilePath] = useState("")
    const [serverResponse, setServerResponse] = useState({ status: 0, body: "" })

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
                    <span>
                        Server response: {serverResponse.status || ""} {serverResponse.body}
                    </span>
                </div>
            </div>
        </>
    )
}

async function play(track: string) {
    const response = await fetch("/play", {
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
