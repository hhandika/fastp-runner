use std::path::PathBuf;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;

use glob::{glob_with, MatchOptions};

pub struct RawSeq {
    pub id: String, 
    pub read_1: PathBuf,
    pub read_2: PathBuf,
    pub adapter_i5: String,
    pub adapter_i7: Option<String>,
    pub dir: PathBuf,
}

impl RawSeq {
    pub fn new() -> Self {
        Self {
            id: String::new(),
            dir: PathBuf::new(),
            read_1: PathBuf::new(),
            read_2: PathBuf::new(),
            adapter_i5: String::new(),
            adapter_i7: None,
        }
    }

    fn get_id(&mut self, id: &str) {
        self.id = String::from(id);
    }

    fn get_dir(&mut self) {
        let fnames = String::from(
            self.read_1
                .file_name()
                .unwrap()
                .to_string_lossy()
        );

        let ids = split_strings(&fnames, false);
        let dir = format!("{}_{}_{}", ids[0], ids[1], ids[2]);
        self.dir = PathBuf::from(dir);
    }

    fn get_reads(&mut self, reads: &[PathBuf]) {
        reads.iter()
            .for_each(|reads| {
                match reads.to_string_lossy().to_lowercase() {
                    s if s.contains("read1") => self.read_1 = PathBuf::from(reads),
                    s if s.contains("_r1") => self.read_1 = PathBuf::from(reads),
                    s if s.contains("read2") => self.read_2 = PathBuf::from(reads),
                    s if s.contains("_r2") => self.read_2 = PathBuf::from(reads),
                    _ => (),
                }
            });
    }

    fn get_adapter_single(&mut self, adapter: &str) {
        self.adapter_i5 = String::from(adapter);
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
            let mut seq = RawSeq::new();
            let lines = split_strings(&line, true);
            let id = String::from(&lines[0]);
            let reads = glob_raw_reads(&input, &id, mid_id);

            seq.get_reads(&reads);
            seq.get_id(&id);
            seq.get_adapter_single(&lines[1]);
            seq.get_dir();
            raw_seqs.push(seq);
            lcounts += 1;
        });

    println!("Total files: {}", lcounts);
    raw_seqs
}

fn split_strings(lines: &str, csv: bool) -> Vec<String> {
    let mut sep = ',';
    if !csv {
        sep = '_';
    }
    let seqs = lines.split(sep)
        .map(|e| e.trim().to_string())
        .collect();
    
    seqs
}

fn glob_raw_reads(path: &PathBuf, id: &str, mid_id: bool) -> Vec<PathBuf> {
    let patterns = get_patterns(path, id, mid_id);
    
    let opts = MatchOptions {
        case_sensitive: false,
        ..Default::default()
    };

    glob_with(&patterns, opts)
        .unwrap()
        .filter_map(|ok| ok.ok())
        .collect()
}

fn get_patterns(path: &PathBuf, id: &str, mid_id: bool) -> String {
    let parent = path.parent().unwrap();
    let mut pat_id = format!("*?{}?*", id);

    if !mid_id {
        pat_id = format!("{}?*", id);
    }

    String::from(parent.join(pat_id).to_string_lossy())
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
                let dir = input.parent().unwrap();
                assert_eq!(dir.join("test_1_cde_R1.fastq"), s.read_1);
                assert_eq!(dir.join("test_1_cde_R2.fastq"), s.read_2);
                assert_eq!("AGTCT", s.adapter_i5);
            });
    }

    #[test]
    fn parse_csv_pattern_test() {
        let input = PathBuf::from("test_files/test2.csv");

        let seq = parse_csv(&input, true);
    
        seq.iter()
        .for_each(|s| {
            let dir = input.parent().unwrap();
            assert_eq!(dir.join("some_animals_XYZ12345_R1.fastq.gz"), s.read_1);
            assert_eq!(dir.join("some_animals_XYZ12345_R2.fastq.gz"), s.read_2);
            assert_eq!("ATGTCTCTCTATATATACT", s.adapter_i5);
        });
    }

}