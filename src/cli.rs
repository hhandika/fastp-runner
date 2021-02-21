use std::path::PathBuf;

use clap::{App, AppSettings, Arg};

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
                        // .conflicts_with_all(&[ "check"])
                        .takes_value(true)
                        .value_name("INPUT")
                )  
        )

        .subcommand(
            App::new("dry")
                .about("Uses for adapter cleaning")
                .arg(
                    Arg::with_name("input")
                        .short("i")
                        .long("input")
                        .help("Inputs a config file")
                        // .conflicts_with_all(&[ "check"])
                        .takes_value(true)
                        .value_name("INPUT")
                )
             
        )
        
        .get_matches();

    match args.subcommand() {
        ("clean", Some(clean_matches)) => {

            if clean_matches.is_present("input") {
                    let path = PathBuf::from(clean_matches.value_of("input").unwrap());
                    let adapter = "CAAGCAGAAGACGGCATACGAGATGCCATAGGTGACTGGAGTTCAGACGTGT";
                    wrapper::run_fastp(&path, &adapter);
            }  

        }

        ("check", Some(_)) => {
            wrapper::check_fastp();
        }

        ("dry", Some(test_matches)) => {
            if test_matches.is_present("input") {
                let path = PathBuf::from(test_matches.value_of("input").unwrap());
                io::dry_run(&path);
            }
        }
        _ => unreachable!("UNREACHABLE COMMANDS!"),
    };
}