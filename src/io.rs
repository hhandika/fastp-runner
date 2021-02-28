use std::path::PathBuf;

use crate::parser::{self, RawSeq};
use crate::wrapper;

pub fn dry_run(input: &PathBuf, is_id: bool, is_rename: bool) {
    display_fastp_status();
    let reads: Vec<RawSeq> = parser::parse_csv(input, is_id, is_rename);

    println!();
    reads.iter()
        .for_each(|r| {
            println!("\x1b[0;32mID\t\t: {}\x1b[0m", r.id);
            println!("Dir\t\t: {}", r.dir.to_string_lossy());
            println!("Read 1\t\t: {}", r.read_1.to_string_lossy());
            println!("Read 2\t\t: {}", r.read_2.to_string_lossy());

            match r.adapter_i7.as_ref() {
                Some(i7) => {
                    println!("Adapter i5\t: {}", r.adapter_i5.as_ref().unwrap());
                    println!("Adapter i7\t: {}", i7);
                }
                None => {
                    if r.auto_idx {
                        println!("Adapter\t\t: AUTO-DETECT");
                    } else {
                        println!("Adapter\t\t: {}", r.adapter_i5.as_ref().unwrap());
                    }
                }
            };

            println!();
        });

}

pub fn process_input(input: &PathBuf, is_id: bool, is_rename: bool) {
    display_fastp_status();
    let reads: Vec<RawSeq> = parser::parse_csv(input, is_id, is_rename);
    wrapper::clean_reads(&reads);
}

fn display_fastp_status() {
    println!("Checking fastp...");
    wrapper::check_fastp();
}