//All the functions needed to use the MusicBrainz API (https://musicbrainz.org/doc/Development/XML_Web_Service/Version_2)
use mop_structs::{SongFile,BasicMetadata};
use mop_online::{make_get_request, percent_encode};
use xml_wrap::{XmlMap};

use std::{str,time,thread};
use std::io::{Error, ErrorKind, self};

static API_ENDPOINT: &'static str = "https://musicbrainz.org/ws/2/";

//TODO: Better error handling (i.e. away from std::io::Result)

fn find_artist_id(raw_data: &str) -> io::Result<String>{
    let mut artist_id = String::new();
    let mut confidence = 75;

    let data = XmlMap::from_str(raw_data);
    let possible_artists = &data.root["metadata"]["artist-list"].get_matching_children("artist");

    if possible_artists.len() < 1{
        return Err(Error::new(ErrorKind::NotFound, "MusicBrainz: The Artist was not found"));
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
        return Err(Error::new(ErrorKind::NotFound, "MusicBrainz: The Artist ID was not found with enough confidence!"));
    }
    //FIXME: Partially Uppercase the Genre
    Ok(artist_id)
}

fn find_recording_data(raw_data: &str) -> io::Result<BasicMetadata>{
    let mut temp_metadata = BasicMetadata::new();
    let mut recording_list : Vec<BasicMetadata> = Vec::new();
    
    let data = XmlMap::from_str(raw_data);
    let recording_list = &data.root["metadata"]["recording-list"];
    let num_recordings = recording_list.attributes["count"].parse::<i32>().unwrap();
    if num_recordings < 1{
        return Err(Error::new(ErrorKind::NotFound, "MusicBrainz: No matching recordings were found"));
    }

    //Select best one (by default, go for first release ever)
    let possible_recordings = &recording_list.get_matching_children("recording");
    let mut best_candidate = BasicMetadata::new();
    best_candidate.date = 9001;
    let min_confidence = 75;
    let mut tag_count = 0;
    for a_recording in *possible_recordings{
        let confidence = a_recording.attributes["score"].parse::<i32>().unwrap();
        let recording_id = &a_recording.attributes["id"];
        if confidence < min_confidence{
            info!("MusicBrainz: Skipping possible recording. Not enough confidence");
            info!("Rec ID: {}", recording_id);
            continue;
        }
        if !a_recording.has_matching_child("release-list") || !a_recording["release-list"].has_matching_child("release"){
            //This is more of an error with MusicBrainz
            info!("MusicBrainz: Skipping recording that does not have releases");
            info!("Rec ID: {}", recording_id);
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
            info!("MusicBrainz: Skipping possible recording. Date tag not found!");
            info!("Rec ID: {}", recording_id);
            continue;
        }
        let release_status = &release["status"].value.clone();
        if release_status != "Official"{
            info!("MusicBrainz: Skipping bootleg recording");
            info!("Rec ID: {}", recording_id);
            continue;
        }

        let release_year = (&release["date"].value)[..4].parse::<i32>().unwrap();
        //Replace the candidate data if it's older
        if release_year > 1800 && release_year < best_candidate.date{
            best_candidate.date = release_year;
            best_candidate.album = (&release["title"]).value.clone();
            let track_number = (&release["medium-list"]["medium"]["track-list"]["track"]["number"])
                .value.parse::<u32>();
            best_candidate.track_number = read_result!(track_number,0);
        }
    }

    if best_candidate.date == 9001{
        return Err(Error::new(ErrorKind::NotFound, "MusicBrainz: Recording metadata was not found!"));
    }

    Ok(best_candidate)
}

fn determine_artist_id(song_file: &SongFile) -> io::Result<String>{
    let mut artist_request = String::new();
    artist_request.push_str("artist?query=");
    let mut sanitized_artist_name = percent_encode(
        song_file.metadata.artist().unwrap()
        ).unwrap();
    artist_request.push_str(&sanitized_artist_name);
    debug!("Request-url:{}",artist_request.as_str());

    let curly_result = make_get_request(API_ENDPOINT, &artist_request)?;
    let artist_id = find_artist_id(curly_result.as_str())?;

    Ok(artist_id)
}

fn check_artist_recording(song_file: &SongFile, artist_id: String) -> io::Result<BasicMetadata>{
    //Query for the Song name
    let mut song_request = String::new();
    song_request.push_str("recording/?query=");
    song_request.push_str(percent_encode(song_file.metadata.title().unwrap()).unwrap().as_str());
    song_request.push_str(percent_encode(" AND arid:").unwrap().as_str()); //"%20AND%20arid%3A"
    song_request.push_str(artist_id.as_str());
    debug!("Request-url:{}",song_request.as_str());
    let request_result = make_get_request(&API_ENDPOINT, &song_request)?;

    let basic_metadata = find_recording_data(request_result.as_str())?;
    Ok(basic_metadata)
}

pub fn check(song_file: &mut SongFile) -> io::Result<()>{
    info!("MusicBrainz: Checking '{} - {}'", song_file.metadata.artist().unwrap(), song_file.metadata.title().unwrap());
    
    //MusicBrainz requires us to emit less than 1 request per second (on average)
    // To do so, we have to sleep after each of these methods are called
    let artist_id = determine_artist_id(song_file)?;
    thread::sleep(time::Duration::from_millis(750));
    debug!("MusicBrainz: {} - Artist ID={}",song_file.metadata.artist().unwrap(), artist_id);

    let recording_metadata = check_artist_recording(song_file, artist_id)?;
    thread::sleep(time::Duration::from_millis(250));
    song_file.set_basic_metadata(recording_metadata);
    debug!("MusicBrainz: {}",song_file);
    song_file.save();

    Ok(())
}