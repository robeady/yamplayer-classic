import React, { ReactNode } from "react"
import { css } from "linaria"
import { observer } from "mobx-react-lite"
import { useBackend } from "../backend/backend"
import { Flex, Button } from "../elements"
import { Box } from "rebass"
import { styled } from "linaria/react"
import { color, space } from "../elements/theme"

export const TrackTable = observer((props: { tracks: TableTrack[] }) => {
    const { playback, library } = useBackend()
    return (
        <>
            {(props.tracks || []).map(t => (
                <TrackRow track={t} enqueue={playback.enqueue} addToFirstPlaylist={library.addToFirstPlaylist} />
            ))}
        </>
    )
})

export interface TableTrack {
    id: string
    coverImageUrl: string
    title: string
    artistName: string
    albumName: string
    durationSecs: number
}

const TrackIcon = styled.img`
    display: block;
`

function TrackRow(props: {
    track: TableTrack
    enqueue: (trackId: string) => void
    addToFirstPlaylist: (trackId: string) => void
}) {
    return (
        <Flex
            className={css`
                border-bottom: 1px solid ${color.border};
                padding: ${space[1]};
                align-items: center;
            `}
            key={props.track.id}>
            <Box pr={3}>
                <TrackIcon width={32} height={32} src={props.track.coverImageUrl} alt="" />
            </Box>
            <Box flex="1" onClick={() => props.enqueue(props.track.id)} title="Click to enqueue">
                <Clickable>{props.track.title}</Clickable>
            </Box>
            <Button onClick={() => props.addToFirstPlaylist(props.track.id)}>Add to first playlist</Button>
            <Box flex="1">{props.track.artistName}</Box>
            <Box flex="1">{props.track.albumName}</Box>
            <Box flex="0.1">{formatDuration(props.track.durationSecs)}</Box>
        </Flex>
    )
}

const formatDuration = (durationSecs: number) =>
    `${Math.floor(durationSecs / 60)}:${String(durationSecs % 60).padStart(2, "0")}`

const Clickable = (props: Children) => (
    <Box
        className={css`
            cursor: pointer;
        `}>
        {props.children}
    </Box>
)

interface Children {
    children?: ReactNode
}
