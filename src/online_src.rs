//Functions to communicate with online sources.
use mop_structs::SongFile;

use std::io::{Error, ErrorKind, self};
use std::str;
use curl::easy::Easy;

fn get_user_agent() -> String{
    return format!("MetadataOrganizationProgram/{} ({})", env!("CARGO_PKG_VERSION"), env!("CARGO_PKG_AUTHORS"));
}

pub fn retrieve_metadata_online(song_file: &SongFile) -> io::Result<()>{
    error!("{}",get_user_agent());
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

fn check_musicbrainz(song_file: &SongFile) -> io::Result<()>{
    let api_endpoint = "https://musicbrainz.org/ws/2/";
    let mut request_url = String::new();
    request_url.push_str(api_endpoint);
    request_url.push_str("artist?query=");
    request_url.push_str(song_file.metadata.artist().unwrap());
    request_url = request_url.replace(" ","%20"); //FIXME: need to handle more characters in URL encoding
    info!("Request-url:{}",request_url.as_str());
    
    let mut buffer = String::new();
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

    let curly_result = str::from_utf8(&mut dst).unwrap();
    info!("CURL Result:");
    info!("{}",curly_result);

    Err(Error::new(ErrorKind::Other, "This isn't implemented."))
}

fn check_allmusic(song_file: &SongFile) -> io::Result<()>{
    Err(Error::new(ErrorKind::Other, "This isn't implemented."))
}