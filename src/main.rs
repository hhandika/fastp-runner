// Heru Handika
// February 2021
// MIT

mod cli;
mod io;
mod parser;
mod runner;
mod tag;
mod utils;

use std::time::Instant;

use clap::crate_version;

fn main() {
    let version = crate_version!();
    let time = Instant::now();
    cli::get_cli(&version);
    let duration = time.elapsed();

    if duration.as_secs() < 60 {
        println!("Execution time: {:?}", duration);
    } else {
        utils::print_formatted_duration(duration.as_secs());
    }
    
    println!("Thank you for using fastp-runner v{} 😊", &version);
}
