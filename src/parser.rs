use std::path::PathBuf;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;

use glob::{glob_with, MatchOptions};

pub struct RawSeq {
    input: PathBuf,
    pub dir: PathBuf,
    pub read_1: PathBuf,
    pub read_2: PathBuf,
    pub adapter: String,
}

impl RawSeq {
    pub fn new(input: &PathBuf) -> Self {
        Self {
            input: PathBuf::from(input),
            dir: PathBuf::new(),
            read_1: PathBuf::new(),
            read_2: PathBuf::new(),
            adapter: String::new(),
        }
    }

    fn get_dir(&mut self) {
        self.dir = self.input
            .parent()
            .unwrap()
            .to_path_buf();
    }

    fn get_reads(&mut self, reads: &[PathBuf]) {
        reads.iter()
            .for_each(|reads| {
                match reads.to_string_lossy() {
                    s if s.contains("Read1") => self.read_1 = PathBuf::from(reads),
                    s if s.contains("R1") => self.read_1 = PathBuf::from(reads),
                    s if s.contains("READ1") => self.read_1 = PathBuf::from(reads),
                    s if s.contains("Read2") => self.read_2 = PathBuf::from(reads),
                    s if s.contains("R2") => self.read_2 = PathBuf::from(reads),
                    s if s.contains("READ2") => self.read_2 = PathBuf::from(reads),
                    _ => (),
                }
            });
    }

    fn get_adapter(&mut self, adapter: &str) {
        self.adapter = String::from(adapter);
    }

}

pub fn parse_csv(input: &PathBuf, mid_id: bool) -> Vec<RawSeq> {
    let file = File::open(input).unwrap();
    let buff = BufReader::new(file);

    let mut raw_seqs = Vec::new();
    let mut lcounts = 0;

    buff.lines()
        .filter_map(|ok| ok.ok())
        .skip(1)
        .for_each(|line| {
            let mut seq = RawSeq::new(input);
            let lines = split_csv_lines(&line);
            let reads = glob_raw_reads(&input, &lines[0], mid_id);

            seq.get_dir();
            seq.get_reads(&reads);
            seq.get_adapter(&lines[1]);
            raw_seqs.push(seq);
            lcounts += 1;
        });

    println!("Total files: {}", lcounts);
    raw_seqs
}

fn split_csv_lines(lines: &str) -> Vec<String> {
    let seqs = lines.split(',')
        .map(|e| e.trim().to_string())
        .collect();
    
    seqs
}

fn glob_raw_reads(path: &PathBuf, id: &str, mid_id: bool) -> Vec<PathBuf> {
    let parent = path.parent().unwrap();
    let mut pat_id = format!("*?{}?*", id);

    if !mid_id {
        pat_id = format!("{}?*", id);
    }

    let patterns = String::from(parent.join(pat_id).to_string_lossy());
    
    let opts = MatchOptions {
        case_sensitive: false,
        ..Default::default()
    };

    glob_with(&patterns, opts)
        .unwrap()
        .filter_map(|ok| ok.ok())
        .collect()
}


#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn glob_raw_reads_test() {
        let input = PathBuf::from("test_files/data.test");
        let pattern = "cde";

        let files = glob_raw_reads(&input, &pattern, true);

        assert_eq!(2, files.len());
    }

    #[test]
    fn glob_id_at_start_test() {
        let input = PathBuf::from("test_files/data.test");
        let pattern = "test_1";
        let mid_id = false;

        let files = glob_raw_reads(&input, &pattern, mid_id);

        assert_eq!(2, files.len());
    }
    
    #[test]
    fn parse_csv_test() {
        let input = PathBuf::from("test_files/test.csv");

        let seq = parse_csv(&input, true);

        assert_eq!(1, seq.len());
        
        seq.iter()
            .for_each(|s| {
                let dir = PathBuf::from(&s.dir);
                assert_eq!(dir.join("test_1_cde_R1.fastq"), s.read_1);
                assert_eq!(dir.join("test_1_cde_R2.fastq"), s.read_2);
                assert_eq!("AGTCT", s.adapter);
            });
    }

    #[test]
    fn parse_csv_pattern_test() {
        let input = PathBuf::from("test_files/test2.csv");

        let seq = parse_csv(&input, true);
    
        seq.iter()
        .for_each(|s| {
            let dir = PathBuf::from(&s.dir);
            assert_eq!(dir.join("some_animals_MNM12345_R1.fastq.gz"), s.read_1);
            assert_eq!(dir.join("some_animals_MNM12345_R2.fastq.gz"), s.read_2);
            assert_eq!("ATGTCTCTCTATATATACT", s.adapter);
        });
    }

}