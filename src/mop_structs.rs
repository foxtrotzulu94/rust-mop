//Basic data structures and their method implementations

extern crate id3;

use id3::{Tag, Frame};
use id3::frame::Content;

use std::fmt;
use std::string::String;
use std::time::Duration;
use std::path::{Path,PathBuf};

use mop_online::get_user_agent;

pub struct BasicMetadata{
    //The assumption is that the program is using title+artist as a key in all lookups
    // Therefore it must be correct to begin with and must not be changed at all!
    // title: String,
    // artist: String,
    pub genre: String,
    pub album: String,
    pub track_number: u32,
    pub date: i32,
    pub composer: String,
}

impl BasicMetadata{
    pub fn new() -> BasicMetadata{
        return BasicMetadata{
            genre : String::new(),
            album : String::new(),
            track_number : 0,
            date : 0,
            composer : String::new(),
        }
    }

    pub fn has_some_data(&self) -> bool{
        return !self.genre.is_empty() || !self.album.is_empty() || !self.composer.is_empty() 
            || self.track_number > 0 || self.date > 1800;
    }
}

impl fmt::Display for BasicMetadata {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\nAlbum: {} #({})\nGenre: {}\nYear: {}\n",
            self.album,
            self.track_number,
            self.genre,
            self.date
        )
    }
}

pub struct SongFile{
    pub metadata: id3::Tag,
    extension: String,
    file_path: PathBuf,
}

impl SongFile{
    pub fn make(file_path : &Path) -> SongFile{
        //Build metadata first
        let tag = id3::Tag::read_from_path(file_path).unwrap();
        let song = SongFile{
            metadata: tag, 
            extension: file_path.extension().unwrap().to_str().unwrap().to_string().to_lowercase(),
            file_path: PathBuf::from(file_path),
            };
        
        return song;
    }

    pub fn save(&mut self){
        let result = self.metadata.write_to_path(self.file_path.as_path());
        //FIXME: Sometimes this returns "Permission Denied"
        //  It's very likely to be an error with the library
        match result {
            Ok(ok) => debug!("Tag saved correctly for {}",self.file_path.to_str().unwrap()),
            Err(e) => {
                error!("FATAL: {}",e);
                error!("{}",self);
                panic!("HALT");
            },
        }
        // assert!(result.is_ok());
    }

    pub fn has_search_key(&self) -> bool{
        return !(
            safe_expand!(self.metadata.artist(), "").is_empty() 
            || safe_expand!(self.metadata.title(), "").is_empty()
            );
    }

    pub fn is_metadata_complete(&self) -> bool{
        //The important fields are: Title, Artist, Genre and Year
        let tag = &self.metadata;
        let mut year = safe_expand!(tag.year(), 0);
        if year == 0{
            //If it doesn't have this tag, panic
            let some_date = safe_expand!(tag.date_recorded(), id3::Timestamp::parse("0").unwrap());
            year = safe_expand!(some_date.year,0) as usize;
        }

        let album = safe_expand!(tag.album(), "");
        let genre = safe_expand!(self.metadata.genre(), "");
        let artist = safe_expand!(tag.artist(), "");
        let title = safe_expand!(tag.title(), "");

        return false;

        return !artist.is_empty()
            && !title.is_empty()
            && !album.is_empty()
            && year>1800; //Reasonably enough, I wouldn't catalogue pre-1800 music
    }

    pub fn has_genre(&self) -> bool{
        let genre = safe_expand!(self.metadata.genre(), "");
        return !genre.is_empty() && !(genre.contains("(") || genre.contains(")"));
    }

    pub fn get_filepath_str(&self) -> Option<&str>{
        return self.file_path.to_str();
    }

    pub fn set_basic_metadata(&mut self, ext_data : BasicMetadata){
        let mut metadata = &mut self.metadata;
        metadata.set_album(ext_data.album);
        metadata.set_year(ext_data.date as usize);
        metadata.set_genre(ext_data.genre);
        metadata.set_track(ext_data.track_number);

        //TODO: Add composer block

        //Loose strings here
        let album_artist = String::from(metadata.artist().unwrap());
        metadata.set_album_artist(album_artist);
    }
}

impl fmt::Display for SongFile {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tag = &self.metadata;
        write!(f, "Path:{}\nTitle: {}\nArtist: {}\nAlbum: {}\nGenre: {}\nYear: {}", 
            self.file_path.display(),
            safe_expand!(tag.title(), "N/A"), 
            safe_expand!(tag.artist(), "N/A"), 
            safe_expand!(tag.album(), "N/A"),
            safe_expand!(tag.genre(), "N/A"),
            safe_expand!(tag.year(), 0), 
            )
    }
}

