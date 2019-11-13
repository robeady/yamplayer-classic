use crate::errors::Try;
use crate::model::{AlbumInfo, ArtistInfo, TrackInfo};
use fstrings::{f, format_args_f};

pub fn read_metadata(file_path: String) -> Try<(TrackInfo, AlbumInfo, ArtistInfo)> {
    let flac = claxon::FlacReader::open(&file_path)?;
    let tag = |name: &str| {
        flac.get_tag(name)
            .next()
            .map(|t| t.to_string())
            .unwrap_or_else(|| {
                log::warn!("no {} tag in {}", name, file_path);
                f!("UNKNOWN {name}")
            })
    };
    let duration_secs = (flac
        .streaminfo()
        .samples
        .unwrap_or_else(|| panic!("no stream info in {}", file_path))
        as f32)
        / flac.streaminfo().sample_rate as f32;
    let track_title = tag("TITLE");
    let album_title = tag("ALBUM");
    let artist_name = tag("ARTIST");
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
