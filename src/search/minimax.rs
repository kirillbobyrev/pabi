//! Implementation of [Minimax] algorithm with [Negamax] and [Alpha-Beta
//! pruning] extensions.
//!
//! [Minimax]: https://en.wikipedia.org/wiki/Minimax
//! [Negamax]: https://en.wikipedia.org/wiki/Negamax
//! [Alpha-Beta pruning]: https://en.wikipedia.org/wiki/Alpha%E2%80%93beta_pruning
// TODO: Implement iterative deepening.
// TODO: Implement alpha-beta pruning.
// TODO: Implement move ordering.

use arrayvec::ArrayVec;

use crate::chess::position::{self, Position};
use crate::evaluation::material::material_advantage;

struct SearchState {
    stack: ArrayVec<Position, 256>,
}

// TODO: Document.
fn negamax(node: &mut SearchState, depth: u8) -> f32 {
    assert!(!node.stack.is_empty());
    let position = node.stack.last_mut().unwrap();
    if depth == 0 {
        return material_advantage(&position) as f32;
    }
    // TODO: Check if the position is terminal (checkmate or draw).
    // TODO: Check transposition table for existing evaluation.
    let mut best_value = f32::NEG_INFINITY;
    // TODO: Do not copy here, figure out how to beat the borrow checker.
    let current_position = node.stack.last().unwrap().clone();
    for next_move in current_position.generate_moves() {
        let mut new_position = current_position.clone();
        new_position.make_move(&next_move);
        node.stack.push(new_position);
        let value = -negamax(node, depth - 1);
        if value > best_value {
            best_value = value;
        }
    }

    best_value
}
