//! Implementation of [Minimax] algorithm with [Negamax] and [Alpha-Beta
//! pruning] extensions.
//!
//! [Minimax]: https://en.wikipedia.org/wiki/Minimax
//! [Negamax]: https://en.wikipedia.org/wiki/Negamax
//! [Alpha-Beta pruning]: https://en.wikipedia.org/wiki/Alpha%E2%80%93beta_pruning
// TODO: Implement iterative deepening.
// TODO: Implement alpha-beta pruning.
// TODO: Implement move ordering.

use std::num::NonZeroI16;

use crate::chess::position::Position;
use crate::evaluation::Value;
use crate::search::Score;

// TODO: Document.
pub(crate) fn negamax(
    state: &mut crate::search::SearchState,
    depth: u8,
    static_evaluator: &dyn Fn(&Position) -> Value,
) -> Score {
    assert!(!state.stack.is_empty());
    let position = state.stack.last_mut().unwrap();
    if depth == 0 {
        let value = static_evaluator(position);
        let _ = state.stack.pop();
        return Score::Relative(value);
    }
    if position.is_checkmate() {
        //
        let moves = NonZeroI16::new(depth as i16 / 2 + 1).unwrap();
        // Players alternate turns every ply and the root node is player's turn.
        let win = depth % 2 == 1;
        let value = if win {
            Score::Checkmate(moves)
        } else {
            Score::Checkmate(-moves)
        };
        let _ = state.stack.pop();
        return value;
    }
    // TODO: Check if the position is terminal (checkmate or stalemate).
    // TODO: Check transposition table for existing evaluation.
    // TODO: Check tablebase for existing evaluation.
    let mut best_value = Score::Relative(Value::MIN);
    // TODO: Do not copy here, figure out how to beat the borrow checker.
    let current_position = state.stack.last().unwrap().clone();
    for next_move in current_position.generate_moves() {
        let mut new_position = current_position.clone();
        new_position.make_move(&next_move);
        state.stack.push(new_position);
        let value = -negamax(state, depth - 1, static_evaluator);
        if value > best_value {
            best_value = value;
        }
    }
    let _ = state.stack.pop();
    best_value
}
