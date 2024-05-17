//! Modern and high-quality chess engine. For more information, see
//!
//! - [README] explaining about design and implementation goals
//! - [Resources] for information on important papers, other engines and
//!   prominent research ideas
//!
//! [README]: https://github.com/kirillbobyrev/pabi/blob/main/README.md
//! [Resources]: https://github.com/kirillbobyrev/pabi/wiki/Resources

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
    clippy::perf,
    clippy::pedantic,
    clippy::style,
    clippy::nursery,
    clippy::complexity,
    clippy::correctness,
    clippy::cargo
)]

// TODO: Re-export types for convenience.
pub mod chess;
pub mod evaluation;
pub mod interface;

pub mod util;

use sysinfo::System;

pub const VERSION: &str = include_str!(concat!(env!("OUT_DIR"), "/version"));

/// Prints information about the host system.
pub fn print_system_info() {
    let sys = System::new_all();
    println!(
        "System: {}",
        System::long_os_version().unwrap_or_else(|| "UNKNOWN".to_string())
    );
    println!(
        "System kernel version: {}",
        System::kernel_version().unwrap_or_else(|| "UNKNOWN".to_string())
    );
    println!(
        "Host name: {}",
        System::host_name().unwrap_or_else(|| "UNKNOWN".to_string())
    );
    // Convert returned KB to GB.
    println!("RAM: {} GB", sys.total_memory() / 1_000_000);
    match sys.physical_core_count() {
        Some(cores) => println!("Physical cores: {cores}"),
        None => println!("Physical cores: UNKNOWN"),
    }
}
