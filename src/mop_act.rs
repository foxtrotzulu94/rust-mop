//Actions of the program as simple, public functions

use std::io;
use std::fs::{self, DirEntry};
use std::path::Path;
use std::string::String;
use std::collections::HashMap;

struct FileCount{
    num: i32,
    bytes: u64,
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
                let ext_count = count.entry(safe_ext).or_insert(FileCount{num: 0, bytes:0});
                (*ext_count).num+=1;
                (*ext_count).bytes+= entry.metadata().unwrap().len();
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

    for (key,val) in count.iter(){
        info!("{}: files={: <6} \t size={:.2}MB", key, val.num, val.bytes as f64/MB);
    }
}