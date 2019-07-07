use crate::errors::Try;
use crate::file_completions::complete_file_path;
use crate::library::{Library, Track};
use crate::player::PlayerApp;
use parking_lot::Mutex;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "method", content = "params")]
pub enum Request {
    Play { path: String },
    TogglePause,
    ChangeVolume { volume: f32 },
    CompleteFilePath { prefix: String },
    GetLibrary,
}

fn json_res(result: Try<impl serde::Serialize>) -> std::result::Result<String, String> {
    match result {
        // TODO: think about what to do if serialisation fails. maybe a special Err?
        Ok(ref ok) => Ok(serde_json::to_string(ok).unwrap()),
        Err(ref err) => Err(serde_json::to_string(err).unwrap()),
    }
}

fn json(value: &impl serde::Serialize) -> std::result::Result<String, String> {
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
        Play { ref path } => json_res(play(player, path)),
        TogglePause => json(&toggle_pause(player)),
        ChangeVolume { volume } => json(&set_volume(player, volume)),
        CompleteFilePath { ref prefix } => json_res(completions(prefix)),
        GetLibrary => json(&list_library(&*library.lock())),
    }
}

fn play(state: &Mutex<PlayerApp>, path: &str) -> Try<()> {
    log::info!("loading path {}", path);
    let mut player = state.lock();
    player.play_file(path)?;
    Ok(())
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
    tracks: Vec<&'a Track>,
}

fn list_library(library: &Library) -> LibraryResp {
    let tracks = library.list_tracks().map(|(_, t)| t).collect();
    LibraryResp { tracks }
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
