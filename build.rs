//! Retrieves information about the version of the engine from Git and the build
//! environment. This information is then written to a file in the output
//! directory and can be accessed at runtime by the engine.

use std::path::Path;
use std::{env, fs};

fn generate_file(filename: &str, contents: &str) {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join(filename);
    fs::write(dest_path, contents).unwrap();
}

// TODO: Add AVX, SSE, potentially MKL and other Candle features support.
fn generate_build_info() {
    let features = format!(
        "{}bmi2",
        if cfg!(target_feature = "bmi2") {
            "+"
        } else {
            "-"
        }
    );
    generate_file("features", &features);
}

fn main() -> shadow_rs::SdResult<()> {
    generate_build_info();
    shadow_rs::new()
}
