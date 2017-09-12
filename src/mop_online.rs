//Functions to communicate with online sources.
use mop_structs::SongFile;
use src_music_brainz;
use src_allmusic;

use std::io::{Error, ErrorKind, self};
use std::str;
use curl::easy::Easy;
use xml::reader::{EventReader, XmlEvent};
use url::percent_encoding::{utf8_percent_encode, percent_decode, DEFAULT_ENCODE_SET};

pub fn get_user_agent() -> String{
    return format!("MetadataOrganizationProgram/{} ({})", env!("CARGO_PKG_VERSION"), env!("CARGO_PKG_AUTHORS"));
}

pub fn percent_encode(some_url : &str) -> io::Result<String>{
     let retval = utf8_percent_encode(some_url, DEFAULT_ENCODE_SET).collect();
     Ok(retval)
}

pub fn make_get_request(endpoint : &str, request_path: &str) -> io::Result<String>{
    //Only check that the request_path doesn't contain the endpoint
    let mut request_url = String::from(endpoint);
    if request_path.contains(endpoint){
        request_url = String::from(request_path);
    } else{
        request_url.push_str(request_path);
    }

    info!("Performing GET request to: {}", request_url);
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

pub fn retrieve_metadata_online(song_file: &mut SongFile) -> io::Result<()>{
    if !song_file.has_search_key(){
        return Err(Error::new(ErrorKind::InvalidInput, "SongFile does not have appropriate search key filled"))
    }

    //TODO: Do a bit more generic approach
    let musicbrainz = src_music_brainz::check(song_file);
    if musicbrainz.is_ok(){
        return Ok(());
    }
    else{
        error!("{:?}",musicbrainz);
    }
    // let all_music = src_allmusic::check(song_file);
    // if all_music.is_ok(){
    //     return Ok(());
    // }

    //If you get to this point, return an error
    Err(Error::new(ErrorKind::NotFound, "SongFile was unchanged"))
}