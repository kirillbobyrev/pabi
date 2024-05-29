//! Modern and high-quality chess engine. For more information, see
//!
//! - [README] explaining about design and implementation goals
//! - [ARCHITECTURE] for introduction into the codebase and design choices.
//!
//! [README]: https://github.com/kirillbobyrev/pabi/blob/main/README.md
//! [ARCHITECTURE]: https://github.com/kirillbobyrev/pabi/wiki/ARCHITECTURE.md

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
pub mod engine;
pub mod evaluation;
pub mod search;

mod interface;
pub use interface::uci;

/// Full version of the engine, including commit hash. Produced by `build.rs`.
pub const VERSION: &str = include_str!(concat!(env!("OUT_DIR"), "/version"));
/// Build type and target. Produced by `build.rs`.
pub const BUILD_INFO: &str = include_str!(concat!(env!("OUT_DIR"), "/build_info"));

/// Prints information about the host system.
pub fn print_system_info() {
    if cfg!(target_feature = "bmi2") {
        println!("BMI2 is supported, move generation will use PEXT and PDEP to speed up");
    } else {
        println!("WARNING: BMI2 is not supported, move generation will be significantly slower");
    }
}
