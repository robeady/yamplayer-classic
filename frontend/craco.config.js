const CracoLinariaPlugin = require("craco-linaria")

module.exports = {
    eslint: {
        mode: "file",
    },
    plugins: [
        {
            plugin: CracoLinariaPlugin,
        },
    ],
}
