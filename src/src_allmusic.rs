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
        let test_node = node.find(Class("performers")).next();
        match test_node{
            None => continue,
            _ => ()
        }
        let curr_artist = test_node.unwrap().text().replace("by ","");
        if curr_title.to_lowercase().trim() == title && curr_artist.to_lowercase().trim() == artist{
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
    let html_data = make_get_request(API_ENDPOINT, song_url)?;

    //Now parse the HTML.
    let doc = Document::from(html_data.as_str());
    let results = doc.find(Attr("itemprop","inAlbum"));
    for node in results{
        //Safety check
        let album_node = node.find(Class("artist-album")).next().unwrap();
        let album_url = album_node.find(Class("title")).next().unwrap()
                            .find(Name("a")).next().unwrap()
                            .attr("href").unwrap();
        debug!("Album {}",album_url);
        return Ok(String::from(album_url));
    }

    Err(Error::new(ErrorKind::Other, "AllMusic: No Albums found for song"))
}

fn build_metadata(artist: &str, title: &str, album_url : &str)-> io::Result<BasicMetadata>{
    let html_data = make_get_request(API_ENDPOINT, album_url)?;

    //Now parse the HTML.
    let doc = Document::from(html_data.as_str());
    let mut ret_val = BasicMetadata::new();

    //Get the release date
    let release_date_block = doc.find(Class("release-date")).next().unwrap().find(Name("span")).next().unwrap();
    let date_text = release_date_block.text();
    ret_val.date = date_text.trim()[date_text.len()-4..].parse::<i32>().unwrap();
    
    //Album title
    let album_title = doc.find(Class("album-title")).next().unwrap().text();
    ret_val.album = String::from(album_title.trim());

    //Sometimes, there aren't any tracks

    let track_node_option = doc.find(Class("track-listing")).next();
    if track_node_option.is_some(){
        let track_list = track_node_option.unwrap().find(Class("track"));
        for a_track in track_list{
            let track_title = a_track.find(Class("title")).next().unwrap().find(Name("a")).next().unwrap().text();
            if track_title.to_lowercase().trim() == title{
                //Track Number
                let track_num = a_track.find(Class("tracknum")).next().unwrap().text();
                ret_val.track_number = track_num.trim().parse::<u32>().unwrap();

                //Composer
                let composer_block = a_track.find(Class("composer")).next().unwrap().text();
                ret_val.composer = String::from(composer_block.trim());
                break;
            }
        }
    } else{
        ret_val.track_number = 1;
        //Composer is not set in this case
    }

    //Genre/Style
    let basic_info = doc.find(Class("basic-info")).next().unwrap();
    let style_block = basic_info.find(Class("styles")).next();
    if style_block.is_some(){
        let style = style_block.unwrap()
                        .find(Name("div")).next().unwrap()
                        .find(Name("a")).next().unwrap().text();
        ret_val.genre = String::from(style.trim());
    }

    Ok(ret_val)
}

pub fn check(song_file: &mut SongFile) -> io::Result<()>{
    info!("AllMusic: Checking '{} - {}'", song_file.metadata.artist().unwrap(), song_file.metadata.title().unwrap());
    // When we query AllMusic, it's mostly scraping from their site
    //  We try searching song -> song by artist -> album by artist

    let clean_artist = String::from(song_file.metadata.artist().unwrap()).to_lowercase();
    let clean_title = String::from(song_file.metadata.title().unwrap()).to_lowercase();

    let song_url = find_matching_song_url(
        clean_artist.as_str(),
        clean_title.as_str(),
    )?;
    info!("Checking Song URL");
    let album_url = find_artist_album(song_url.as_str())?;

    let recording_metadata = build_metadata(
        clean_artist.as_str(),
        clean_title.as_str(),
        album_url.as_str()
    )?;

    song_file.set_basic_metadata(recording_metadata);
    song_file.save();

    Ok(())
}