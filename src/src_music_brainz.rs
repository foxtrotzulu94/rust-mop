//All the functions needed to use the MusicBrainz API (https://musicbrainz.org/doc/Development/XML_Web_Service/Version_2)
use mop_structs::SongFile;
use mop_online::{make_get_request};

use std::io::{Error, ErrorKind, self};
use std::str;
use curl::easy::Easy;
use xml::reader::{EventReader, XmlEvent};

fn find_artist_id(xml_parser: EventReader<&[u8]>) -> String{
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
                error!("Error: {}", e);
                break;
            }
            _ => {}
        }
    }

    return artist_id;
}

pub fn check(song_file: &SongFile) -> io::Result<()>{
    let api_endpoint = "https://musicbrainz.org/ws/2/";
    let mut request_url = String::new();
    request_url.push_str(api_endpoint);
    request_url.push_str("artist?query=");
    request_url.push_str(song_file.metadata.artist().unwrap());
    request_url = request_url.replace(" ","%20"); //FIXME: need to handle more characters in URL encoding
    debug!("Request-url:{}",request_url.as_str());

    //MusicBrainz always replies in XML
    let curly_result = make_get_request(&request_url).unwrap();
    let parser = EventReader::from_str(curly_result.as_str());
    let artist_id = find_artist_id(parser);
    warn!("{} - Artist ID={}",song_file.metadata.artist().unwrap(), artist_id);

    //Now query for the Song name
    //TODO:
    //Get all album info (Sort by year, if possible)
    //Query Genre somewhere!

    Err(Error::new(ErrorKind::Other, "This isn't implemented."))
}