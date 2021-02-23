use std::fs;
use std::str;
use std::io::{self, Result, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[cfg(target_family="unix")]
use std::os::unix;

use spinners::{Spinner, Spinners};

use crate::parser::RawSeq;

pub fn clean_reads(reads: &[RawSeq]) {
    let dir = Path::new("clean_reads");
    check_dir_exists(&dir);

    reads.iter()
        .for_each(|r| {
            match r.adapter_i7.as_ref() { // Check if i7 contains sequence
                Some(_) => call_fastp(&dir, &r, true), // if yes -> dual index
                None => call_fastp(&dir, &r, false),
            };
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

fn call_fastp(dir: &Path, input: &RawSeq, is_dual_idx: bool) {
    let seq_dir = dir.join(&input.dir);
    let output_r1 = get_out_fnames(&seq_dir, &input.read_1);
    let output_r2 = get_out_fnames(&seq_dir, &input.read_2);
        
    let stdout = io::stdout();
    let mut buff = io::BufWriter::new(stdout);

    let msg = format!("Processing {:?}\t", input.dir);
    let spin = Spinner::new(Spinners::Moon, msg);

    if is_dual_idx {
        let adapter_i7 = String::from(input.adapter_i7.as_ref().unwrap());
        call_fastp_dual_idx(input, &output_r1, &output_r2, &adapter_i7).unwrap();
    } else if input.auto_idx {
        call_fastp_auto_idx(&input, &output_r1, &output_r2).unwrap();
    } else {
        call_fastp_single_idx(input, &output_r1, &output_r2).unwrap();
    }

    try_creating_symlink(&seq_dir, &input.read_1, &input.read_2);
    reorganize_reports(&seq_dir);
    
    spin.stop();

    writeln!(buff, "\x1b[0;32mDONE!\x1b[0m").unwrap();
}

fn call_fastp_auto_idx(
    input: &RawSeq, 
    output_r1: &PathBuf, 
    output_r2: &PathBuf) 
-> Result<()> {

    let out = Command::new("fastp")
        .arg("-i")
        .arg(input.read_1.clone())
        .arg("-I")
        .arg(input.read_2.clone())
        .arg("--detect_adapter_for_pe")
        .arg("-o")
        .arg(output_r1)
        .arg("-O")
        .arg(output_r2)
        .output()
        .unwrap();

    check_fastp_status(&out);
    write_stdout(&out);

    Ok(())
}

fn call_fastp_single_idx(
    input: &RawSeq, 
    output_r1: &PathBuf, 
    output_r2: &PathBuf) 
-> Result<()> {

    let out: Output = Command::new("fastp")
        .arg("-i")
        .arg(input.read_1.clone())
        .arg("-I")
        .arg(input.read_2.clone())
        .arg("--adapter_sequence")
        .arg(input.adapter_i5.clone())
        .arg("-o")
        .arg(output_r1)
        .arg("-O")
        .arg(output_r2)
        .output()
        .unwrap();

    check_fastp_status(&out);
    write_stdout(&out);

    Ok(())
}

fn call_fastp_dual_idx(
    input: &RawSeq, 
    output_r1: &PathBuf, 
    output_r2: &PathBuf,
    adapter_i7: &str
) -> Result<()> {

    let out = Command::new("fastp")
        .arg("-i")
        .arg(input.read_1.clone())
        .arg("-I")
        .arg(input.read_2.clone())
        .arg("--adapter_sequence")
        .arg(input.adapter_i5.clone())
        .arg("--adapter_sequence_r2")
        .arg(adapter_i7)
        .arg("-o")
        .arg(output_r1)
        .arg("-O")
        .arg(output_r2)
        .output()
        .unwrap();

    check_fastp_status(&out);
    write_stdout(&out);

    Ok(())
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

fn get_out_fnames(seq_dir: &Path, fnames: &Path) -> PathBuf {
    let outdir = seq_dir.join("trimmed-reads");
    fs::create_dir_all(&outdir).unwrap();

    outdir.join(fnames.file_name().unwrap())
}

fn try_creating_symlink(dir: &Path, read_1: &Path, read_2: &Path) {
    if cfg!(target_family="unix") {
        create_symlink(dir, read_1, read_2).unwrap(); 
    } else {
        println!("Skip creating symlink. Operating system is not supported.");
    }
}

#[cfg(target_family="unix")]
fn create_symlink(dir: &Path, read_1: &Path, read_2: &Path) -> Result<()> {
    let symdir = dir.join("raw_reads");
    fs::create_dir_all(&symdir).unwrap();

    let path_r1 = symdir.join(read_1.file_name().unwrap());
    let path_r2 = symdir.join(read_2.file_name().unwrap());

    unix::fs::symlink(read_1, path_r1).unwrap();
    unix::fs::symlink(read_2, path_r2).unwrap();

    Ok(())
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

pub fn check_fastp() {
    let out = Command::new("fastp")
        .arg("--version")
        .output()
        .expect("CANNOT FIND FASTP. IT MAY BE NOT PROPERLY INSTALLED.");
    
    if out.status.success() {
        println!("[OK]\t{}\n", str::from_utf8(&out.stderr).unwrap().trim());
    } 

}