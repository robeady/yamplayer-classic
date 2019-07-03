use actix_web::post;
use actix_web::{error, web, App, HttpServer, Result};
use log;
use rodio::decoder::Decoder;
use rodio::{Device, Sink, Source};
use serde_derive::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{Cursor, Read};
use std::sync::{Arc, Mutex};

type Try<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug, Serialize, Deserialize)]
struct PlayRequest {
    path: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct VolumeRequest {
    volume: f32,
}

pub struct PlayerApp {
    device: Arc<Device>,
    sink: Sink,
}

impl PlayerApp {
    pub fn new() -> Try<PlayerApp> {
        let device = Arc::new(rodio::default_output_device().ok_or("no output device")?);
        let sink = Sink::new(&device);
        Ok(PlayerApp { device, sink })
    }

    pub fn volume(&self) -> f32 {
        self.sink.volume()
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.sink.set_volume(volume)
    }

    pub fn play_file(&mut self, path: &str) -> Try<()> {
        log::debug!("loading file");
        let buffer = load_file(path)?;
        log::debug!("file loaded into memory");
        let source: Decoder<_> = Decoder::new(Cursor::new(buffer))?;
        let duration_secs = source.total_duration().expect("unknown duration").as_secs();
        log::info!(
            "playing track with length: {}:{:02}",
            duration_secs / 60,
            duration_secs % 60
        );
        self.sink.stop();
        let new_sink = Sink::new(&self.device);
        new_sink.append(source);
        new_sink.set_volume(self.sink.volume());
        new_sink.play();
        self.sink = new_sink;
        Ok(())
    }
}

#[post("/player/play")]
fn play(state: web::Data<Mutex<PlayerApp>>, req: web::Json<PlayRequest>) -> Result<()> {
    log::info!("loading path {}", req.path);
    let mut player = state.lock().unwrap();
    player
        .play_file(&req.path)
        .map_err(error::ErrorInternalServerError)?;
    Ok(())
}

#[post("/player/volume")]
fn set_volume(state: web::Data<Mutex<PlayerApp>>, req: web::Json<VolumeRequest>) -> String {
    let mut player = state.lock().unwrap();
    player.set_volume(req.volume);
    "".to_string()
}

pub fn run_server() -> Try<()> {
    let player_app = PlayerApp::new()?;
    let state = web::Data::new(Mutex::new(player_app));
    HttpServer::new(move || {
        App::new()
            .register_data(state.clone())
            .service(play)
            .service(set_volume)
    })
    .bind("127.0.0.1:8080")?
    .run()?;
    Ok(())
}

fn load_file(path: &str) -> Try<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}
