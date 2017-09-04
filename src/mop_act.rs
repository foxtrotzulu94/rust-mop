//Actions of the program as simple, public functions

extern crate id3;

use mop_structs;

use std::io;
use std::fs::{self, DirEntry};
use std::path::Path;
use std::string::String;
use std::collections::HashMap;

struct FileCount{
    num: i32,
    bytes: u64,
}

///Checks whether the given extension is acceptable by MOP
fn is_audio_extension(ext: &str) -> bool{
    let ret_val = match ext {
        "mp3"|"ogg" => true,
        "aac"|"mp4" => {
            warn!("Extension not yet supported");
            false
        }
        _ => false,
    };

    return ret_val;
}

fn visit_dirs(dir: &Path) -> io::Result<(HashMap<String,FileCount>)> {
    let mut count : HashMap<String,FileCount> = HashMap::new();

    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let subcount = visit_dirs(&path)?;
                for (key,val) in subcount.iter() {
                    let ext_count = count.entry((*key).clone()).or_insert(FileCount{num: 0, bytes:0});
                    (*ext_count).num += (*val).num;
                    (*ext_count).bytes += (*val).bytes;
                }
            } else {
                //info!("{}",path.display());
                //info!("{}",path.extension().unwrap().to_str().unwrap().to_string().to_lowercase())
                let safe_ext = path.extension().unwrap().to_str().unwrap().to_string().to_lowercase();
                if is_audio_extension(safe_ext.as_str()) {
                    let ext_count = count.entry(safe_ext).or_insert(FileCount{num: 0, bytes:0});
                    (*ext_count).num+=1;
                    (*ext_count).bytes+= entry.metadata().unwrap().len();
                }
            }
        }
    }
    Ok((count))
}

pub fn quick_check(curr_dir: String){
    info!("Doing a quick check of {}",curr_dir);

    //FIXME: On windows, canonicalize returns "\\?\" (UNC Path)
    let cleaned_path = fs::canonicalize(curr_dir.as_str()).unwrap();
    let working_path = cleaned_path.as_path();

    let count = visit_dirs(working_path).unwrap();
    let base : f64 = 1024.0;
    let MB = base.powi(2);

    let mut total_size : u64 = 0;
    let mut total_num = 0;
    for (key,val) in count.iter(){
        info!("{:<3}: files={: <6} \t size={:.2}MB", key, val.num, val.bytes as f64/MB);
        total_num+=val.num;
        total_size+=val.bytes;
    }

    info!("TOTAL: {} files - {:.2} MB",total_num, total_size as f64 / MB);
}

pub fn fix_metadata(working_dir: String){
    //TODO:
    // For every dir, filter music files, then fix the metadata on music files

    for entry in fs::read_dir(working_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let safe_ext = path.extension().unwrap().to_str().unwrap().to_string().to_lowercase();
        
        if !path.is_dir() && is_audio_extension(safe_ext.as_str()) {
            //Do one and then quit
            let /*mut*/ song_file = mop_structs::SongFile::make(path.as_path());
            // song_file.metadata.set_genre("R&B");
            // song_file.save();
            info!("\n{}",song_file);
            info!("Is complete: {}",song_file.is_metadata_complete());
            break;
        }
    }
}