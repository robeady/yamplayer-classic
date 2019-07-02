# Prototype music player server

Run with `cargo run --release`. It will start the server on port 8080.

Play a file by POSTing to /play:

    { "path": "<file path>" }

Try it

    cat request.json | curl -X POST -H "Content-Type: application/json" http://127.0.0.1:8080/play -d @-

Tested with flac, should support mp3, wav and ogg as well.

Expect some stuttering (on windows at least), seems cured if CPU priority of the process is set to high

