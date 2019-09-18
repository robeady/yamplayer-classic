use crate::errors::Try;

mod api;
mod bootstrap;
mod errors;
mod file_completions;
mod http;
mod ids;
mod library;
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
