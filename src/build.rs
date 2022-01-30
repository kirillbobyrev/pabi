// TODO: Describe why this is needed and what the details are.

use std::error::Error;
use std::fmt::Write;
use std::path::Path;
use std::{env, fs, process};

fn generate_file(filename: &str, contents: &str) {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join(filename);
    fs::write(&dest_path, contents).unwrap();
}


// TODO: This can fail at several levels: be more principled about it.
fn git_revision_hash() -> String {
    let output = process::Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .unwrap()
        .stdout;
    std::str::from_utf8(&output).unwrap().trim().to_string()
}

fn generate_version() -> Result<(), Box<dyn Error>> {
    let mut version = String::new();
    write!(
        version,
        "{} ({})\n",
        clap::crate_version!(),
        git_revision_hash()
    )?;
    write!(version, "Build type: {}\n", env::var("PROFILE").unwrap())?;
    write!(version, "Target: {}", env::var("TARGET").unwrap())?;
    generate_file("version", &version);
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    generate_version()?;
    Ok(())
}
