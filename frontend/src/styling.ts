import { preset } from "@rebass/preset"
import { merge } from "lodash"

// preset: https://github.com/rebassjs/rebass/blob/master/packages/preset/src/index.js
export const THEME = merge(preset, {
    colors: {
        // custom primary color
        primary: "hsl(38, 75%, 55%)",
        primaryHover: "hsl(38, 70%, 50%)",
        border: "hsl(0, 0%, 92%)",
    },
    buttons: {
        primary: {
            cursor: "pointer",
            fontWeight: "body",
            "&:hover": {
                bg: "primaryHover",
            },
        },
    },
    variants: {
        link: {
            color: "text",
            textDecoration: "none",
            "&:hover": {
                color: "primaryHover",
            },
        },
    },
})
console.log(THEME)
