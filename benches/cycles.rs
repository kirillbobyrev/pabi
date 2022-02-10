//! iai benchmarks are measuring projected processor cycles spent on executing a
//! piece of code. They are less noisy and give a better understanding of
//! whether the performance is "objectively" changing between different
//! versions.
//!
//! It doesn't eliminate the necessity of measuring the time, though, because
//! knowing the absolute values is very important, too. Hence, the two sets of
//! benchmarks are very similar but complement each other.
//!
//! Another problem is that there seems to be no way to benchmark a specific
//! piece of code with iai: the measurements include the whole function
//! execution.

use std::fs;

use pabi::chess::position::Position;

fn parse_stockfish_book_positions() {
    for line in fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/data/positions.fen"))
        .unwrap()
        .lines()
    {
        iai::black_box(Position::try_from(line).expect("benchmarks are given valid positions"));
    }
}

// TODO: Perft.

iai::main!(parse_stockfish_book_positions);
