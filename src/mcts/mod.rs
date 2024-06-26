//! [Search] is a "dynamic" position evaluation and one of the most imporant
//! parts of the engine. It uses move generation and knowledge about the chess
//! rules to efficiently look ahead into possible continuations and their
//! respective static evaluation to combine them into a final score that the
//! engine assigns to the position.
//!
//! [Search]: https://www.chessprogramming.org/Search

use std::io::Write;
use std::time::{Duration, Instant};

use crate::chess::core::Move;
use crate::chess::position::Position;
use crate::evaluation::Score;

mod state;
mod tree;
use state::State;

/// Search depth in plies.
pub type Depth = u8;

pub(crate) struct Limiter {
    pub(crate) timer: Instant,
    pub(crate) depth: Option<Depth>,
    pub(crate) time: Option<Duration>,
}

/// Adding reserve time to ensure that the engine does not exceed the time
/// limit.
// TODO: Tweak/tune this.
const RESERVE: Duration = Duration::from_millis(100);

/// Runs the search algorithm to find the best move under given time
/// constraints.
pub(crate) fn find_best_move(
    root: Position,
    max_depth: Option<Depth>,
    time: Option<Duration>,
    output: &mut impl Write,
) -> Move {
    let limiter = Limiter {
        timer: Instant::now(),
        depth: max_depth,
        time,
    };

    let mut state = State::new(root);

    let mut best_move = None;

    let max_depth = limiter.depth.unwrap_or(Depth::MAX);

    for depth in 1..=max_depth {
        let (next_move, score) = find_best_move_and_score(depth, &mut state);

        best_move = Some(next_move);
        writeln!(
            output,
            "info depth {} score {} pv {} nodes {} time {}",
            depth,
            score,
            &best_move.unwrap().to_string(),
            state.searched_nodes(),
            limiter.timer.elapsed().as_millis(),
        )
        .unwrap();

        if let Some(time_limit) = limiter.time {
            if limiter.timer.elapsed() + RESERVE >= time_limit {
                break;
            }
        }
    }

    writeln!(
        output,
        "info nodes {} nps {}",
        state.searched_nodes(),
        (state.searched_nodes() as f64 / limiter.timer.elapsed().as_secs_f64()) as u64,
    )
    .unwrap();

    best_move.unwrap()
}

fn find_best_move_and_score(depth: Depth, state: &mut State) -> (Move, Score) {
    todo!()
}

/// Runs search on a small set of positions to provide an estimate of engine's
/// performance.
///
/// Implementing `bench` CLI command is a [requirement for OpenBench].
///
/// NOTE: This function **has to run less than 60 seconds**.
///
/// See <https://github.com/AndyGrant/OpenBench/blob/master/Client/bench.py> for
/// more details.
///
/// [requirement for OpenBench]: https://github.com/AndyGrant/OpenBench/wiki/Requirements-For-Public-Engines#basic-requirements
// TODO: Add more positions and limit search in each position to few seconds
// (summing up to 60).
pub fn openbench() {
    let mut total_nodes = 0;
    let timer = Instant::now();

    for (position, depth) in [
        (Position::starting(), 6),
        (
            Position::from_fen(
                "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            )
            .unwrap(),
            5,
        ),
        (
            Position::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap(),
            5,
        ),
    ] {
        let mut state = State::new(position);

        todo!();

        total_nodes += state.searched_nodes();
    }

    let elapsed = timer.elapsed();

    println!(
        "{} nodes {} nps",
        total_nodes,
        (total_nodes as f64 / elapsed.as_secs_f64()) as u64,
    );
}
