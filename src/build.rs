//! Retrieves information about the version of the engine from Git and the build
//! environment. This information is then written to a file in the output
//! directory and can be accessed at runtime by the engine.

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

fn generate_version() {
    let version = format!("{} ({})", env!("CARGO_PKG_VERSION"), git_revision_hash());
    generate_file("version", &version);
    let build_info = format!(
        "Build type: {}, Target: {}",
        env::var("PROFILE").unwrap(),
        env::var("TARGET").unwrap()
    );
    generate_file("build_info", &build_info);
}

fn main() {
    generate_version();
}
