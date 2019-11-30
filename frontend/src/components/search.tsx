import React, { useState } from "react"
import { withRouter, RouteComponentProps } from "react-router-dom"
import { Flex, Heading, Button, Grid } from "../elements"
import { css } from "linaria"
import { styled } from "linaria/react"
import { Link } from "./links"
import { SearchResults } from "../backend/ServerApi"
import { useBackend } from "../backend/backend"
import { observer } from "mobx-react-lite"
import { TrackTable } from "../components/TrackTable"
import { space } from "../elements/theme"
import { iterate } from "iterare"

export const SearchResultsScreen = observer((props: { query: string }) => {
    const results = useBackend().library.getSearchResults(props.query)
    if (results === "loading") {
        return <span>loading</span>
    }
    return (
        <>
            <Heading>Tracks</Heading>
            <Tracks {...results} />
            <Button>Go home</Button>
            <Heading>Artists</Heading>
            <Artists artists={results.artists} />
            <Heading>Albums</Heading>
            <Albums albums={results.albums} />
        </>
    )
})

const HoverImage = styled.img`
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
        <Flex
            className={css`
                flex-wrap: wrap;
                margin: 0 -${space[3]};
            `}>
            {iterate(props.artists)
                .take(24)
                .map(a => {
                    const link = `/artists/${a.library_id || a.external_id}`
                    return (
                        <Grid
                            direction="y"
                            gap={1}
                            className={css`
                                margin: ${space[2]} ${space[3]};
                            `}>
                            <Link to={link}>
                                <HoverImage
                                    className={css`
                                        border-radius: 50%;
                                    `}
                                    src={a.info.image_url || unknownArtistImageUrl}
                                    height={150}
                                    width={150}
                                />
                            </Link>
                            <Link
                                className={css`
                                    text-align: center;
                                    max-width: 150px;
                                `}
                                to={link}>
                                {a.info.name}
                            </Link>
                        </Grid>
                    )
                })
                .toArray()}
        </Flex>
    )
}

const Albums = (props: { albums: SearchResults["albums"] }) => {
    return (
        <Flex
            className={css`
                flex-wrap: wrap;
                margin: 0 -${space[3]};
            `}>
            {iterate(props.albums)
                .take(24)
                .map(a => (
                    <Grid
                        direction="y"
                        gap={2}
                        className={css`
                            margin: ${space[2]} ${space[3]};
                        `}>
                        <HoverImage src={a.info.cover_image_url || unknownArtistImageUrl} height={150} width={150} />
                        <span
                            className={css`
                                max-width: 150px;
                                text-align: center;
                            `}>
                            {a.info.title}
                        </span>
                    </Grid>
                ))
                .toArray()}
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
