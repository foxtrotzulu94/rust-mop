//Functions to communicate with online sources.
use mop_structs::SongFile;

use std::io::{Error, ErrorKind, self};
use std::str;
use curl::easy::Easy;
use xml::reader::{EventReader, XmlEvent};

fn get_user_agent() -> String{
    return format!("MetadataOrganizationProgram/{} ({})", env!("CARGO_PKG_VERSION"), env!("CARGO_PKG_AUTHORS"));
}

fn make_get_request(request_url: &String) -> io::Result<String>{
    let mut curly = Easy::new();
    let mut dst = Vec::new();
    {       
        curly.get(true)?;
        curly.useragent(get_user_agent().as_str())?;
        curly.url(request_url.as_str())?;
        let mut connection = curly.transfer();
        connection.write_function(|data|{
                dst.extend_from_slice(data);
                Ok(data.len())
            }).unwrap();
        connection.perform().unwrap(); 
    }

    //If we've made it this far, let's risk the panic!
    let ret_val = String::from_utf8(dst.clone()).unwrap();
    Ok(ret_val)
}

fn check_musicbrainz(song_file: &SongFile) -> io::Result<()>{
    let api_endpoint = "https://musicbrainz.org/ws/2/";
    let mut request_url = String::new();
    request_url.push_str(api_endpoint);
    request_url.push_str("artist?query=");
    request_url.push_str(song_file.metadata.artist().unwrap());
    request_url = request_url.replace(" ","%20"); //FIXME: need to handle more characters in URL encoding

    //info!("Request-url:{}",request_url.as_str());

    //MusicBrainz always replies in XML
    let curly_result = make_get_request(&request_url).unwrap();
    assert!(curly_result.len()>10);
    let parser = EventReader::from_str(curly_result.as_str());
    let mut artist_id = String::new();
    let mut confidence = 75;

    for e in parser {
        match e {
            //TODO: Scan data to find corresponding ArtistID and break on match
            Ok(XmlEvent::StartElement { name, attributes, namespace }) => {
                if name.local_name.contains("artist") {
                    info!("{}",name);
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
                        info!("{} confidence:{}",artist_id,confidence);
                        break;
                    }
                }
            }
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
            _ => {}
        }
    }

    Err(Error::new(ErrorKind::Other, "This isn't implemented."))
}

fn check_allmusic(song_file: &SongFile) -> io::Result<()>{
    Err(Error::new(ErrorKind::Other, "This isn't implemented."))
}

pub fn retrieve_metadata_online(song_file: &SongFile) -> io::Result<()>{
    //TODO: Do a bit more generic approach
    let musicbrainz = check_musicbrainz(song_file);
    if musicbrainz.is_ok(){
        return Ok(());
    }
    let all_music = check_allmusic(song_file);
    if all_music.is_ok(){
        return Ok(());
    }

    //If you get to this point, return an error
    Err(Error::new(ErrorKind::NotFound, "SongFile was unchanged"))
}