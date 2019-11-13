use crate::errors::Try;
use crate::library::Library;
use serde_derive::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

#[derive(Deserialize)]
struct Bootstrap {
    playlists: HashMap<String, Vec<String>>,
    tracks: Vec<String>,
}

pub fn bootstrap_library(library: &Library) -> Try<()> {
    library.in_transaction(|| {
        let b: Bootstrap = serde_yaml::from_reader(BufReader::new(File::open("bootstrap.yml")?))?;
        for (playlist, tracks) in b.playlists {
            log::info!("creating playlist {}", playlist);
            let playlist_id = library.create_playlist(playlist)?;
            for playlist_track in tracks {
                log::info!("adding track {} to playlist", playlist_track);
                let track_id = library.add_local_track(playlist_track)?;
                library.add_track_to_playlist(track_id, playlist_id)?;
            }
        }
        for track in b.tracks {
            log::info!("adding track {} to library", track);
            library.add_local_track(track)?;
        }
        Ok(())
    })
}
