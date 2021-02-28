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
            println!("\x1b[0;33m================Processing {}================\x1b[0m", &reads.id);
            let mut run = Runner::new(&dir, &reads);

            if reads.adapter_i7.as_ref().is_some() { // Check if i7 contains sequence
                run.dual_idx = true;
            }

            run.get_out_fnames(); 
            run.display_settings();
            run.process_reads();
        });

    println!();
} 

fn check_dir_exists(dir: &Path) {
    if dir.exists() {
        panic!("CLEAN READ DIR EXISTS. PLEASE RENAME OR REMOVE IT");
    } else { // if not create one
        fs::create_dir_all(dir)
            .expect("CAN'T CREATE CLEAN READ DIR");
    }
}

struct Runner<'a> {
    clean_dir: PathBuf,
    dual_idx: bool,
    out_r1: PathBuf,
    out_r2: PathBuf,
    reads: &'a RawSeq,
}

impl<'a> Runner<'a> {
    fn new(dir: &Path, input: &'a RawSeq) -> Self {
        Self {
            clean_dir: dir.join(&input.dir),
            dual_idx: false,
            out_r1: PathBuf::new(),
            out_r2: PathBuf::new(),
            reads: input,
        }
    }

    fn process_reads(&mut self) { 
        let spin = self.set_spinner();
        let out = self.call_fastp();
        
        let mut reports = FastpReports::new(&self.clean_dir);
        
        reports.check_fastp_status(&out);
        reports.write_stdout(&out);
        self.try_creating_symlink();
        reports.reorganize_reports();
        spin.stop();
        self.print_done();
        reports.display_report_paths();
    }

    fn print_done(&self) {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        writeln!(handle, "\x1b[0;32mDONE!\x1b[0m").unwrap();
    }
    
    // fn get_dir(&mut self, dir: &Path) {
    //     if self.is_rename() {
    //         self.clean_dir = dir.join(&self.reads.outname.as_ref().unwrap());
    //     } else {
    //         self.clean_dir = dir.join(&self.reads.dir);
    //     }
    // }

    fn get_out_fnames(&mut self) {
        let outdir = self.clean_dir.join("trimmed_reads");
        fs::create_dir_all(&outdir).unwrap();
        
        let out1 = self.reads.read_1.file_name().unwrap();
        let out2 = self.reads.read_2.file_name().unwrap();

        if self.is_rename() {
            let out1 = self.rename_output(&out1.to_str().unwrap());
            let out2 = self.rename_output(&out2.to_str().unwrap());
            self.out_r1 = outdir.join(out1);
            self.out_r2 = outdir.join(out2);
        } else {
            self.out_r1 = outdir.join(out1);
            self.out_r2 = outdir.join(out2);
        }
    }

    fn is_rename(&self) -> bool {
        self.reads.outname.is_some()
    }

    fn rename_output(&self, outname: &str) -> String {
        let target = self.reads.outname.as_ref().unwrap();
        outname.replace(&self.reads.id, &target)
    }

    fn display_settings(&self) {
        let stdout = io::stdout();
        let mut buff = io::BufWriter::new(stdout);
        
        // writeln!(buff).unwrap();
        writeln!(buff, "Target dir\t: {}", &self.clean_dir.to_string_lossy()).unwrap();
        writeln!(buff, "Input R1\t: {}", &self.reads.read_1.to_string_lossy()).unwrap();
        writeln!(buff, "Input R2\t: {}", &self.reads.read_2.to_string_lossy()).unwrap();
        writeln!(buff, "Output R1\t: {}", &self.out_r1.to_string_lossy()).unwrap();
        writeln!(buff, "Output R2\t: {}", &self.out_r2.to_string_lossy()).unwrap();
        
        if self.reads.auto_idx {
            writeln!(buff, "Adapters\t: AUTO-DETECT").unwrap();
        } else if !self.dual_idx {
            writeln!(buff, "Adapters\t: {}", self.reads.adapter_i5.as_ref().unwrap()).unwrap();
        } else {
            writeln!(buff, "Adapter i5\t: {}", self.reads.adapter_i5.as_ref().unwrap()).unwrap();
            writeln!(buff, "Adapters i7\t: {}", self.reads.adapter_i7.as_ref().unwrap()).unwrap();
        }

        writeln!(buff).unwrap();
    }

    fn set_spinner(&mut self) -> Spinner {
        let msg = "Fastp is processing...\t".to_string();
        
        Spinner::new(Spinners::Moon, msg)
    }

    fn call_fastp(&self) -> Output {
        let mut out = Command::new("fastp");

        out.arg("-i")
            .arg(self.reads.read_1.clone())
            .arg("-I")
            .arg(self.reads.read_2.clone())
            .arg("-o")
            .arg(self.out_r1.clone())
            .arg("-O")
            .arg(self.out_r2.clone());

        self.set_fastp_idx(&mut out);
        out.output().unwrap()
    }

