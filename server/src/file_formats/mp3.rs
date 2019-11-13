use crate::errors::Try;
use crate::model::{AlbumInfo, ArtistInfo, TrackInfo};
use fstrings::{f, format_args_f};
use id3::Tag;
use std::fs::File;
use std::io::BufReader;

pub fn read_metadata(file_path: String) -> Try<(TrackInfo, AlbumInfo, ArtistInfo)> {
    // TODO: move mp3 handling code to separate module
    let mp3_tags = Tag::read_from_path(&file_path)?;
    let tag = |name: &str, value: Option<&str>| {
        value.map(|t| t.to_string()).unwrap_or_else(|| {
            log::warn!("no {} tag in {}", name, file_path);
            f!("UNKNOWN {name}")
        })
    };
    let mut mp3 = minimp3::Decoder::new(BufReader::new(File::open(&file_path)?));
    let mut duration_secs = 0_f32;
    loop {
        let frame_result = mp3.next_frame();
        let frame = match frame_result {
            Ok(frame) => frame,
            // An error caused by some IO operation required during decoding.
            Err(minimp3::Error::Io(e)) => return Err(e.into()),
            // The decoder tried to parse a frame from its internal buffer, but there was not enough.
            Err(minimp3::Error::InsufficientData) => {
                panic!("not enough data in encoder buffer??? {}", file_path)
            }
            // The decoder encountered data which was not a frame (ie, ID3 data), and skipped it.
            Err(minimp3::Error::SkippedData) => continue,
            // The decoder has reached the end of the provided reader.
            Err(minimp3::Error::Eof) => break,
        };
        let seconds_of_audio =
            (frame.data.len() / frame.channels) as f32 / frame.sample_rate as f32;
        duration_secs += seconds_of_audio;
    }
    let track_title = tag("TITLE", mp3_tags.title());
    let album_title = tag("ALBUM", mp3_tags.album());
    let artist_name = tag("ARTIST", mp3_tags.artist());
    Ok((
        TrackInfo {
            title: track_title,
            isrc: None,
            duration_secs,
            file_path: Some(file_path),
        },
        AlbumInfo {
            title: album_title,
            cover_image_url: None,
            release_date: None,
        },
        ArtistInfo {
            name: artist_name,
            image_url: None,
        },
    ))
}
