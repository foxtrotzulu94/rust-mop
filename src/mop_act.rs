//Actions of the program as simple, public functions

use std::io;
use std::fs::{self, DirEntry};
use std::path::Path;
use std::string::String;
use std::collections::HashMap;

fn visit_dirs(dir: &Path) -> io::Result<()> {
    info!("{}",dir.display());
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path)?;
            } else {
                info!("{}",path.display());
            }
        }
    }
    Ok(())
}

pub fn quick_check(curr_dir: String){
    info!("Doing a quick check");
    visit_dirs(&Path::new(curr_dir.as_str()));

}