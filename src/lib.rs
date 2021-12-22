mod board;

use clap::Parser;
use log::info;
use sysinfo::{System, SystemExt};

#[derive(Parser, Debug)]
#[clap(about, version, author)]
pub struct Opts {}

/// Prints information about the host system.
pub fn log_system_info() {
    let sys = System::new_all();
    info!("Environment summary:");
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
    info!("RAM: {} GB", sys.total_memory() / 1_000_000); // Convert KB to GB.
    info!(
        "Processors: {}, Physical cores: {}",
        sys.processors().len(),
        sys.physical_core_count().unwrap_or_default()
    );
}
