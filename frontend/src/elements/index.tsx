import { styled } from "linaria/react"
import { fontSize, space, color } from "./theme"

function unreachable(x: never): never {
    throw Error("unreachable: " + x)
}

export const Flex = styled.div`
    display: flex;
`

export const Heading = styled.h2<{ size?: keyof typeof fontSize }>`
    font-size: ${props => fontSize[props.size === undefined ? 2 : props.size]};
`

export const Button = styled.button`
    text-align: center;
    padding: ${space[2]} ${space[3]};
    color: white;
    background-color: ${color.primary};
    border: 0;
`

function gridAutoFlow(direction?: "x" | "y"): "column" | "row" | "unset" {
    switch (direction) {
        case "x":
            return "column"
        case "y":
            return "row"
        case undefined:
            return "unset"
        default:
            return unreachable(direction)
    }
}

interface GridProps {
    direction?: "x" | "y"
    gap?: keyof typeof space
}

export const Grid = styled.div<GridProps & JSX.IntrinsicElements["div"]>`
    display: grid;
    grid-auto-flow: ${props => gridAutoFlow(props.direction)};
    grid-gap: ${props => (props.gap === undefined ? "unset" : space[props.gap])};
    /* with lots of y-direction grids in a row, these ensure that everything aligns to top */
    align-items: start;
    align-content: start;
`
