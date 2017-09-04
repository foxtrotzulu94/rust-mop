//Basic data structures and their method implementations

use std::string::String;
use std::time::Duration;

pub struct Metadata {
    name: String,
    artist: String,
    genre: String,
    length: Duration,
    album: String,
    // CoverArt: //Don't use yet
    track_number: i32,
    date: i32,
    composer: String,

    pub is_complete: bool,
    pub is_correct: bool,
}

pub struct SongFile{
    metadata: Metadata,
    extension: String,
    dir_path: String,
    filename: String,
}

//FIXME: Temp method to check that everything is OK
pub fn new_metadata() -> Metadata{
    return Metadata {
            name: String::new(),
            artist: String::new(),
            genre: String::new(),
            length: Duration::new(0,0),
            album: String::new(),
            // CoverArt: //Don't use yet
            track_number: -1,
            date: -1,
            composer: String::new(),

            is_complete: false,
            is_correct: false,
        };
}

