use crate::library::Library;
use crate::websocket;
use actix_web::web;
use actix_web::{App, HttpServer};
use parking_lot::Mutex;

use crate::bootstrap::bootstrap_library;
use crate::errors::Try;
use crate::http;
use crate::player::PlayerApp;

pub struct State {
    pub player: Mutex<PlayerApp>,
    pub library: Mutex<Library>,
}

pub fn run_server() -> Try<()> {
    let player_app = PlayerApp::new()?;
    let mut library = Library::new();
    bootstrap_library(&mut library)?;
    let shared_state = web::Data::new(State {
        player: Mutex::new(player_app),
        library: Mutex::new(library),
    });
    HttpServer::new(move || {
        App::new()
            .register_data(shared_state.clone())
            .service(http::api_handler)
            .route("/ws/", web::get().to(websocket::index))
        //            .service(play)
        //            .service(set_volume)
        //.service(handle_complete_file_path)
        //.service(toggle_pause)
        //.service(list_library)
    })
    .bind("127.0.0.1:8080")?
    .run()?;
    Ok(())
}
