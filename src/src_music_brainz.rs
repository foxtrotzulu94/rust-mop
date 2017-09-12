//All the functions needed to use the MusicBrainz API (https://musicbrainz.org/doc/Development/XML_Web_Service/Version_2)
use mop_structs::{SongFile,BasicMetadata};
use mop_online::{make_get_request, percent_encode};
use xml_wrap::{XmlMap};

use std::str;
use std::io::{Error, ErrorKind, self};
use xml::reader::{EventReader, XmlEvent};

static API_ENDPOINT: &'static str = "https://musicbrainz.org/ws/2/";

//TODO: Better error handling (i.e. away from std::io::Result)

fn find_artist_id(raw_data: &str) -> io::Result<String>{
    let mut artist_id = String::new();
    let mut confidence = 75;

    let data = XmlMap::from_str(raw_data);
    let possible_artists = &data.root["metadata"]["artist-list"].get_matching_children("artist");

    for an_artist in *possible_artists{
        let mut some_artist_id = String::new();
        let mut curr_confidence=0;
        for (name, value) in an_artist.attributes.iter(){
            match name.as_str() {
                "id" => some_artist_id = value.clone(),
                "score" => curr_confidence = value.parse::<i32>().unwrap(),
                _ => (),
            }
        }
        if curr_confidence > confidence{
            artist_id = some_artist_id;
            confidence = curr_confidence;
            debug!("{} confidence:{}",artist_id,confidence);
            break;
        }
    }

    Ok(artist_id)
}

fn find_recording_data(raw_data: &str) -> io::Result<BasicMetadata>{
    //TODO: get the request and return a well formed metadata struct instance

    //Two steps: 

    //Go through recordings
    let mut temp_metadata = BasicMetadata::new();
    let mut recording_list : Vec<BasicMetadata> = Vec::new();
    
    let data = XmlMap::from_str(raw_data);
    let possible_recordings = &data.root["metadata"]["recording-list"].get_matching_children("recording");


    //Select best one (i.e. go for first release ever)

    //Construct the metadata off of that

    Err(Error::new(ErrorKind::Other, "This isn't implemented."))
}

pub fn check(song_file: &SongFile) -> io::Result<()>{
    debug!("Checking '{} - {}'", song_file.metadata.artist().unwrap(), song_file.metadata.title().unwrap());
    let mut artist_request = String::new();
    artist_request.push_str("artist?query=");
    let sanitized_artist_name = percent_encode(song_file.metadata.artist().unwrap()).unwrap();
    artist_request.push_str(&sanitized_artist_name);
    debug!("Request-url:{}",artist_request.as_str());

    //MusicBrainz always replies in XML
    let curly_result = make_get_request(API_ENDPOINT, &artist_request).unwrap();
    let artist_id = find_artist_id(curly_result.as_str())?;
    //TODO: Check if the artist was found or error
    warn!("{} - Artist ID={}",song_file.metadata.artist().unwrap(), artist_id);

    //Now query for the Song name
    let mut song_request = String::new();
    song_request.push_str("recording/?query=");
    song_request.push_str(percent_encode(song_file.metadata.title().unwrap()).unwrap().as_str());
    song_request.push_str(percent_encode(" AND arid%3A").unwrap().as_str()); //"%20AND%20arid%3A"
    song_request.push_str(artist_id.as_str());
    debug!("Request-url:{}",song_request.as_str());
    let curly_result_two = make_get_request(&API_ENDPOINT, &song_request).unwrap();
    //TODO: Check if more than one song was returned or error!
    // warn!("{}",curly_result_two.as_str());

    //TODO:
    //Get all album info (Sort by year, if possible)
    //Query Genre somewhere!

    Err(Error::new(ErrorKind::Other, "This isn't implemented."))
}