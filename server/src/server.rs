use crate::library::Library;
use log;
use parking_lot::Mutex;

use crate::api::EventSink;
use crate::api::{App, Request};
use crate::bootstrap::bootstrap_library;
use crate::errors::Try;
use crate::http;
use crate::player::PlayerApp;
use crate::websocket::ws_connection;
use std::sync::Arc;
use warp::Filter;

pub fn run_server() -> Try<()> {
    let event_sink = Arc::new(EventSink::empty());
    let player_app = PlayerApp::new(Arc::clone(&event_sink))?;
    let mut library = Library::new(Arc::clone(&event_sink));
    if let Err(e) = bootstrap_library(&mut library) {
        log::warn!("Did not bootstrap library: {}", e)
    }
    let app = Arc::new(App {
        player: player_app,
        library: Mutex::new(library),
        event_sink: Arc::clone(&event_sink),
    });

    let app_state = warp::any().map(move || app.clone());

    let http_rpc = warp::post2()
        .and(warp::path("api"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(app_state.clone())
        .map(|request: Request, app: Arc<App>| http::api_handler(app, request));

    let websocket = warp::get2()
        .and(warp::path("ws"))
        .and(warp::path::end())
        .and(warp::ws2())
        .and(app_state.clone())
        .map(|ws: warp::ws::Ws2, app: Arc<App>| ws.on_upgrade(move |ws| ws_connection(app, ws)));

    warp::serve(http_rpc.or(websocket)).run(([127, 0, 0, 1], 8080));

    Ok(())
}
