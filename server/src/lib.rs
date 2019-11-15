#![deny(clippy::all)]
// TODO: enable pedantic

pub mod api;
mod bootstrap;
pub mod errors;
mod file_completions;
mod file_formats;
mod http;
pub mod ids;
mod library;
pub mod model;
mod playback;
mod player;
mod queue;
pub mod serde;
pub mod server;
pub mod services;
mod websocket;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate anyhow;
