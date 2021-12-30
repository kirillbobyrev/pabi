//! Modern and high-quality chess engine. See [README] for information about
//! design and implementtaiton goals.
//!
//! [README]: https://github.com/kirillbobyrev/pabi/blob/main/README.md

// TODO: Move most of those to deny.
#![warn(
    missing_docs,
    rustdoc::missing_doc_code_examples,
    variant_size_differences
)]
#![deny(
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
#![deny(
    rustdoc::private_doc_tests,
    rustdoc::missing_crate_level_docs,
    rustdoc::broken_intra_doc_links,
    rustdoc::invalid_codeblock_attributes,
    rustdoc::invalid_html_tags,
    rustdoc::invalid_rust_codeblocks,
    rustdoc::bare_urls
)]

pub mod bitboard;
pub mod board;
pub mod core;
pub mod perft;

use clap::Parser;
use sysinfo::{System, SystemExt};
use tracing::info;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
pub struct Opts {
    pub fen: String,
}

/// Prints information about the host system.
pub fn log_system_info() {
    let sys = System::new_all();
    info!(
        "System: {}",
        sys.long_os_version().unwrap_or("UNKNOWN".to_string())
    );
    info!(
        "System kernel version: {}",
        sys.kernel_version().unwrap_or("UNKNOWN".to_string())
    );
    info!(
        "Host name: {}",
        sys.host_name().unwrap_or("UNKNOWN".to_string())
    );
    // Convert returned KB to GB.
    info!("RAM: {} GB", sys.total_memory() / 1_000_000);
    info!(
        "Processors: {}, Physical cores: {}",
        sys.processors().len(),
        sys.physical_core_count().unwrap_or_default()
    );
}
