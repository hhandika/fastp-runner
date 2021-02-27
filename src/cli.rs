use std::path::PathBuf;

use clap::{App, AppSettings, Arg, ArgMatches};

use crate::io;
use crate::wrapper;

pub fn get_cli(version: &str) {
    let args = App::new("renamer")
        .version(version)
        .about("Batch adapter cleaning using fastp")
        .author("Heru Handika <hhandi1@lsu.edu>")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            App::new("check")
                .help("Check the status of fastp")
                .about("Check if fastp is installed")
            )

        .subcommand(
            App::new("clean")
                .about("Uses for adapter cleaning")
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
                        .help("A unique ID is at the start of filename")
                        .takes_value(false)
                )
                
                .arg(
                    Arg::with_name("dry-run")
                        .long("dry")
                        .help("Check if the program detect the correct files")
                        .takes_value(false)
                )
        )
        
        .get_matches();

    match args.subcommand() {
        ("clean", Some(clean_matches)) => run_fastp_clean(clean_matches, version),
        ("check", Some(_)) => wrapper::check_fastp(),
        _ => unreachable!("UNREACHABLE COMMANDS!"),
    };
}

fn run_fastp_clean(matches: &ArgMatches, version: &str) {
    if matches.is_present("input") {
        let path = PathBuf::from(matches.value_of("input").unwrap());
        let mut is_id = false;

        if matches.is_present("id") {
            is_id = true;
        }

        if matches.is_present("dry-run") {
            io::dry_run(&path, is_id);
        } else {
            io::process_input(&path, is_id, version);
        }
    } 
}