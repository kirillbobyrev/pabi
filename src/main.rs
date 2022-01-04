use clap::StructOpt;
use pabi::chess::position::Position;
use pabi::Opts;
// TODO: Set up tracer.
use tracing_subscriber;

fn main() {
    let opts = Opts::parse();
    tracing_subscriber::fmt::init();
    pabi::log_system_info();
    let position = Position::from_fen(&opts.fen).unwrap();
    println!("{:?}", &position);
}
