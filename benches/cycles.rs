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

use pabi::chess::position::Position;
use pabi::util;

fn parse_stockfish_book_positions() {
    for book in util::stockfish_books() {
        for serialized_position in util::read_compressed_book(&book).lines() {
            iai::black_box(
                Position::try_from(serialized_position)
                    .expect("benchmarks are given valid positions"),
            );
        }
    }
}

// TODO: Perft.

iai::main!(parse_stockfish_book_positions);
