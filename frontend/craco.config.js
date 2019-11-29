const CracoLinariaPlugin = require("craco-linaria")
const CssSourcemapsPlugin = require("./craco-css-sourcemaps")

module.exports = {
    eslint: {
        mode: "file",
    },
    plugins: [
        {
            plugin: CracoLinariaPlugin,
            // either I'm stupid or inline configuration doesn't work. Config is placed in linaria.config.js instead.
        },
        {
            plugin: CssSourcemapsPlugin,
        },
    ],
}
