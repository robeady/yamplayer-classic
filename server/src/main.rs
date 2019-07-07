use crate::errors::Try;

mod api;
mod bootstrap;
mod errors;
mod file_completions;
mod http;
mod library;
mod player;
mod server;
mod websocket;

fn main() -> Try<()> {
    env_logger::init();
    server::run_server()
}
