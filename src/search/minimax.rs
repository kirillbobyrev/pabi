//! Implementation of [Minimax] algorithm with [Negamax] and [Alpha-Beta
//! pruning] extensions.
//!
//! [Minimax]: https://en.wikipedia.org/wiki/Minimax
//! [Negamax]: https://en.wikipedia.org/wiki/Negamax
//! [Alpha-Beta pruning]: https://en.wikipedia.org/wiki/Alpha%E2%80%93beta_pruning
// TODO: Implement iterative deepening.
// TODO: Implement alpha-beta pruning.
// TODO: Implement move ordering.

use std::ops::Neg;

use crate::evaluation::material::material_advantage;

#[derive(PartialEq)]
enum Evaluation {
    Value(f32),
    Checkmate(i16),
}

impl PartialOrd for Evaluation {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Self::Value(a), Self::Value(b)) => a.partial_cmp(b),
            (Self::Checkmate(a), Self::Checkmate(b)) => a.partial_cmp(b),
            (Self::Value(_), Self::Checkmate(checkmate_in_n)) => {
                if checkmate_in_n.is_negative() {
                    Some(std::cmp::Ordering::Greater)
                } else {
                    Some(std::cmp::Ordering::Less)
                }
            },
            (Self::Checkmate(checkmate_in_n), Self::Value(_)) => {
                if checkmate_in_n.is_positive() {
                    Some(std::cmp::Ordering::Greater)
                } else {
                    Some(std::cmp::Ordering::Less)
                }
            },
        }
    }
}

impl Neg for Evaluation {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Self::Value(value) => Self::Value(-value),
            Self::Checkmate(checkmate_in_n) => Self::Checkmate(-checkmate_in_n),
        }
    }
}

// TODO: Document.
fn negamax(state: &mut crate::search::SearchState, depth: u8) -> Evaluation {
    assert!(!state.stack.is_empty());
    let position = state.stack.last_mut().unwrap();
    if depth == 0 {
        let value = material_advantage(position) as f32;
        let _ = state.stack.pop();
        return Evaluation::Value(value);
    }
    // TODO: Check if the position is terminal (checkmate or stalemate).
    // TODO: Check transposition table for existing evaluation.
    // TODO: Check tablebase for existing evaluation.
    let mut best_value = Evaluation::Value(f32::NEG_INFINITY);
    // TODO: Do not copy here, figure out how to beat the borrow checker.
    let current_position = state.stack.last().unwrap().clone();
    for next_move in current_position.generate_moves() {
        let mut new_position = current_position.clone();
        new_position.make_move(&next_move);
        state.stack.push(new_position);
        let value = -negamax(state, depth - 1);
        if value > best_value {
            best_value = value;
        }
    }
    let _ = state.stack.pop();
    best_value
}
