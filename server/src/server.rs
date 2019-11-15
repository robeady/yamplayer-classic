use crate::api::{App, Request};
use crate::api::{EventSink, Payload};
use crate::bootstrap::bootstrap_library;
use crate::errors::Try;
use crate::http;
use crate::library::Library;
use crate::player::PlayerApp;
use crate::services::{Service, ServiceId};
use crate::websocket::ws_connection;
use log;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use warp::Filter;

pub struct Server {
    services: HashMap<ServiceId, Box<dyn Service>>,
}

impl Server {
    pub fn new(services: Vec<Box<dyn Service>>) -> Self {
        Server {
            services: services.into_iter().map(|s| (s.id(), s)).collect(),
        }
    }

    pub fn run(self) -> Try<()> {
        let event_sink = Arc::new(EventSink::empty());
        event_sink.add_destination(Box::new(|payload: &Payload| {
            log::info!("event: {}", payload.json)
        }));
        let player_app = PlayerApp::new(Arc::clone(&event_sink))?;
        let database_path = format!(
            "database-{}.sqlite",
            chrono::Local::now().format("%Y-%m-%d_%H-%M-%S")
        );
        log::info!("opening database file {}", database_path);
        let library = Library::new(database_path, Arc::clone(&event_sink))?;
        if let Err(e) = bootstrap_library(&library) {
            log::warn!("Did not bootstrap library: {}", e)
        }
        let app = Arc::new(App {
            services: self.services,
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
            .and(app_state)
            .map(|ws: warp::ws::Ws2, app: Arc<App>| {
                ws.on_upgrade(move |ws| ws_connection(app, ws))
            });

        warp::serve(http_rpc.or(websocket)).run(([127, 0, 0, 1], 8080));

        Ok(())
    }
}
