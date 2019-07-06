use crate::library::{Library, Track};
use actix_web::web::{Data, Json};
use actix_web::{error, get, post, App, HttpResponse, HttpServer, Result};
use log;
use parking_lot::Mutex;
use rodio::decoder::Decoder;
use rodio::{Device, Sink, Source};
use serde_derive::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Cursor, Read};
use std::path::Path;
use std::sync::Arc;

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

    pub fn toggle_pause(&mut self) {
        if self.sink.is_paused() {
            self.sink.play()
        } else {
            self.sink.pause()
        }
    }

    pub fn play_file(&mut self, path: &str) -> Try<()> {
        log::debug!("loading file");
        let buffer = load_file(path)?;
        log::debug!("file loaded into memory");
        let source: Decoder<_> = Decoder::new(Cursor::new(buffer))?;
        match source.total_duration().map(|d| d.as_secs()) {
            None => log::warn!("playing track with unknown length"),
            Some(duration_secs) => log::info!(
                "playing track with length: {}:{:02}",
                duration_secs / 60,
                duration_secs % 60
            ),
        }
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
fn play(state: Data<Mutex<PlayerApp>>, req: Json<PlayRequest>) -> Result<()> {
    log::info!("loading path {}", req.path);
    let mut player = state.lock();
    player
        .play_file(&req.path)
        .map_err(error::ErrorInternalServerError)?;
    Ok(())
}

#[post("/player/volume")]
fn set_volume(state: Data<Mutex<PlayerApp>>, req: Json<VolumeRequest>) -> () {
    let mut player = state.lock();
    player.set_volume(req.volume);
}

#[post("/player/toggle-pause")]
fn toggle_pause(state: Data<Mutex<PlayerApp>>) -> () {
    let mut player = state.lock();
    player.toggle_pause();
}

#[derive(Debug, Serialize, Deserialize)]
struct CompleteFilePathReq {
    prefix: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CompleteFilePathResp {
    completions: Vec<String>,
}

#[post("/completions/file-path")]
fn handle_complete_file_path(req: Json<CompleteFilePathReq>) -> Result<Json<CompleteFilePathResp>> {
    Ok(Json(CompleteFilePathResp {
        completions: complete_file_path(&req.prefix).map_err(error::ErrorInternalServerError)?,
    }))
}

#[derive(Debug, Serialize)]
struct LibraryResp<'a> {
    tracks: Vec<&'a Track>,
}

#[get("/library")]
fn list_library(library: Data<Mutex<Library>>) -> HttpResponse {
    HttpResponse::Ok().json2(&LibraryResp {
        tracks: library.lock().list_tracks().map(|(_, t)| t).collect(),
    })
}

fn complete_file_path(prefix: &str) -> Try<Vec<String>> {
    let index_of_last_slash = prefix
        .rfind('/')
        .or_else(|| prefix.rfind('\\'))
        .unwrap_or(prefix.len() - 1);
    let (directory, prefix) = prefix.split_at(index_of_last_slash + 1);
    let mut result = Vec::new();
    for file in Path::new(directory).read_dir()? {
        let name = file?
            .file_name()
            .into_string()
            .map_err(|s| format!("invalid file name {:?}", s))?;
        if name.starts_with(prefix) {
            result.push([directory, &name].concat())
        }
    }
    Ok(result)
}

pub fn run_server() -> Try<()> {
    let player_app = PlayerApp::new()?;
    let mut library = Library::new();
    bootstrap_library(&mut library)?;
    let shared_player = Data::new(Mutex::new(player_app));
    let shared_library = Data::new(Mutex::new(library));
    HttpServer::new(move || {
        App::new()
            .register_data(shared_player.clone())
            .register_data(shared_library.clone())
            .service(play)
            .service(set_volume)
            .service(handle_complete_file_path)
            .service(toggle_pause)
            .service(list_library)
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

#[derive(Deserialize)]
struct Bootstrap {
    tracks: Vec<String>,
}

fn bootstrap_library(library: &mut Library) -> Try<()> {
    let b: Bootstrap = serde_yaml::from_reader(BufReader::new(File::open("bootstrap.yml")?))?;
    for track in b.tracks {
        log::info!("adding track {} to library", track);
        library.add_track(track)?;
    }
    Ok(())
}
