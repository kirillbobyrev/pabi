//! Modern and high-quality chess engine. For more information, see [README].
//!
//! [README]: https://github.com/kirillbobyrev/pabi/blob/main/README.md

// TODO: Gradually move most of warnings to deny.
#![warn(
    missing_docs,
    variant_size_differences,
    absolute_paths_not_starting_with_crate,
    macro_use_extern_crate,
    unused_extern_crates,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    let_underscore_drop
)]
#![warn(
    clippy::correctness,
    clippy::suspicious,
    clippy::style,
    clippy::complexity,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]
#![deny(
    // rustc
    unreachable_pub,
    keyword_idents,
    keyword_idents_2018,
    keyword_idents_2024,
    unused_results,
    trivial_casts,
    trivial_numeric_casts,
    single_use_lifetimes,
    // clippy
    clippy::perf,
    // rustdoc
    rustdoc::broken_intra_doc_links,
    rustdoc::invalid_rust_codeblocks,
    rustdoc::unescaped_backticks
)]

// TODO: Re-export types for convenience.
pub mod chess;
pub mod evaluation;
pub mod search;

mod engine;
pub use engine::Engine;
use shadow_rs::shadow;

shadow!(build);

/// Build type and target. Produced by `build.rs`.
const FEATURES: &str = include_str!(concat!(env!("OUT_DIR"), "/features"));

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

/// Prints informations about the engine version, author and GitHub repository
/// on engine startup.
pub fn print_engine_info() {
    println!("Pabi chess engine {}", engine_version());
    println!("<https://github.com/kirillbobyrev/pabi>");
}

/// Prints information the build type, features and whether the build is clean
/// on engine startup.
pub fn print_binary_info() {
    println!("Release build: {}", !shadow_rs::is_debug());
    println!("Features: {FEATURES}");
    if !shadow_rs::git_clean() {
        println!("Warning: built with uncommitted changes");
    }
    println!();
}
