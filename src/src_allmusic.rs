//All the functions needed to scrape AllMusic (http://www.allmusic.com/)
use mop_structs::{SongFile,BasicMetadata};
use mop_online::{make_get_request, percent_encode};

use std::io::{Error, ErrorKind, self};
use std::str;
use curl::easy::Easy;
use xml::reader::{EventReader, XmlEvent};

/// Returns the URL corresponding to the song on allmusic.com
fn find_matching_song_url(artist: &str, title: &str) -> io::Result<String>{
    Err(Error::new(ErrorKind::Other, "AllMusic: This isn't implemented."))
}

/// Takes the oldest album and returns the url to it
fn find_artist_album(song_url : &str)-> io::Result<String>{
    Err(Error::new(ErrorKind::Other, "AllMusic: This isn't implemented."))
}

fn build_metadata(album_url : &str)-> io::Result<BasicMetadata>{
    Err(Error::new(ErrorKind::Other, "AllMusic: This isn't implemented."))
}

pub fn check(song_file: &mut SongFile) -> io::Result<()>{
    // When we query AllMusic, it's mostly scraping from their site
    //  We try searching song -> song by artist -> album by artist
    let mut album_url = String::new();
    {
        let artist = safe_expand!(song_file.metadata.artist(),"");
        let title = safe_expand!(song_file.metadata.title(),"");

        let song_url = find_matching_song_url(artist,title)?;
        album_url = find_artist_album(song_url.as_str())?;
    }
    let recording_metadata = build_metadata(album_url.as_str())?;

    song_file.set_basic_metadata(recording_metadata);
    song_file.save();

    Err(Error::new(ErrorKind::Other, "AllMusic: This isn't implemented."))
}