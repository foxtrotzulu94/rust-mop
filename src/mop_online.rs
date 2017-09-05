//Functions to communicate with online sources.
use mop_structs::SongFile;
use src_music_brainz;
use src_allmusic;

use std::io::{Error, ErrorKind, self};
use std::str;
use curl::easy::Easy;
use xml::reader::{EventReader, XmlEvent};

pub fn get_user_agent() -> String{
    return format!("MetadataOrganizationProgram/{} ({})", env!("CARGO_PKG_VERSION"), env!("CARGO_PKG_AUTHORS"));
}

pub fn make_get_request(request_url: &String) -> io::Result<String>{
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

pub fn retrieve_metadata_online(song_file: &SongFile) -> io::Result<()>{
    //TODO: Do a bit more generic approach
    let musicbrainz = src_music_brainz::check(song_file);
    if musicbrainz.is_ok(){
        return Ok(());
    }
    let all_music = src_allmusic::check(song_file);
    if all_music.is_ok(){
        return Ok(());
    }

    //If you get to this point, return an error
    Err(Error::new(ErrorKind::NotFound, "SongFile was unchanged"))
}