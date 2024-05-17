// TODO: Describe why this is needed and what the details are.
// This would be better compile-time but is not possible without Nightly Rust
// and unstable features right now, possibly due to a bug:
// https://github.com/rust-lang/rust/issues/93481

use std::error::Error;
use std::fmt::Write;
use std::path::Path;
use std::{env, fs, process};

// TODO: This can fail at several levels: be more principled about it.
fn git_revision_hash() -> String {
    let output = process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap()
        .stdout;
    std::str::from_utf8(&output).unwrap().trim().to_string()
}

fn generate_file(filename: &str, contents: &str) {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join(filename);
    fs::write(dest_path, contents).unwrap();
}

fn generate_version() -> Result<(), Box<dyn Error>> {
    let mut version = String::new();
    writeln!(
        version,
        "{} ({})",
        env!("CARGO_PKG_VERSION"),
        git_revision_hash()
    )?;
    writeln!(version, "Build type: {}", env::var("PROFILE").unwrap())?;
    write!(version, "Target: {}", env::var("TARGET").unwrap())?;
    generate_file("version", &version);
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    generate_version()
}
