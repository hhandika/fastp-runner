use std::path::PathBuf;

use crate::parser::{self, RawSeq};

pub fn get_sequences(input: &PathBuf) {
    let seq_reads: Vec<RawSeq> = parser::parse_csv(input, true);

    seq_reads.iter()
        .for_each(|reads| {
            println!("Dir {:?}", reads.dir);
            println!("Read 1 {:?}", reads.read_1);
            println!("Read 2 {:?}", reads.read_2);
            println!("Adapter: {}", reads.adapter);
        })
}