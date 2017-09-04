//Basic data structures and their method implementations

extern crate id3;

use std::fmt;
use std::string::String;
use std::time::Duration;
use std::path::{Path,PathBuf};

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
        let artist = tag.artist().unwrap();
        let title = tag.title().unwrap();
        let genre = tag.genre().unwrap();
        let year = tag.year().unwrap();

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
        write!(f, "Title: {}\nArtist: {}\nAlbum: {}\nGenre: {}\nYear: {}\nPath:{}", 
            tag.title().unwrap(), 
            tag.artist().unwrap(), 
            tag.album().unwrap(),
            tag.genre().unwrap(),
            tag.year().unwrap(), 
            self.file_path.display())
    }
}

