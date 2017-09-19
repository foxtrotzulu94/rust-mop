//Actions of the program as simple, public functions

use mop_online::retrieve_metadata_online;
use mop_structs::SongFile;

use std::io;
use std::fs;
use std::error::Error;
use std::path::{Path,PathBuf};
use std::string::String;
use std::collections::HashMap;

struct FileCount{
    num: i32,
    bytes: u64,
    incomplete_metadata_count: i32,
    invalid_key_count: i32,
}

///Checks whether the given extension is acceptable by MOP
fn is_audio_extension(ext: &str) -> bool{
    let ret_val = match ext {
        "mp3" => true,
        "aac"|"mp4"|"ogg" => {
            warn!("Extension not yet supported: {}",ext);
            false
        }
        _ => false,
    };

    return ret_val;
}

///Visits a path recursively and calls `func` on each of the files
fn visit_path(path: &Path, func: &mut FnMut(&Path)) -> io::Result<()> {
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_path(&path, func)?;
            } else {
                func(&path);
            }
        }
    }

    Ok(())
}

///Builds the songlist by using `func` to walk directories
fn build_song_list(working_dir: String, func: &mut FnMut(&Path, &mut Vec<SongFile>)) -> Vec<SongFile>{
    let mut song_list : Vec<SongFile> = Vec::new();

    // static mut CACHE_DIR: &str = "";
    // static mut CACHE_RES: &'static [SongFile] = &[];
    // unsafe{
    //     if CACHE_DIR == working_dir.as_str(){
    //         song_list.extend_from_slice(&CACHE_RES);
    //         return song_list;
    //     }
    // }

    let cleaned_path = fs::canonicalize(working_dir.as_str()).unwrap();
    let working_path = cleaned_path.as_path();
    // let working_path = Path::new(&working_dir);

    //Build the list of songs
    {
        //NOTE: Borrow song_list for a short duration and then give it back at the end of the scope
        let mut song_accumulator = |some_path : &Path| {
            func(some_path, &mut song_list);
        };

        match visit_path(working_path, &mut song_accumulator){
            Ok(()) => info!("Finished scanning directories"),
            Err(e) => error!("{}",e),
        };
    }

    //TODO: cache, somehow...
    song_list
}

///Common function for accumulating songs
fn song_accumulator(some_path : &Path, song_list : &mut Vec<SongFile>) {
    if some_path.is_dir(){
        return;
    }
    let safe_ext = some_path.extension().unwrap().to_str().unwrap().to_string().to_lowercase();
    if is_audio_extension(safe_ext.as_str()) {
        let song_file = SongFile::make(some_path);
        if !song_file.has_search_key(){
            error!("No Title/Artist for {}", song_file.get_filepath_str().unwrap());
        } else {
            song_list.push(song_file);
        }
    }
}

///Perform a quick check of the path and the audio files contained.
pub fn path_check(curr_dir: String){
    info!("Doing a quick check of {}",curr_dir);

    //FIXME: On windows, canonicalize returns "\\?\" (UNC Path)
    let cleaned_path = fs::canonicalize(curr_dir.as_str()).unwrap();
    let working_path = cleaned_path.as_path();

    let mut file_collection : HashMap<String,FileCount> = HashMap::new();
    {
        let mut collector = |some_path : &Path| {
            if some_path.is_dir(){
                return;
            }
            let safe_ext = some_path.extension().unwrap().to_str().unwrap().to_string().to_lowercase();
            if is_audio_extension(safe_ext.as_str()) {
                let song_file = SongFile::make(some_path);
                if !song_file.has_search_key(){
                    error!("Title/Artist Missing: {}", song_file.get_filepath_str().unwrap());
                }

                let ext_count = file_collection.entry(safe_ext).or_insert(
                    FileCount{num: 0, bytes:0, incomplete_metadata_count: 0, invalid_key_count: 0});
                (*ext_count).num+=1;
                (*ext_count).bytes+= fs::metadata(some_path).unwrap().len();
                if !song_file.has_search_key(){
                    (*ext_count).invalid_key_count += 1;
                }
                if !song_file.is_metadata_complete(){
                    (*ext_count).incomplete_metadata_count+=1;
                }
            }
        };

        match visit_path(working_path, &mut collector){
            Ok(()) => info!("Finished scanning directories"),
            Err(e) => error!("{}",e),
        };
    }

    let count = file_collection;
    let base : f64 = 1024.0;
    let mb = base.powi(2);

    let mut file_count = FileCount{num: 0, bytes:0, incomplete_metadata_count: 0, invalid_key_count: 0};
    println!("\nFILES:");
    for (key,val) in count.iter(){
        println!(".{:<3}: files={: <6} size={:.2}MB", key, val.num, val.bytes as f64 / mb);
        println!("\tFiles without Artist-Title: {:<6} Files with incomplete tags: {:<6}", val.invalid_key_count, val.incomplete_metadata_count);
        file_count.num += val.num;
        file_count.bytes += val.bytes;
        file_count.incomplete_metadata_count = val.incomplete_metadata_count;
        file_count.invalid_key_count = val.invalid_key_count;
    }

    println!("\nTOTAL: {} files - {:.2} MB", file_count.num, file_count.bytes as f64 / mb);
    println!("\tFiles without Artist-Title: {:<6} Files with incomplete tags: {:<6}", 
        file_count.invalid_key_count, file_count.incomplete_metadata_count);
}

