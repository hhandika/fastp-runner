use std::fs;
use std::env::consts;
use std::os::unix;
use std::io::Result;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::parser::RawSeq;


pub fn clean_reads(reads: &[RawSeq]) {
    let dir = "clean_reads";
    fs::create_dir_all(dir)
        .expect("CAN'T CREATE CLEAN READ DIR");

    reads.iter()
        .for_each(|r| {
            print!("Processing {:?}", r.dir);
            match r.adapter_i7.as_ref() { // Check if i7 contains sequence
                Some(_) => call_fastp(&r, true), // if yes -> dual index
                None => call_fastp(&r, false),
            };

            println!("Done!");
        })
} 

fn call_fastp(input: &RawSeq, is_dual_idx: bool) {
    let output_r1 = get_out_fnames(&input.dir, &input.read_1);
    let output_r2 = get_out_fnames(&input.dir, &input.read_2);

    if is_dual_idx {
        let adapter_i7 = String::from(input.adapter_i7.as_ref().unwrap());
        call_fastp_dual_idx(input, &output_r1, &output_r2, &adapter_i7).unwrap();
    } else if input.auto_idx {
        call_fastp_auto_idx(&input, &output_r1, &output_r2).unwrap();
    } else {
        call_fastp_single_idx(input, &output_r1, &output_r2).unwrap();
    }

    try_creating_symlink(&input.read_1, &input.read_2);
    reorganize_reports();
}

fn call_fastp_auto_idx(
    input: &RawSeq, 
    output_r1: &PathBuf, 
    output_r2: &PathBuf) 
-> Result<()> {

    let mut out = Command::new("fastp")
        .arg("-i")
        .arg(input.read_1.clone())
        .arg("-I")
        .arg(input.read_2.clone())
        .arg("--detect_adapter_for_pe")
        .arg("-o")
        .arg(output_r1)
        .arg("-O")
        .arg(output_r2)
        .spawn()
        .unwrap();

    out.wait().unwrap();

    Ok(())
}

fn call_fastp_single_idx(
    input: &RawSeq, 
    output_r1: &PathBuf, 
    output_r2: &PathBuf) 
-> Result<()> {

    let mut out = Command::new("fastp")
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
        .spawn()
        .unwrap();

    out.wait().unwrap();

    Ok(())
}

fn call_fastp_dual_idx(
    input: &RawSeq, 
    output_r1: &PathBuf, 
    output_r2: &PathBuf,
    adapter_i7: &str
) -> Result<()> {
    
    let mut out = Command::new("fastp")
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
        .spawn()
        .unwrap();

    out.wait().unwrap();

    Ok(())
}

fn get_out_fnames(seq_dir: &PathBuf, fnames: &PathBuf) -> PathBuf {
    let clean_dir = Path::new("clean_reads");
    let outdir = clean_dir.join(seq_dir).join("trimmed-reads");
    fs::create_dir_all(&outdir).unwrap();

    outdir.join(fnames.file_name().unwrap())
}

fn try_creating_symlink(read_1: &PathBuf, read_2: &PathBuf) {
    let os = consts::OS;
    match os {
        "linux" | "macos" => create_symlink(&read_1, &read_2).unwrap(),
        "windows" => println!("The program can't create symlink in Windows"),
        _ => ()
    };
}

fn create_symlink(read_1: &PathBuf, read_2: &PathBuf) -> Result<()> {
    let dir = Path::new("clean_reads");
    let symdir = dir.join("raw_reads");
    fs::create_dir_all(&symdir).unwrap();

    let path_r1 = symdir.join(read_1.file_name().unwrap());
    let path_r2 = symdir.join(read_2.file_name().unwrap());

    unix::fs::symlink(read_1, path_r1).unwrap();
    unix::fs::symlink(read_2, path_r2).unwrap();

    Ok(())
}

fn reorganize_reports() {
    let fastp_html = PathBuf::from("fastp.html");
    let fastp_json = PathBuf::from("fastp.json");

    let dir = Path::new("clean_reads");
    let parent = dir.parent().unwrap().join("fastp_reports");

    fs::create_dir_all(&parent).unwrap();

    let html_out = parent.join(&fastp_html);
    let json_out = parent.join(&fastp_json);
    
    // Move json and html reports
    fs::rename(&fastp_html, &html_out).unwrap();
    fs::rename(&fastp_json, &json_out).unwrap();
}

pub fn check_fastp() {
    let out = Command::new("iqtree")
        .arg("--version")
        .output()
        .expect("CANNOT FIND Fastp");
    
    if out.status.success() {
        println!("[OK]\tFastp");
    } else {
        println!("ERROR")
    }
}