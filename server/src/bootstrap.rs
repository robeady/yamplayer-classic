use crate::errors::Try;
use crate::library::Library;
use serde_derive::Deserialize;
use std::fs::File;
use std::io::BufReader;

#[derive(Deserialize)]
struct Bootstrap {
    tracks: Vec<String>,
}

pub fn bootstrap_library(library: &mut Library) -> Try<()> {
    let b: Bootstrap = serde_yaml::from_reader(BufReader::new(File::open("bootstrap.yml")?))?;
    for track in b.tracks {
        log::info!("adding track {} to library", track);
        library.add_track(track)?;
    }
    Ok(())
}
