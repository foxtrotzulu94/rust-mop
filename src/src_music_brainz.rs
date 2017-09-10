//All the functions needed to use the MusicBrainz API (https://musicbrainz.org/doc/Development/XML_Web_Service/Version_2)
use mop_structs::{SongFile,BasicMetadata};
use mop_online::{make_get_request, percent_encode};

use std::io::{Error, ErrorKind, self};
use std::str;
use curl::easy::Easy;
use xml::reader::{EventReader, XmlEvent};

static API_ENDPOINT: &'static str = "https://musicbrainz.org/ws/2/";

//TODO: Better error handling (i.e. away from std::io::Result)

fn find_artist_id(xml_parser: EventReader<&[u8]>) -> io::Result<String>{
    let mut artist_id = String::new();
    let mut confidence = 75;
    for e in xml_parser {
        match e {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                if name.local_name.contains("artist") {
                    let mut some_artist_id = String::new();
                    let mut curr_confidence=0;
                    //Find score attribute and artist ID
                    for attr in attributes{
                        match attr.name.local_name.as_str(){
                            "id" => some_artist_id=attr.value,
                            "score" => curr_confidence = attr.value.parse::<i32>().unwrap(),
                            _ => (),
                        }
                    }
                    if curr_confidence > confidence{
                        artist_id = some_artist_id;
                        debug!("{} confidence:{}",artist_id,confidence);
                        break;
                    }
                }
            }
            Err(e) => {
                error!("Error: {:?}", e);
                return Err(Error::new(ErrorKind::Other, e.msg()))
            }
            _ => {}
        }
    }

    Ok(artist_id)
}

fn find_recording_data(xml_parser: EventReader<&[u8]>) -> io::Result<BasicMetadata>{
    //TODO: get the request and return a well formed metadata struct instance

    //Two steps: 

    //Go through recordings
    let mut temp_metadata = BasicMetadata::new();
    let mut recording_list : Vec<BasicMetadata> = Vec::new();
    let mut is_reading_release = false;
    for e in xml_parser {
        match e {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                //TODO: populate the list
                match name.local_name.as_str() {
                    "release" => is_reading_release = true,
                    "title" => {
                        if temp_metadata.album.is_empty(){
                            //TODO: apparently we have to get the next "event", XmlEvent::Characters
                            // ref: https://github.com/Enet4/dicom-rs/blob/b13c13facf5ddcedce92344f86c334109f958aa6/dictionary_builder/main.rs#L226
                        }
                    }
                    _ => {}
                }   
            }
            Ok(XmlEvent::EndElement { name }) => {
                match name.local_name.as_str() {
                    "recording"|"release" => {
                        is_reading_release = false;
                        //Save and continue with next recording
                        if temp_metadata.has_some_data(){
                            recording_list.push(temp_metadata);
                        }
                        temp_metadata = BasicMetadata::new();
                    }
                    
                    _ => {}
                }  
            }
            Err(e) => {
                error!("Error: {:?}", e);
                return Err(Error::new(ErrorKind::Other, e.msg()))
            }
            _ => {}
        }
    }

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
    let parser = EventReader::from_str(curly_result.as_str());
    let artist_id = find_artist_id(parser)?;
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
    warn!("{}",curly_result_two.as_str());

    //TODO:
    //Get all album info (Sort by year, if possible)
    //Query Genre somewhere!

    Err(Error::new(ErrorKind::Other, "This isn't implemented."))
}