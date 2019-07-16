use crate::errors::{string_err, Try};
use crate::file_completions::complete_file_path;
use crate::library::{Library, TrackId};
use crate::player::PlayerApp;
use parking_lot::Mutex;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "method", content = "params")]
pub enum Request {
    Enqueue { track_id: String },
    Stop,
    TogglePause,
    ChangeVolume { volume: f32 },
    CompleteFilePath { prefix: String },
    GetLibrary,
    AddToLibrary { path: String },
}

fn json_res(result: Try<impl serde::Serialize>) -> JsonResult {
    match result {
        // TODO: think about what to do if serialisation fails. maybe a special Err?
        Ok(ref ok) => Ok(serde_json::to_string(ok).unwrap()),
        Err(ref err) => Err(serde_json::to_string(err).unwrap()),
    }
}

fn json(value: &impl serde::Serialize) -> JsonResult {
    // TODO: think about what to do if serialisation fails. maybe a special Err?
    Ok(serde_json::to_string(value).unwrap())
}

pub type JsonResult = std::result::Result<String, String>;

pub fn handle_request(
    player: &Mutex<PlayerApp>,
    library: &Mutex<Library>,
    request: Request,
) -> JsonResult {
    use Request::*;
    match request {
        Enqueue { ref track_id } => json_res(enqueue(player, library, track_id)),
        Stop => json(&stop(player)),
        TogglePause => json(&toggle_pause(player)),
        ChangeVolume { volume } => json(&set_volume(player, volume)),
        CompleteFilePath { ref prefix } => json_res(completions(prefix)),
        GetLibrary => json(&list_library(&*library.lock())),
        AddToLibrary { path } => json_res(add_to_library(library, path)),
    }
}

fn enqueue(state: &Mutex<PlayerApp>, library: &Mutex<Library>, track_id: &str) -> Try<()> {
    let track_id = TrackId(track_id.parse()?);
    let lib = library.lock();
    let file_path = &lib
        .get_track(track_id)
        .ok_or(string_err(format!("Unknown track {}", track_id.0)))?
        .file_path;
    log::info!("enqueueing track {} from {}", track_id.0, file_path);
    let mut player = state.lock();
    player.add_to_queue(file_path)?;
    Ok(())
}

fn stop(player: &Mutex<PlayerApp>) {
    player.lock().empty_queue();
}

fn set_volume(state: &Mutex<PlayerApp>, volume: f32) -> () {
    let mut player = state.lock();
    player.set_volume(volume);
}

fn toggle_pause(state: &Mutex<PlayerApp>) -> () {
    let mut player = state.lock();
    player.toggle_pause();
}

#[derive(Debug, Serialize)]
struct LibraryResp<'a> {
    tracks: Vec<TrackResp<'a>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TrackResp<'a> {
    id: String,
    file_path: &'a str,
    title: &'a str,
    artist: &'a str,
    album: &'a str,
}

fn list_library(library: &Library) -> LibraryResp {
    let tracks = library
        .list_tracks()
        .map(|(id, t)| TrackResp {
            id: id.0.to_string(),
            file_path: &t.file_path,
            title: &t.title,
            artist: &t.artist,
            album: &t.album,
        })
        .collect();
    LibraryResp { tracks }
}

fn add_to_library(library: &Mutex<Library>, track_file_path: String) -> Try<()> {
    library.lock().add_track(track_file_path)?;
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct CompleteFilePathResp {
    completions: Vec<String>,
}

fn completions(prefix: &str) -> Try<CompleteFilePathResp> {
    Ok(CompleteFilePathResp {
        completions: complete_file_path(prefix)?,
    })
}
