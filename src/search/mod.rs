//! [Search] is a "dynamic" position evaluation and one of the most imporant
//! parts of the engine. It uses move generation and knowledge about the chess
//! rules to efficiently look ahead into possible continuations and their
//! respective static evaluation to combine them into a final score that the
//! engine assigns to the position.
//!
//! [Search]: https://www.chessprogramming.org/Search

use std::io::Write;
use std::time::{Duration, Instant};

use arrayvec::ArrayVec;

use crate::chess::core::Move;
use crate::chess::position::Position;
use crate::evaluation::Score;

pub(crate) mod minimax;
mod transposition;

type Depth = u8;

/// The search depth does not grow fast and an upper limit is set for improving
/// performance.
///
/// Realistically, it is probably way lower than 256, but it should not affect
/// performance too much.
const MAX_SEARCH_DEPTH: usize = 256;

struct Context {
    position_history: ArrayVec<Position, MAX_SEARCH_DEPTH>,
    num_nodes: u64,
    // TODO: num_pruned for debugging
}

impl Context {
    fn new(position: &Position) -> Self {
        let mut position_history = ArrayVec::new();
        position_history.push(position.clone());
        Self {
            position_history,
            num_nodes: 1,
        }
    }
}

/// Adding reserve time to ensure that the engine does not exceed the time
/// limit.
// TODO: Tweak this.
const RESERVE: Duration = Duration::from_millis(500);

/// Runs the search algorithm to find the best move under given time
/// constraints.
pub fn go(position: &Position, limit: Duration, output: &mut impl Write) -> Move {
    let timer = Instant::now();

    let mut context = Context::new(position);

    let mut best_move = None;

    const MAX_DEPTH: Depth = 8;

    for depth in 1..MAX_DEPTH {
        let (next_move, score) = find_best_move_and_score(position, depth, &mut context);

        best_move = Some(next_move);
        writeln!(
            output,
            "info depth {} score {} pv {} nodes {} time {}",
            depth,
            score,
            &best_move.unwrap().to_string(),
            context.num_nodes,
            timer.elapsed().as_millis(),
        )
        .unwrap();

        if timer.elapsed() + RESERVE >= limit {
            break;
        }
    }

    writeln!(
        output,
        "info nodes {} nps {}",
        context.num_nodes,
        (context.num_nodes as f64 / timer.elapsed().as_secs_f64()) as u64,
    )
    .unwrap();
    best_move.unwrap()
}

fn find_best_move_and_score(
    position: &Position,
    depth: Depth,
    context: &mut Context,
) -> (Move, Score) {
    context.num_nodes += 1;

    let mut best_move = None;
    let mut best_score = Score::MIN;

    let alpha = Score::MIN;
    let beta = Score::MAX;

    for next_move in position.generate_moves() {
        let mut next_position = position.clone();
        next_position.make_move(&next_move);

        context.position_history.push(next_position);

        let score = -minimax::negamax(context, depth - 1, -beta, -alpha);

        let _ = context.position_history.pop();

        if score >= best_score {
            best_score = score;
            best_move = Some(next_move);
        }
    }

    (best_move.unwrap(), best_score)
}
