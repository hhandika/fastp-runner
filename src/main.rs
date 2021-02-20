// Heru Handika
// February 2021
// MIT

mod cli;
mod io;
mod parser;
mod wrapper;

use clap::crate_version;

fn main() {
    let version = crate_version!();
    println!("Starting fastp-runner v{}", &version);
    cli::get_cli(&version);

}
