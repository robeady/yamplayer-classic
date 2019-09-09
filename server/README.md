# Prototype music player server

## Installation

Use rustup to install the nightly edition of the rust toolchain

    rustup toolchain install nightly

If you are using IntelliJ, open the `server` directory as a cargo project. Update your toolchain location (Settings > Languages & Frameworks > Rust) to point to the nightly version of rust. On windows this is in `C:\Users\<username>\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\bin`.

## Running

Run with `cargo run`. It will start the server on port 8080.

## Usage (instructions outdated and no longer working)

Enqueue a file by POSTing to /play:

    { "path": "<file path>" }

Try it

    cat request.json | curl -X POST -H "Content-Type: application/json" http://127.0.0.1:8080/play -d @-

## Observations

Tested with flac, should support mp3, wav and ogg as well.

Expect some stuttering (on windows at least), seems cured if CPU priority of the process is increased.
