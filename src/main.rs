#[macro_use]
extern crate log;
extern crate id3;
extern crate env_logger;
extern crate chrono;
extern crate clap;
extern crate curl;
extern crate xml;
extern crate url;
extern crate select;

#[macro_use]
pub mod mop_macro{
    #[macro_export]
    macro_rules! read_result {
        ($x:expr, $y:expr) => {
            match $x{
                Err(e) => {
                    error!("{}",e);
                    $y
                },
                Ok(value) => value,
            };
        }
    }
    
    #[macro_export]
    macro_rules! safe_expand {
        ($x:expr, $y:expr) => {
            match $x{
                None => $y,
                Some(value) => value,
            };
        }
    }

    macro_rules! replace_all_for_one {
        ($string:expr, $sub:expr, $($x:expr),+) => {
            {
                let mut ret_val : String = String::from($string);
                $(
                    ret_val = ret_val.replace($x, $sub);
                )+

                ret_val
            }
        };
    }

    macro_rules! ntfs_safename {
        ($s:expr) => {
            replace_all_for_one!($s," ",
                "/","\\",":","|","<",">","?","*","\"");
        }
    }
}

mod src_music_brainz;
mod src_allmusic;
mod xml_wrap;
mod mop_structs;
mod mop_online;
mod mop_act;

use log::{LogRecord, LogLevelFilter};
use env_logger::LogBuilder;
use chrono::prelude::*;
use clap::{Arg, App, SubCommand};

use xml_wrap::XmlMap;

fn init_logging(log_level: &str){
    let format = |record: &LogRecord| {
        let dt = Local::now();
        format!("{} [{}]: {}", dt.format("%Y-%m-%d %H%M%S").to_string(), record.level(), record.args())
    };

    let mut builder = LogBuilder::new();
    builder.format(format).filter(None, LogLevelFilter::Info);
    
    builder.parse(log_level);
    builder.init().unwrap();
}

fn test_wrap(){
    let testy = r#"</ ? < > \ : * | " and any character you can type with the Ctrl key"#;
    let sanitized = ntfs_safename!(testy);
    println!("{}", sanitized)
}

fn main(){
    //Parse command line args and check the sub command
    let args = App::new("MOP - Metadata Organization Program")
                        .version(env!("CARGO_PKG_VERSION"))
                        .author(env!("CARGO_PKG_AUTHORS"))
                        .about("Cleans up messy music files!")
                        .subcommand(SubCommand::with_name("all")
                                    .about("Perform all operations available (check->fix->art->rename)"))                       
                        .subcommand(SubCommand::with_name("check")
                                    .about("Verify the given directory and print info about it"))
                        .subcommand(SubCommand::with_name("fix")
                                    .about("Do a full fix of all file metadata")
                                    // .arg(Arg::with_name("genre")
                                    //     .short("g")
                                    //     .long("genre")
                                    //     .value_name("GENRE")
                                    //     .help("Fix Genre Metadata as well")
                                    //     .required(false)
                                    //     .takes_value(false))
                                    )
                        .subcommand(SubCommand::with_name("cover-art")
                                    .about("Retrieve the cover art for all file, if possible"))
                        .subcommand(SubCommand::with_name("rename")
                                    .about("Rename files to a specifc format")
                                    .arg(Arg::with_name("format")
                                        .short("f")
                                        .long("format")
                                        .help("Use a specific renaming format (by default './%artist - %title') ")
                                        .required(false)
                                        .takes_value(true)))
                        .subcommand(SubCommand::with_name("dev")
                                    .about("DevCommand :)"))
                        .arg(Arg::with_name("directory")
                            .short("d")
                            .long("working-dir")
                            .value_name("PATH")
                            .help("Set the working directory of the program")
                            .required(true)
                            .takes_value(true))
                        .arg(Arg::with_name("log")
                            .short("v")
                            .long("log")
                            .value_name("debug|info|warn|error")
                            .help("Set the logging verbosity of the program (Default: 'warn')")
                            .takes_value(true))
                        .get_matches();

    init_logging(args.value_of("log").unwrap_or("warn"));
    info!("Starting up MOP");
    let working_directory = String::from(args.value_of("directory").unwrap());
    info!("Working Directory: {}",working_directory);

    match args.subcommand_name() {
        Some("all")     => mop_act::do_all(working_directory),
        Some("check")   => mop_act::path_check(working_directory),
        Some("fix")     => mop_act::fix_metadata(working_directory),
        Some("cover-art")     => mop_act::get_cover_art(working_directory),
        Some("rename")  => {
            let format_string = String::from(args.value_of("format").unwrap_or("./%artist - %title"));
            mop_act::bulk_rename(working_directory, format_string);
        },
        Some("dev")     => test_wrap(),
        None        => panic!("No subcommand was used - Not supported yet!"),
        _           => panic!("The subcommand that was used that is not supported yet"),
    }
}