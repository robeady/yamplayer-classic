import React, { useState } from "react"
import { withRouter, RouteComponentProps } from "react-router-dom"
import { Flex, Box, Heading, Button, Image } from "rebass"
import { css } from "linaria"
import { styled } from "linaria/react"
import { Link } from "./links"
import { SearchResults } from "../backend/ServerApi"
import { useBackend } from "../backend/backend"
import { observer } from "mobx-react-lite"
import { TrackTable } from "../components/TrackTable"

export const SearchResultsScreen = observer((props: { query: string }) => {
    const results = useBackend().library.getSearchResults(props.query)
    if (results === "loading") {
        return <span>loading</span>
    }
    return (
        <>
            <Heading py={3}>Tracks</Heading>
            <Tracks {...results} />
            <Button>Go home</Button>
            <Heading py={3}>Artists</Heading>
            <Artists artists={results.artists} />
            <Heading py={3}>Albums</Heading>
            <Albums albums={results.albums} />
        </>
    )
})

const HoverImage = styled(Image)`
    &:hover {
        opacity: 0.8;
    }
`

const Tracks = (props: SearchResults) => {
    return (
        <TrackTable
            tracks={props.tracks.map(t => ({
                id: t.track.external_id,
                coverImageUrl: t.album.info.cover_image_url,
                title: t.track.info.title,
                albumName: t.album.info.title,
                artistName: t.artist.info.name,
                durationSecs: t.track.info.duration_secs,
            }))}
        />
    )
}

const Artists = (props: { artists: SearchResults["artists"] }) => {
    return (
        <Flex flexWrap="wrap">
            {props.artists.map(a => {
                const link = `/artists/${a.library_id || a.external_id}`
                return (
                    <Flex flexDirection="column" p={3}>
                        <Link to={link}>
                            <HoverImage
                                className={css`
                                    border-radius: 50%;
                                `}
                                src={a.info.image_url || unknownArtistImageUrl}
                                height={200}
                                width={200}
                            />
                        </Link>
                        <Link
                            marginTop={2}
                            className={css`
                                text-align: center;
                            `}
                            to={link}>
                            {a.info.name}
                        </Link>
                    </Flex>
                )
            })}
        </Flex>
    )
}

const Albums = (props: { albums: SearchResults["albums"] }) => {
    return (
        <Flex flexWrap="wrap">
            {props.albums.map(a => (
                <Flex p={3} flexDirection="column">
                    <HoverImage src={a.info.cover_image_url || unknownArtistImageUrl} height={200} width={200} />
                    <Box
                        marginTop={2}
                        maxWidth={200}
                        className={css`
                            text-align: center;
                        `}>
                        {a.info.title}
                    </Box>
                </Flex>
            ))}
        </Flex>
    )
}

const unknownArtistImageUrl =
    "https://e-cdns-images.dzcdn.net/images/artist/d41d8cd98f00b204e9800998ecf8427e/264x264-000000-80-0-0.jpg"

export const SearchBox = withRouter((props: RouteComponentProps) => {
    const [text, setText] = useState("")
    return (
        <input
            value={text}
            onKeyDown={e => {
                if (e.key === "Enter") {
                    props.history.push(`/search/${text}`)
                }
            }}
            onChange={e => setText(e.target.value)}
        />
    )
})
