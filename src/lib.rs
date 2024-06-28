//! Modern and high-quality chess engine. For more information, see [README].
//!
//! [README]: https://github.com/kirillbobyrev/pabi/blob/main/README.md

// TODO: Re-export types for convenience.
pub mod chess;
pub mod engine;
pub mod environment;
pub mod evaluation;
pub mod mcts;

pub use engine::Engine;

shadow_rs::shadow!(build);

/// Features the engine is built with (e.g. build type and target). Produced by
/// `build.rs`.
const BUILD_FEATURES: &str = include_str!(concat!(env!("OUT_DIR"), "/features"));

/// Returns the full engine version that can be used to identify how it was
/// built in the first place.
fn engine_version() -> String {
    format!(
        "{} (commit {}, branch {})",
        build::PKG_VERSION,
        build::SHORT_COMMIT,
        build::BRANCH
    )
}

/// Prints information about the engine version, author and GitHub repository
/// on engine startup.
pub fn print_engine_info() {
    println!("Pabi chess engine {}", engine_version());
    println!("<https://github.com/kirillbobyrev/pabi>");
}

/// Prints information the build type, features and whether the build is clean
/// on engine startup.
pub fn print_binary_info() {
    println!("Release build: {}", !shadow_rs::is_debug());
    println!("Features: {BUILD_FEATURES}");
    if !shadow_rs::git_clean() {
        println!("Warning: built with uncommitted changes");
    }
    println!();
}
