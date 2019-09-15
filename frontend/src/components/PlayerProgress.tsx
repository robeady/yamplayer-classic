import React, { useState, useEffect } from "react"
import { PlaybackTiming } from "../Model"
import Slider from "@material-ui/core/Slider"

export const PlayerProgress = (props: PlaybackTiming) => {
    const [currentTimestampOffsetMillis, setCurrentTimestampOffsetMillis] = useState(performance.now())

    const playing = props.playingSinceTimestamp !== "paused"

    useEffect(() => {
        if (playing) {
            setCurrentTimestampOffsetMillis(performance.now())
            const handle = setInterval(() => setCurrentTimestampOffsetMillis(performance.now()), 500)
            return () => {
                clearInterval(handle)
            }
        }
    }, [playing])

    let secondsSinceStart =
        props.playingSinceTimestamp === "paused"
            ? props.positionSecsAtTimestamp
            : clamp(
                  (currentTimestampOffsetMillis - props.playingSinceTimestamp) / 1000 + props.positionSecsAtTimestamp,
                  0,
                  props.durationSecs,
              )

    return <PlayerProgressRaw currentSecs={secondsSinceStart} totalSecs={props.durationSecs} />
}

export const NoPlayerProgress = () => <PlayerProgressRaw currentSecs={null} totalSecs={null} />

const PlayerProgressRaw = (props: { currentSecs: number | null; totalSecs: number | null }) => (
    <div className="progress">
        <Time secs={props.currentSecs} />
        <div className="progressBar">
            <Slider
                min={0}
                max={1}
                step={0.001}
                value={props.currentSecs !== null && props.totalSecs !== null ? props.currentSecs / props.totalSecs : 0}
            />
        </div>
        <Time secs={props.totalSecs} />
    </div>
)

const Time = (props: { secs: number | null }) => {
    if (props.secs === null) return <span />
    const totalSeconds = Math.round(props.secs)
    const mins = Math.floor(totalSeconds / 60)
    const secs = (totalSeconds % 60).toString().padStart(2, "0")
    return (
        <span>
            {mins}:{secs}
        </span>
    )
}

const clamp = (number: number, min: number, max: number) => (number < min ? min : number > max ? max : number)
