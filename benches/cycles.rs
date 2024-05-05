//! iai benchmarks are measuring projected processor cycles spent on executing a
//! piece of code. They have more precision and give a better understanding of
//! whether the performance is "objectively" changing between different
//! versions. This comes at a cost of slowing the code down by the
//! instrumentation (Valgrind), which reduces the volume of benchmarks that can
//! be run.
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

// Low depths of known perft results (https://www.chessprogramming.org/Perft_Results).
fn perft() {
    for (position, depth, nodes) in [
        // Position 1 (starting).
        (Position::starting(), 5, 4865609),
        // Position 2.
        (
            Position::from_fen(
                "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            )
            .unwrap(),
            4,
            4085603,
        ),
        // Position 3.
        (
            Position::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap(),
            5,
            674624,
        ),
        // Position 4.
        (
            Position::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1")
                .unwrap(),
            4,
            422333,
        ),
        // Position 5.
        (
            Position::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8")
                .unwrap(),
            4,
            2103487,
        ),
        // Position 6.
        (
            Position::from_fen(
                "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
            )
            .unwrap(),
            4,
            3894594,
        ),
        (
            Position::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1")
                .unwrap(),
            4,
            422333,
        ),
    ]
    .iter()
    {
        assert_eq!(pabi::chess::position::perft(position, *depth), *nodes);
    }
}

iai::main!(parse_stockfish_book_positions, perft);
