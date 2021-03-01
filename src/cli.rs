use std::path::PathBuf;

use clap::{App, AppSettings, Arg, ArgMatches};

use crate::io;
use crate::cleaner;

pub fn get_cli(version: &str) {
    let args = App::new("renamer")
        .version(version)
        .about("Batch adapter trimming and raw-read sequence cleaning using fastp")
        .author("Heru Handika <hhandi1@lsu.edu>")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            App::new("check")
                .about("Checks if fastp is installed")
            )

        .subcommand(
            App::new("clean")
                .about("Runs fastp")
                .arg(
                    Arg::with_name("input")
                        .short("i")
                        .long("input")
                        .help("Inputs a config file")
                        .takes_value(true)
                        .value_name("INPUT")
                )
                
                .arg(
                    Arg::with_name("id")
                        .long("id")
                        .help("Uses id instead of filenames")
                        .takes_value(false)
                )
                
                .arg(
                    Arg::with_name("dry-run")
                        .long("dry")
                        .help("Checks if the program detect the correct files")
                        .takes_value(false)
                )

                .arg(
                    Arg::with_name("rename")
                        .long("rename")
                        .help("Checks if the program detect the correct files")
                        .takes_value(false)
                )
        )
        
        .get_matches();

    match args.subcommand() {
        ("clean", Some(clean_matches)) => run_fastp_clean(clean_matches, version),
        ("check", Some(_)) => cleaner::check_fastp(),
        _ => (),
    };
}

fn run_fastp_clean(matches: &ArgMatches, version: &str) {
    if matches.is_present("input") {
        let path = PathBuf::from(matches.value_of("input").unwrap());
        let mut is_id = false;
        let mut is_rename = false;

        if matches.is_present("id") {
            is_id = true;
        }

        if matches.is_present("rename") {
            is_rename = true;
        }

        if matches.is_present("dry-run") {
            io::dry_run(&path, is_id, is_rename);
        } else {
            println!("Starting fastp-runner v{}...\n", version);
            io::process_input(&path, is_id, is_rename);
        }
    } 
}