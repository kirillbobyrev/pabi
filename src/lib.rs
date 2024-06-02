//! Modern and high-quality chess engine. For more information, see
//!
//! - [README] explaining about design and implementation goals
//! - [CONTRIBUTING] for introduction into the codebase and design choices.
//!
//! [README]: https://github.com/kirillbobyrev/pabi/blob/main/README.md
//! [CONTRIBUTING]: https://github.com/kirillbobyrev/pabi/wiki/CONTRIBUTING.md

// TODO: Gradually move most of warnings to deny.
#![warn(missing_docs, variant_size_differences)]
// Rustc lints.
#![warn(
    absolute_paths_not_starting_with_crate,
    keyword_idents,
    macro_use_extern_crate,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unused_extern_crates,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    unused_results
)]
// Rustdoc lints.
#![warn(
    rustdoc::missing_doc_code_examples,
    rustdoc::private_doc_tests,
    rustdoc::missing_crate_level_docs,
    rustdoc::broken_intra_doc_links,
    rustdoc::invalid_codeblock_attributes,
    rustdoc::invalid_html_tags,
    rustdoc::invalid_rust_codeblocks,
    rustdoc::bare_urls
)]
// Clippy lints.
#![warn(
    clippy::correctness,
    clippy::suspicious,
    clippy::style,
    clippy::complexity,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]
// Performance is extremely important.
#![deny(clippy::perf)]

// TODO: Re-export types for convenience.
pub mod chess;
pub mod evaluation;
pub mod search;

mod engine;
pub use engine::Engine;
use shadow_rs::shadow;

shadow!(build);

/// Build type and target. Produced by `build.rs`.
pub const FEATURES: &str = include_str!(concat!(env!("OUT_DIR"), "/features"));

pub(crate) fn get_version() -> String {
    format!(
        "{} (commit {}, branch {})",
        build::PKG_VERSION,
        build::SHORT_COMMIT,
        build::BRANCH
    )
}

pub fn print_engine_info() {
    println!("Pabi Chess Engine");
    println!("Version {}", get_version());
    println!("https://github.com/kirillbobyrev/pabi");
}

/// Prints information about the binary to the standard output. This includes
/// the version, build type and what features are enabled.
pub fn print_binary_info() {
    println!("Debug: {}", shadow_rs::is_debug());
    println!("Features: {FEATURES}");
    if !shadow_rs::git_clean() {
        println!("Warning: built with uncommitted changes");
    }
    println!();
}
