use std::path::PathBuf;

use crate::parser::{self, RawSeq};

pub fn dry_run(input: &PathBuf) {
    let seq_reads: Vec<RawSeq> = parser::parse_csv(input, true);

    println!();
    seq_reads.iter()
        .for_each(|reads| {
            println!("\x1b[0;32mID\t: {}\x1b[0m", reads.id);
            println!("Dir\t: {}", reads.dir.to_string_lossy());
            println!("Read 1\t: {}", reads.read_1.to_string_lossy());
            println!("Read 2\t: {}", reads.read_2.to_string_lossy());
            println!("Adapter\t: {}", reads.adapter);
            println!();
        });
}