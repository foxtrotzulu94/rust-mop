//All the functions needed to scrape AllMusic (http://www.allmusic.com/)
use mop_structs::SongFile;

use std::io::{Error, ErrorKind, self};
use std::str;
use curl::easy::Easy;
use xml::reader::{EventReader, XmlEvent};

pub fn check(song_file: &mut SongFile) -> io::Result<()>{
    Err(Error::new(ErrorKind::Other, "AllMusic: This isn't implemented."))
}