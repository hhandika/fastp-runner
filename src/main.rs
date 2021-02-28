// Heru Handika
// February 2021
// MIT

mod cli;
mod cleaner;
mod io;
mod itru;
mod parser;

use std::time::Instant;

use clap::crate_version;

fn main() {
    let version = crate_version!();
    let time = Instant::now();
    cli::get_cli(&version);
    let duration = time.elapsed();

    println!("Execution time: {:?}", duration);
    println!("Thank you for using fastp-runner v{} 😊", &version);
}
