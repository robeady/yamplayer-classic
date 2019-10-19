#![deny(clippy::all)]
// TODO: enable pedantic

use crate::errors::Try;

mod api;
mod bootstrap;
mod deezer;
mod errors;
mod file_completions;
mod http;
mod library;
mod model;
mod playback;
mod player;
mod queue;
mod serde;
mod server;
mod websocket;

fn main() -> Try<()> {
    setup_logging();
    server::run_server()
}

fn setup_logging() {
    env_logger::Builder::from_default_env()
        .default_format_timestamp_nanos(true)
        .init()
}
