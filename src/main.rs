use clap::StructOpt;
use pabi::board::Board;
use pabi::Opts;
use tracing_subscriber;

fn main() {
    let opts = Opts::parse();
    tracing_subscriber::fmt::init();
    pabi::log_system_info();
    let board = Board::from_fen(&opts.fen).unwrap();
    println!("{:#?}", &board);
}
