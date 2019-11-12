CREATE TABLE tracks (
    track_id INTEGER PRIMARY KEY NOT NULL,
    album_id INTEGER NOT NULL REFERENCES albums (album_id),
    artist_id INTEGER NOT NULL REFERENCES artists (artist_id),
    title TEXT NOT NULL,
    isrc TEXT,
    duration_secs REAL NOT NULL,
    file_path TEXT
);

CREATE TABLE albums (
    album_id INTEGER PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    cover_image_url TEXT,
    release_date TEXT
);

CREATE TABLE artists (
    artist_id INTEGER PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    image_url TEXT
);

CREATE TABLE playlists (
    playlist_id INTEGER PRIMARY KEY NOT NULL,
    name TEXT NOT NULL
);

CREATE TABLE playlist_tracks (
    _id INTEGER PRIMARY KEY NOT NULL,
    playlist_id INTEGER NOT NULL REFERENCES playlists (playlist_id),
    track_id INTEGER NOT NULL REFERENCES tracks (track_id)
);

