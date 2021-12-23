use clap::StructOpt;
use pabi::board::Position;
use pabi::Opts;
use tracing_subscriber;

fn main() {
    let opts = Opts::parse();
    tracing_subscriber::fmt::init();
    pabi::log_system_info();
    let position = Position::from_fen(&opts.fen_position);
    println!("Read position: {:?}", &position);
}
