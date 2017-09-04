#[macro_use]
extern crate log;
extern crate env_logger;
extern crate chrono;

mod mop_structs;

use chrono::prelude::*;

use std::env;
use log::{LogRecord, LogLevelFilter};
use env_logger::LogBuilder;
use mop_structs::Metadata as Metadata;

fn init_logging(){
    let format = |record: &LogRecord| {
        let dt = Local::now();
        format!("{} {} [{}]: {}", dt.format("%Y-%m-%d %H%M%S").to_string(), record.location().file(), record.level(), record.args())
    };

    let mut builder = LogBuilder::new();
    builder.format(format).filter(None, LogLevelFilter::Info);

    //If building in debug, always check
    if cfg!(debug_assertions) {
        builder.parse("debug");
    }  else if env::var("MOP_LOG").is_ok() {
        builder.parse(&env::var("MOP_LOG").unwrap());
    }

    builder.init().unwrap();
}

fn main(){
    init_logging();
    info!("starting up");

    let testy: Metadata = mop_structs::new_metadata();
    debug!("testy.x={}", testy.is_complete);

    // Test to see that we can build
    let line_args = env::args().skip(1);
    
    for argument in line_args {
        println!("{}", argument);
    }
}