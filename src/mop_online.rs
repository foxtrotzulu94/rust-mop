//Functions to communicate with online sources.
use mop_structs::SongFile;
use src_music_brainz;
use src_allmusic;

use std::io::{Error, ErrorKind, self};
use std::str;
use curl::easy::Easy;
use url::percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};

pub fn get_user_agent() -> String{
    return format!("MetadataOrganizationProgram/{} ({})", env!("CARGO_PKG_VERSION"), env!("CARGO_PKG_AUTHORS"));
}

pub fn percent_encode(some_url : &str) -> io::Result<String>{
     let mut retval : String = utf8_percent_encode(some_url, DEFAULT_ENCODE_SET).collect();
     retval = retval.replace("&","%26").replace("/","%2F").replace(":","%3A");
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

    info!("MOP_Online: Performing GET request to: {}", request_url);
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
    let response_code = curly.response_code().unwrap();
    if response_code >= 400{
        return Err(Error::new(ErrorKind::InvalidData, format!("MOP_Online: CURL Request returned error {}",response_code)));
    }

    //If we've made it this far, let's risk the panic!
    let ret_val = String::from_utf8(dst.clone()).unwrap();
    Ok(ret_val)
}

pub fn retrieve_metadata_online(song_file: &mut SongFile) -> io::Result<()>{
    if !song_file.has_search_key(){
        return Err(Error::new(ErrorKind::InvalidInput, "MOP_Online: SongFile does not have appropriate search key filled"))
    }

    //[src_music_brainz::check, src_allmusic::check].to_vec();
    let mut online_sources : Vec<fn(&mut SongFile) -> io::Result<()>>	 = Vec::new();
    online_sources.push(src_allmusic::check);
    online_sources.push(src_music_brainz::check);

    for check in online_sources{
        let result = check(song_file);
        match result {
            Err(expr) => warn!("{}", expr),
            Ok(_) => {
                return Ok(());
            },
        }
    }

    //If you get to this point, return an error
    Err(Error::new(ErrorKind::NotFound, "MOP_Online: SongFile was unchanged"))
}