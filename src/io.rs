use std::path::PathBuf;

use crate::parser::{self, RawSeq};
use crate::wrapper;

pub fn dry_run(input: &PathBuf) {
    let reads: Vec<RawSeq> = parser::parse_csv(input, true);

    println!();
    reads.iter()
        .for_each(|r| {
            println!("\x1b[0;32mID\t\t: {}\x1b[0m", r.id);
            println!("Dir\t\t: {}", r.dir.to_string_lossy());
            println!("Read 1\t\t: {}", r.read_1.to_string_lossy());
            println!("Read 2\t\t: {}", r.read_2.to_string_lossy());

            match r.adapter_i7.as_ref() {
                Some(i7) => {
                    println!("Adapter i5\t: {}", r.adapter_i5);
                    println!("Adapter i7\t: {}", i7);
                }
                None => println!("Adapter\t\t: {}", r.adapter_i5),
            };

            println!();
        });
}

pub fn process_input(input: &PathBuf, is_mid_id: bool) {
    let reads: Vec<RawSeq> = parser::parse_csv(input, is_mid_id);
    wrapper::clean_reads(&reads);
}