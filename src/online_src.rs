//Functions to communicate with online sources.

use mop_structs::SongFile;

use std::io;

//TODO: Fix errors

pub fn retrieve_metadata_online(song_file: &SongFile) -> io::Result<()>{
    let musicbrainz = check_musicbrainz(song_file);
    if musicbrainz.Ok(){
        return Ok(());
    }

    Err()
}

fn check_musicbrainz(song_file: &SongFile) -> io::Result<()>{
    Err()
}