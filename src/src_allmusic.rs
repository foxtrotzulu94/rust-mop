//All the functions needed to scrape AllMusic (http://www.allmusic.com/)
use mop_structs::{SongFile,BasicMetadata};
use mop_online::{make_get_request, percent_encode};

use std::io::{Error, ErrorKind, self};
use std::str;

use select::document::Document;
use select::predicate::{Predicate, Attr, Class, Name};

static API_ENDPOINT: &'static str = "http://www.allmusic.com/";

/// Returns the URL corresponding to the song on allmusic.com
fn find_matching_song_url(artist: &str, title: &str) -> io::Result<String>{
    let mut request_path = String::from("search/songs/");
    request_path.push_str(percent_encode(title)?.as_str());
    let html_data = make_get_request(API_ENDPOINT, request_path.as_str())?;

    //Now parse the HTML
    let doc = Document::from(html_data.as_str());
    let results = doc.find(Class("song"));
    for node in results{
        //Safety check
        let title_node = node.find(Class("title")).next().unwrap();
        let curr_title = title_node.text().replace("\"","");
        let curr_artist = node.find(Class("performers")).next().unwrap().text().replace("by ","");
        if curr_title.trim() == title && curr_artist.trim() == artist{
            let song_url = title_node.find(Name("a")).next().unwrap().attr("href").unwrap();
            debug!("Entry: {} - {}", curr_title.trim(), curr_artist.trim());
            debug!("{}",song_url);
            return Ok(String::from(song_url));
        }
    }

    Err(Error::new(ErrorKind::Other, "AllMusic: Song URL not found!"))
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
    // song_file.save();

    Err(Error::new(ErrorKind::Other, "AllMusic: This isn't implemented."))
}