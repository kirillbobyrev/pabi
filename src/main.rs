use clap::StructOpt;
use pabi::chess::position::Position;
use pabi::Opts;
use tracing_subscriber;

fn main() {
    let opts = Opts::parse();
    // TODO: Allow configuring tracer.
    tracing_subscriber::fmt::init();
    pabi::log_system_info();
    let position = Position::try_from(opts.fen.as_str()).unwrap();
    println!("{position:?}");
}
