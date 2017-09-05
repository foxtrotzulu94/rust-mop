//Basic data structures and their method implementations

extern crate id3;

use std::fmt;
use std::string::String;
use std::time::Duration;
use std::path::{Path,PathBuf};

macro_rules! safe_expand_tag {
    ($x:expr, $y:expr) => {
        match $x{
            None => $y,
            Some(value) => value,
        };
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
        self.metadata.write_to_path(self.file_path.as_path());
    }

    pub fn is_metadata_complete(&self) -> bool{
        //The important fields are: Title, Artist, Genre and Year
        let tag = &self.metadata;
        let year = safe_expand_tag!(tag.year(), 0);
        let genre = safe_expand_tag!(tag.genre(), "");
        let album = safe_expand_tag!(tag.album(), "");
        let artist = safe_expand_tag!(tag.artist(), "");
        let title = safe_expand_tag!(tag.title(), "");

        return !artist.is_empty()
            && !title.is_empty()
            && !genre.is_empty() && !(genre.contains("(") || genre.contains(")"))
            && year>1800; //Reasonably enough, I wouldn't catalogue pre-1800 music
    }
}

impl fmt::Display for SongFile {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tag = &self.metadata;
        write!(f, "\nTitle: {}\nArtist: {}\nAlbum: {}\nGenre: {}\nYear: {}\nPath:{}", 
            safe_expand_tag!(tag.title(), "N/A"), 
            safe_expand_tag!(tag.artist(), "N/A"), 
            safe_expand_tag!(tag.album(), "N/A"),
            safe_expand_tag!(tag.genre(), "N/A"),
            safe_expand_tag!(tag.year(), 0), 
            self.file_path.display())
    }
}

