use std::fs;
use std::str;
use std::io::{self, Result, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[cfg(target_family="unix")]
use std::os::unix;

use spinners::{Spinner, Spinners};

use crate::parser::RawSeq;

pub fn check_fastp() {
    let out = Command::new("fastp")
        .arg("--version")
        .output()
        .expect("CANNOT FIND FASTP. IT MAY BE NOT PROPERLY INSTALLED.");
    
    if out.status.success() {
        println!("[OK]\t{}\n", str::from_utf8(&out.stderr).unwrap().trim());
    } 

}

pub fn clean_reads(reads: &[RawSeq]) {
    let dir = Path::new("clean_reads");
    check_dir_exists(&dir);

    reads.iter()
        .for_each(|reads| {
            let mut run = Runner::new(&dir, &reads);
            match reads.adapter_i7.as_ref() { // Check if i7 contains sequence
                Some(_) => run.dual_idx = true, // if yes -> dual index
                None => (),
            };

            run.call_fastp();
        });

    println!();
} 

fn check_dir_exists(dir: &Path) {
    if dir.exists() {
        panic!("CLEAN READ DIR EXISTS");
    } else { // if not create one
        fs::create_dir_all(dir)
        .expect("CAN'T CREATE CLEAN READ DIR");
    }
}

struct Runner {
    clean_dir: PathBuf,
    adapter_i5: Option<String>,
    adapter_i7: Option<String>,
    dual_idx: bool,
    auto_idx: bool,
    in_r1: PathBuf,
    in_r2: PathBuf,
    out_r1: PathBuf,
    out_r2: PathBuf
}

impl Runner {
    fn new(dir: &Path, input: &RawSeq) -> Self {
        Self {
            clean_dir: dir.join(&input.dir),
            adapter_i5: Some(input.adapter_i5.clone()),
            adapter_i7: input.adapter_i7.clone(),
            dual_idx: false,
            auto_idx: input.auto_idx,
            in_r1: input.read_1.clone(),
            in_r2: input.read_2.clone(),
            out_r1: PathBuf::new(),
            out_r2: PathBuf::new(),
        }
    }

    fn call_fastp(&mut self) {
        self.get_out_fnames();  
        let stdout = io::stdout();
        let mut buff = io::BufWriter::new(stdout);
        let spin = self.set_spinner();
        
        let out: Output;
        if self.dual_idx {
            out = self.call_fastp_dual_idx();
        } else if self.auto_idx {
            out = self.call_fastp_auto_idx();
        } else {
            out = self.call_fastp_single_idx();
        }
        
        check_fastp_status(&out);
        write_stdout(&out); 
        self.try_creating_symlink();
        reorganize_reports(&self.clean_dir);
        
        spin.stop();
    
        writeln!(buff, "\x1b[0;32mDONE!\x1b[0m").unwrap();
    }

    fn get_out_fnames(&mut self) {
        let outdir = self.clean_dir.join("trimmed-reads");
        fs::create_dir_all(&outdir).unwrap();
        
        let r1 = self.in_r1.file_name().unwrap();
        let r2 = self.in_r2.file_name().unwrap();
        self.out_r1 = outdir.join(r1);
        self.out_r2 = outdir.join(r2);
    }

    fn set_spinner(&mut self) -> Spinner {
        let msg = format!("Fastp is processing... \t");
        let spin = Spinner::new(Spinners::Moon, msg);

        spin
    }

    fn call_fastp_auto_idx(&self)-> Output {
        let out = Command::new("fastp")
            .arg("-i")
            .arg(self.in_r1.clone())
            .arg("-I")
            .arg(self.in_r2.clone())
            .arg("--detect_adapter_for_pe")
            .arg("-o")
            .arg(self.out_r1.clone())
            .arg("-O")
            .arg(self.out_r2.clone())
            .output()
            .unwrap();
    
        out
    }

    fn call_fastp_single_idx(&self) -> Output {
        let out: Output = Command::new("fastp")
            .arg("-i")
            .arg(self.in_r1.clone())
            .arg("-I")
            .arg(self.in_r2.clone())
            .arg("--adapter_sequence")
            .arg(String::from(self.adapter_i5.as_ref().unwrap()))
            .arg("-o")
            .arg(self.out_r1.clone())
            .arg("-O")
            .arg(self.out_r2.clone())
            .output()
            .unwrap();
    
        out
    }

    fn call_fastp_dual_idx(&self) -> Output {
        let out = Command::new("fastp")
            .arg("-i")
            .arg(self.in_r1.clone())
            .arg("-I")
            .arg(self.in_r2.clone())
            .arg("--adapter_sequence")
            .arg(String::from(self.adapter_i5.as_ref().unwrap()))
            .arg("--adapter_sequence_r2")
            .arg(String::from(self.adapter_i7.as_ref().unwrap()))
            .arg("-o")
            .arg(self.out_r1.clone())
            .arg("-O")
            .arg(self.out_r2.clone())
            .output()
            .unwrap();
    
        out
    }

    fn try_creating_symlink(&self) {
        if cfg!(target_family="unix") {
            #[cfg(target_family="unix")]
            self.create_symlink().unwrap();
        } else {
            println!("Skip creating symlink in dir {} for {} and {}. \
                Operating system is not supported.", 
                &self.clean_dir.to_string_lossy(), 
                &self.in_r1.to_string_lossy(), 
                &self.in_r2.to_string_lossy());
        }
    }
    
    #[cfg(target_family="unix")]
    fn create_symlink(&self) -> Result<()> {
        let symdir = self.clean_dir.join("raw_read_symlinks");
        fs::create_dir_all(&symdir).unwrap();
    
        let path_r1 = symdir.join(self.in_r1.file_name().unwrap());
        let path_r2 = symdir.join(self.in_r2.file_name().unwrap());
    
        unix::fs::symlink(&self.in_r1, path_r1).unwrap();
        unix::fs::symlink(&self.in_r2, path_r2).unwrap();
    
        Ok(())
    }
    
}

// Less likely this will be called 
// because potential input errors that cause fastp
// to failed is mitigated before passing the input
// to it.
fn check_fastp_status(out: &Output) {
    if !out.status.success() {
        fastp_is_failed(out);
    }

    let fastp_html = Path::new("fastp.html");
    let fastp_json = Path::new("fastp.json");

    if !fastp_html.is_file() || !fastp_json.is_file() {
        fastp_is_failed(out);
    }
}

fn fastp_is_failed(out: &Output) {
    io::stdout().write_all(&out.stdout).unwrap();
    io::stdout().write_all(&out.stderr).unwrap();
    panic!("FASTP FAILED TO RUN");
}

// We remove the clutter of fastp stdout in the console. 
// Instead, we save it as a log file.
fn write_stdout(out: &Output) {
    let fname = fs::File::create("fastp.log").unwrap();
    let mut buff = BufWriter::new(&fname);

    // Rust recognize fastp console output as stderr
    // Hence, we write stderr instead of stdout.
    buff.write_all(&out.stderr).unwrap();
}

fn reorganize_reports(dir: &Path) {
    let fastp_html = Path::new("fastp.html");
    let fastp_json = Path::new("fastp.json");
    let fastp_out = Path::new("fastp.log");

    let parent = dir.join("fastp_reports");

    fs::create_dir_all(&parent).unwrap();

    let html_out = parent.join(&fastp_html);
    let json_out = parent.join(&fastp_json);
    let log_out = parent.join(&fastp_out);
    
    // Move json, html, and log reports
    fs::rename(&fastp_html, &html_out).unwrap();
    fs::rename(&fastp_json, &json_out).unwrap();
    fs::rename(&fastp_out, &log_out).unwrap();
}