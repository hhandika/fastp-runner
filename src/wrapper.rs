use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub fn run_fastp(input: &PathBuf, adapter: &str) {
    println!("Processing {:?}", input);
    println!();

    let output = get_out_fnames(input);
    let mut out = Command::new("fastp")
        .arg("-i")
        .arg(input)
        .arg("-o")
        .arg(&output)
        .arg("-a")
        .arg(adapter)
        .spawn()
        .unwrap();

    out.wait().unwrap();
    reorganize_reports(input);

    println!("Done!");
}

fn get_out_fnames(input: &PathBuf) -> PathBuf {
    let indir = input.parent().unwrap();
    let outdir = indir.join("clean_reads");
    fs::create_dir_all(&outdir).unwrap();
    let fname = input.file_name().unwrap();
    
    outdir.join(fname)
}

fn reorganize_reports(input: &PathBuf) {
    let fastp_html = PathBuf::from("fastp.html");
    let fastp_json = PathBuf::from("fastp.json");
    let parent = input.parent().unwrap().join("fastp_reports");

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