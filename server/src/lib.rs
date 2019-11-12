#![deny(clippy::all)]
// TODO: enable pedantic

pub mod api;
mod bootstrap;
pub mod errors;
mod file_completions;
mod http;
mod library;
pub mod model;
mod playback;
mod player;
mod queue;
pub mod serde;
pub mod server;
mod websocket;

#[macro_use]
extern crate diesel;