    fn set_fastp_idx(&self, out: &mut Command) {
        if self.dual_idx {
            self.set_fastp_dual_idx(out);
        } else if self.reads.auto_idx {
            self.set_fastp_auto_idx(out);
        } else {
            self.set_fastp_single_idx(out);
        }
    }

    fn set_fastp_auto_idx(&self, out: &mut Command) {
        out.arg("--detect_adapter_for_pe");
    }

    fn set_fastp_single_idx(&self, out: &mut Command) {
        out.arg("--adapter_sequence")
            .arg(String::from(self.reads.adapter_i5.as_ref().unwrap()));
    }

    fn set_fastp_dual_idx(&self, out: &mut Command) {
        out.arg("--adapter_sequence")
            .arg(String::from(self.reads.adapter_i5.as_ref().unwrap()))
            .arg("--adapter_sequence_r2")
            .arg(String::from(self.reads.adapter_i7.as_ref().unwrap()));
    }

    fn try_creating_symlink(&self) {
        if cfg!(target_family="unix") {
            #[cfg(target_family="unix")]
            self.create_symlink().unwrap();
        } else {
            println!("Skip creating symlink in dir {} for {} and {}. \
                Operating system is not supported.", 
                &self.clean_dir.to_string_lossy(), 
                &self.reads.read_1.to_string_lossy(), 
                &self.reads.read_2.to_string_lossy());
        }
    }
    
    #[cfg(target_family="unix")]
    fn create_symlink(&self) -> Result<()> {
        let symdir = self.clean_dir.join("raw_read_symlinks");
        fs::create_dir_all(&symdir).unwrap();
    
        let path_r1 = symdir.join(self.reads.read_1.file_name().unwrap());
        let path_r2 = symdir.join(self.reads.read_2.file_name().unwrap());
    
        unix::fs::symlink(&self.reads.read_1, path_r1).unwrap();
        unix::fs::symlink(&self.reads.read_2, path_r2).unwrap();
    
        Ok(())
    }
    
}

struct FastpReports {
    dir: PathBuf,
    html: PathBuf,
    json: PathBuf,
    log: PathBuf,
    html_out: PathBuf,
    json_out: PathBuf,
    log_out: PathBuf,
}

impl FastpReports {
    fn new(dir: &Path) -> Self {
        Self {
            dir: dir.join("fastp_reports"),
            html: PathBuf::from("fastp.html"),
            json: PathBuf::from("fastp.json"),
            log: PathBuf::from("fastp.log"),
            html_out: PathBuf::new(),
            json_out: PathBuf::new(),
            log_out: PathBuf::new(),
        }
    }

    // Less likely this will be called 
    // because potential input errors that cause fastp
    // to failed is mitigated before passing the input
    // to it.
    fn check_fastp_status(&self, out: &Output) {
        if !out.status.success() {
            self.fastp_is_failed(out);
        }

        if !self.html.is_file() || !self.json.is_file() {
            self.fastp_is_failed(out);
        }
    }
    
    fn fastp_is_failed(&self, out: &Output) {
        io::stdout().write_all(&out.stdout).unwrap();
        io::stdout().write_all(&out.stderr).unwrap();
        panic!("FASTP FAILED TO RUN");
    }

    // We remove the clutter of fastp stdout in the console. 
    // Instead, we save it as a log file.
    fn write_stdout(&self, out: &Output) {
        let fname = fs::File::create(&self.log).unwrap();
        let mut buff = BufWriter::new(&fname);

        // Rust recognize fastp console output as stderr
        // Hence, we write stderr instead of stdout.
        buff.write_all(&out.stderr).unwrap();
    }

    fn reorganize_reports(&mut self) {
        fs::create_dir_all(&self.dir).unwrap();
    
        self.html_out = self.dir.join(&self.html);
        self.json_out = self.dir.join(&self.json);
        self.log_out = self.dir.join(&self.log);
        
        // Move json, html, and log reports
        fs::rename(&self.html, &self.html_out).unwrap();
        fs::rename(&self.json, &self.json_out).unwrap();
        fs::rename(&self.log, &self.log_out).unwrap();
    }

    fn display_report_paths(&self) {
        let stdout = io::stdout();
        let mut handle = io::BufWriter::new(stdout);

        writeln!(handle).unwrap();
        writeln!(handle, "Fastp Reports:").unwrap();
        writeln!(handle, "1. {}", self.html_out.to_string_lossy()).unwrap();
        writeln!(handle, "2. {}", self.json_out.to_string_lossy()).unwrap();
        writeln!(handle, "3. {}", self.log_out.to_string_lossy()).unwrap();
        writeln!(handle).unwrap();
    }   
}