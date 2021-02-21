// Heru Handika
// February 2021
// MIT

mod cli;
mod io;
mod parser;
mod wrapper;

use std::time::Instant;

use clap::crate_version;

fn main() {
    let version = crate_version!();
    
    let time = Instant::now();
    println!("Starting fastp-runner v{}", &version);
    cli::get_cli(&version);
    let duration = time.elapsed();

    println!("Execution time: {:?}", duration);
    println!("Thank you for using fastp-runner v{}", &version);
}