///Write online metadata for incomplete files in `working_dir`
pub fn fix_metadata(working_dir: String){
    let mut search_function = song_accumulator;
    let song_list = build_song_list(working_dir, &mut search_function);

    //Testing that this works
    let mut unchanged_files : Vec<(SongFile,String)> = Vec::new();
    for mut a_song in song_list{
        if !a_song.is_metadata_complete(){
            match retrieve_metadata_online(&mut a_song){
                Err(e) => {
                    //Do we STILL have incomplete metadata?
                    if !a_song.is_metadata_complete(){
                        error!("Metadata still incomplete for '{} - {}'", 
                            a_song.metadata.artist().unwrap(), a_song.metadata.title().unwrap());
                        unchanged_files.push((a_song, String::from(e.description())));
                    }
                },
                _ => info!("SUCCESS"),
            };

        } else {
            info!("Skipping '{} - {}' as complete", a_song.metadata.artist().unwrap(), a_song.metadata.title().unwrap());
        }
    }

    //Status report
    let num_unchanged_files = unchanged_files.len();
    println!("\nAutomated fix completed - Pay attention below for unchanged/problematic files\n");
    for (failed_file, reason) in unchanged_files{
        println!("File '{}'", failed_file.get_filepath_str().unwrap());
        println!("Reason: {}", reason);
    }

    println!("{} files unchanged after fix", num_unchanged_files);
}

pub fn get_cover_art(working_dir: String){
    error!("Cover art retrieval is not yet implemented!");
}

pub fn bulk_rename(working_dir: String, format_string: String){
    let song_list = build_song_list(working_dir.clone(), &mut song_accumulator);

    let mut format_path = working_dir.clone();
    match &working_dir.as_str()[..1]{
        "/" => (),
        _ => format_path.push_str("/"),
    }
    //Treat the './' as the working directory    
    format_path = format_string.replace("./",format_path.as_str());

    //Just rename the files as they appear
    //FIXME: This is horribly inefficient...
    let build_new_path = |a_song: &SongFile| -> PathBuf{
        let mut new_name = format_path.clone()
                .replace("%artist", a_song.metadata.artist().unwrap())
                .replace("%title", a_song.metadata.title().unwrap())
                .replace("%album", a_song.metadata.album().unwrap_or("N/A"))
                .replace("%year", &a_song.metadata.year().unwrap_or(1969).to_string())
                .replace("%track", &a_song.metadata.track().unwrap_or(00).to_string());
        new_name.push_str(".");
        new_name.push_str(a_song.extension.as_str());
        return PathBuf::from(new_name);
    };

    info!("Renaming with pattern {}", format_string);
    let mut successful_renames = 0;
    let num_songs = song_list.len();
    for a_song in song_list{
        if a_song.has_search_key(){
            let result = fs::rename(
                a_song.get_filepath(),
                build_new_path(&a_song).as_path()
            );
            match result {
                Ok(_) => successful_renames+=1,
                Err(e) => {
                    error!("{}",a_song.get_filepath_str().unwrap());
                    error!("{}",e);
                },
            }
        }
        else{
            error!("Did not rename {}", a_song.get_filepath_str().unwrap());
        }
    }

    println!("\nSuccessfully renamed {}/{} files", successful_renames, num_songs);
}

pub fn do_all(working_dir: String){
    //Do them all
    path_check(working_dir.clone());
    fix_metadata(working_dir.clone());
    get_cover_art(working_dir.clone());
    bulk_rename(working_dir, String::from("./%artist - %title"));
}