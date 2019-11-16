CREATE TABLE external_tracks (
    _id INTEGER PRIMARY KEY NOT NULL,
    track_id INTEGER NOT NULL REFERENCES tracks (track_id),
    service_id TEXT NOT NULL,
    external_id TEXT NOT NULL
);

CREATE TABLE external_albums (
    _id INTEGER PRIMARY KEY NOT NULL,
    album_id INTEGER NOT NULL REFERENCES albums (album_id),
    service_id TEXT NOT NULL,
    external_id TEXT NOT NULL
);

CREATE TABLE external_artists (
    _id INTEGER PRIMARY KEY NOT NULL,
    artist_id INTEGER NOT NULL REFERENCES artists (artist_id),
    service_id TEXT NOT NULL,
    external_id TEXT NOT NULL
);
