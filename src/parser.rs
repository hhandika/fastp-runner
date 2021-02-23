use std::path::PathBuf;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;

use glob::{glob_with, MatchOptions};

use crate::itru;

#[derive(Clone)]
pub struct RawSeq {
    pub id: String, 
    pub read_1: PathBuf,
    pub read_2: PathBuf,
    pub adapter_i5: String,
    pub adapter_i7: Option<String>,
    pub dir: PathBuf,
    pub auto_idx: bool,
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
            auto_idx: false,
        }
    }

    fn get_id(&mut self, id: &str) {
        self.id = String::from(id);
    }

    fn get_dir(&mut self) {
        if self.read_1.to_string_lossy().is_empty() {
            panic!("MISSING READ FILES FOR {}. \
                Read 1: {:?} \
                Read 2: {:?}", 
                self.id, 
                self.read_1,
                self.read_2);
        }
        
        let fnames = String::from(
            self.read_1
                .file_name()
                .expect("MISSING FILES")
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

    fn get_adapter_dual(&mut self, adapter_i5: &str, adapter_i7: &str) {
        let adapter_i5 = String::from(adapter_i5.trim());
        let adapter_i7 = String::from(adapter_i7.trim());

        if !adapter_i5.is_empty() && !adapter_i7.is_empty() {
            self.adapter_i5 = adapter_i5;
            self.adapter_i7 = Some(adapter_i7);
        } else if !adapter_i5.is_empty() && adapter_i7.is_empty() {
            self.adapter_i5 = adapter_i5;
        } else if adapter_i5.is_empty() && adapter_i7.is_empty(){
            self.get_adapter_auto();
        } else {
            self.adapter_i5 = adapter_i5;
        }
    }

    fn get_adapter_auto(&mut self) {
        self.auto_idx = true;
    }

}

pub fn parse_csv(input: &PathBuf, mid_id: bool) -> Vec<RawSeq> {
    let file = File::open(input).unwrap();
    let buff = BufReader::new(file);

    let mut raw_seqs = Vec::new();
    let mut lcounts: usize = 0;

    buff.lines()
        .filter_map(|ok| ok.ok())
        .skip(1)
        .for_each(|line| {
            let mut seq = RawSeq::new();
            let lines = split_strings(&line, true);
            let id = String::from(&lines[0]);
            let reads = glob_raw_reads(&input, &id, mid_id);
            check_reads(&reads, &id, &lcounts);
            seq.get_reads(&reads);
            seq.get_id(&id);
            seq.get_dir();
            get_adapters(&mut seq, &lines);

            raw_seqs.push(seq);
            lcounts += 1;
        });

    println!("Total files: {}", lcounts);

    raw_seqs
}

fn check_reads(reads: &[PathBuf], id: &str, lnum: &usize) {
    if reads.is_empty() {
        panic!("FILE {} AT LINE {} IS MISSING", id, lnum + 1)
    }
}

fn get_adapters(seq: &mut RawSeq, adapters: &[String]) {
    match adapters.len() {
        1 => seq.get_adapter_auto(),
        2 => {
            let i5 = adapters[1].to_uppercase();
            if is_insert_missing(&i5) {
                panic!("INSERT MISSING!");
            } else {
                seq.get_adapter_single(&i5);
            }  
        },

        3 => {
            let i5 = adapters[1].to_uppercase();
            if is_insert_missing(&i5) {
                panic!("INSERT MISSING!");
            } else {
            let i7 = adapters[2].to_uppercase();
            seq.get_adapter_dual(&i5, &i7);
            }
        },

        4 => {
            let i7 = adapters[2].to_uppercase();
            if is_insert_missing(&adapters[1]) {
                let i5 = itru::insert_tag(&adapters[1], &adapters[3]);  
                seq.get_adapter_dual(&i5, &i7);
            } else {
                panic!("TOO MANY COLUMNS!");
            }
        }

        5 => {
            let i5 = itru::insert_tag(&adapters[1], &adapters[3]);
            let i7 = itru::insert_tag(&adapters[2], &adapters[4]);
            seq.get_adapter_dual(&i5, &i7);
        },

        _ => panic!("Unexpected cvs columns. It should be \
            2 columns for single index and 3 column for \
            dual index. The app received {} columns", adapters.len()),
    }
}

fn is_insert_missing(adapter: &str) -> bool {
    adapter.contains('*')
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
        case_sensitive: true,
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

    #[test]
    fn parse_csv_dual_indexes_test() {
        let input = PathBuf::from("test_files/dual_index_test.csv");

        let seq = parse_csv(&input, true);
        let i5 = "ATGTCTCTCTATATATACT";
        let i7 = String::from("ATGTCTCTCTATATATGCT");
        seq.iter()
            .for_each(|s| {
                let dir = input.parent().unwrap();
                assert_eq!(dir.join("some_animals_XYZ12345_R1.fastq.gz"), s.read_1);
                assert_eq!(dir.join("some_animals_XYZ12345_R2.fastq.gz"), s.read_2);
                assert_eq!(i5, s.adapter_i5);
                assert_eq!(true, s.adapter_i7.is_some());
                assert_eq!(i7, String::from(s.adapter_i7.as_ref().unwrap()))
        });
    }

    #[test]
    #[should_panic]
    fn parse_csv_panic_test() {
        let input = PathBuf::from("test_files/invalid.csv");

        parse_csv(&input, true);
    }

    #[test]
    #[should_panic]
    fn parse_csv_multicols_panic_test() {
        let input = PathBuf::from("test_files/invalid_multicols.csv");

        parse_csv(&input, true);
    }

    #[test]
    fn get_adapter_test() {
        let mut seq = RawSeq::new();
        let id = String::from("MNCT");
        let i5 = String::from("ATGTGTGTGATatc");
        let i7 = String::from("ATTTGTGTTTCCC");

        let adapters: Vec<String> = vec![id, i5, i7];

        get_adapters(&mut seq, &adapters);

        assert_eq!("ATGTGTGTGATATC", seq.adapter_i5);

    }

    #[test]
    fn get_adapter_insert_test() {
        let mut seq = RawSeq::new();
        let id = String::from("MNCT");
        let i5 = String::from("ATGTGTGTGA*Tatc");
        let i7 = String::from("ATTTGTGTTT*CCC");

        let tag_i5 = String::from("ATT");
        let tag_i7 = String::from("GCC");

        let adapters: Vec<String> = vec![id, i5, i7, tag_i5, tag_i7];

        get_adapters(&mut seq, &adapters);

        assert_eq!("ATGTGTGTGATAATATC", seq.adapter_i5);
        assert_eq!("ATTTGTGTTTCGGCCC", String::from(seq.adapter_i7.as_ref().unwrap()));
    }

    #[test]

    fn is_insert_test() {
        let seq = "ATATTAT*T";

        assert_eq!(true, is_insert_missing(seq));
    }

}