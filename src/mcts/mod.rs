//! Implements [Monte Carlo Tree Search] (MCTS) algorithm.
//!
//! [Monte Carlo Tree Search]: https://en.wikipedia.org/wiki/Monte_Carlo_tree_search

use std::io::Write;
use std::time::{Duration, Instant};

use crate::chess::core::Move;
use crate::chess::position::Position;
use crate::evaluation::QValue;

mod environment;

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
    todo!()
}

fn find_best_move_and_score(depth: Depth, state: &mut State) -> (Move, QValue) {
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
pub fn openbench() {
    todo!()
}
