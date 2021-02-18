// use std::env::consts;
use std::fs;
use std::process::Command;
// use std::io::{self, BufRead};
use std::path::PathBuf;

// use std::io::{self, Write};

fn main() {
    let input = PathBuf::from("data/sample_buno_r1.fastq.gz");
    let output = PathBuf::from("data/sample_buno_clean_r1.fastq.gz");
    let adapter = "CAAGCAGAAGACGGCATACGAGATGCCATAGGTGACTGGAGTTCAGACGTGT";
    run_fastp(&input, &output, &adapter);
}

fn run_fastp(input: &PathBuf, output: &PathBuf, adapter: &str) {
    println!("Processing {:?}", input);
    println!();
    let mut out = Command::new("fastp")
        .arg("-i")
        .arg(input)
        .arg("-o")
        .arg(output)
        .arg("-a")
        .arg(adapter)
        .spawn()
        .unwrap();

    out.wait().unwrap();
    let fastp_html = PathBuf::from("fastp.html");
    let fastp_json = PathBuf::from("fastp.json");
    let parent = output.parent().unwrap().join("res");
    fs::create_dir_all(&parent).unwrap();
    let html_out = parent.join(&fastp_html);
    let json_out = parent.join(&fastp_json);
    fs::rename(&fastp_html, &html_out).unwrap();
    fs::rename(&fastp_json, &json_out).unwrap();

    println!("Done!");
}