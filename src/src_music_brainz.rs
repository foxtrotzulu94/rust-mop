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

    if possible_artists.len() < 1{
        return Err(Error::new(ErrorKind::NotFound, "The Artist was not found on MusicBrainz"));
    }

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

    if artist_id.is_empty() {
        return Err(Error::new(ErrorKind::NotFound, "The Artist ID was not found with enough confidence!"));
    }
    //FIXME: Partially Uppercase the Genre
    Ok(artist_id)
}

fn find_recording_data(raw_data: &str) -> io::Result<BasicMetadata>{
    //TODO: get the request and return a well formed metadata struct instance

    let mut temp_metadata = BasicMetadata::new();
    let mut recording_list : Vec<BasicMetadata> = Vec::new();
    
    let data = XmlMap::from_str(raw_data);
    let possible_recordings = &data.root["metadata"]["recording-list"].get_matching_children("recording");

    //Select best one (by default, go for first release ever)
    let mut best_candidate = BasicMetadata::new();
    best_candidate.date = 9001;
    let min_confidence = 90;
    let mut tag_count = 0;
    for a_recording in *possible_recordings{
        let confidence = a_recording.attributes["score"].parse::<i32>().unwrap();
        if confidence < min_confidence{
            debug!("Skipping possible recording. Not enough confidence");
            continue;
        }

        //Try to find a genre before outright rejectig this recording
        if a_recording.has_matching_child("tag-list"){
            let possible_genres = &a_recording["tag-list"].get_matching_children("tag");
            
            for a_genre in *possible_genres{
                let votes = a_genre.attributes["count"].parse::<i32>().unwrap();
                if votes > tag_count{
                    tag_count = votes;
                    best_candidate.genre = (&a_genre["name"]).value.clone();
                }
            }
        }

        let release = &a_recording["release-list"]["release"];
        if !release.has_matching_child("date"){
            debug!("Skipping possible recording. Date tag not found!");
            continue;
        }

        let release_year = (&release["date"].value)[..4].parse::<i32>().unwrap();
        //Replace the candidate data if it's older
        if release_year > 1800 && release_year < best_candidate.date{
            best_candidate.date = release_year;
            best_candidate.album = (&release["title"]).value.clone();
            best_candidate.track_number = (&release["medium-list"]["medium"]["track-list"]["track"]["number"])
                .value.parse::<u32>().unwrap();
        }
    }

    if best_candidate.date == 9001{
        return Err(Error::new(ErrorKind::NotFound, "Recording metadata was not found!"));
    }

    Ok(best_candidate)
}

fn determine_artist_id(song_file: &SongFile) -> io::Result<String>{
    let mut artist_request = String::new();
    artist_request.push_str("artist?query=");
    let sanitized_artist_name = percent_encode(song_file.metadata.artist().unwrap()).unwrap();
    artist_request.push_str(&sanitized_artist_name);
    debug!("Request-url:{}",artist_request.as_str());

    let curly_result = make_get_request(API_ENDPOINT, &artist_request).unwrap();
    let artist_id = find_artist_id(curly_result.as_str())?;

    Ok(artist_id)
}

fn check_artist_recording(song_file: &SongFile, artist_id: String) -> io::Result<BasicMetadata>{
    //Query for the Song name
    let mut song_request = String::new();
    song_request.push_str("recording/?query=");
    song_request.push_str(percent_encode(song_file.metadata.title().unwrap()).unwrap().as_str());
    song_request.push_str(percent_encode(" AND arid%3A").unwrap().as_str()); //"%20AND%20arid%3A"
    song_request.push_str(artist_id.as_str());
    debug!("Request-url:{}",song_request.as_str());
    let request_result = make_get_request(&API_ENDPOINT, &song_request).unwrap();

    let basic_metadata = find_recording_data(request_result.as_str())?;
    Ok(basic_metadata)
}

pub fn check(song_file: &mut SongFile) -> io::Result<()>{
    info!("Checking '{} - {}'", song_file.metadata.artist().unwrap(), song_file.metadata.title().unwrap());

    let artist_id = determine_artist_id(song_file)?;
    warn!("{} - Artist ID={}",song_file.metadata.artist().unwrap(), artist_id);

    let recording_metadata = check_artist_recording(song_file, artist_id)?;
    song_file.set_basic_metadata(recording_metadata);
    warn!("{}",song_file);
    // song_file.save();

    // Ok(())
    Err(Error::new(ErrorKind::Other, "This isn't implemented."))
}