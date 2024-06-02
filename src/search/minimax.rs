//! Implementation of [Minimax] algorithm with [Negamax] and [Alpha-Beta
//! pruning] extensions.
//!
//! [Minimax]: https://en.wikipedia.org/wiki/Minimax
//! [Negamax]: https://en.wikipedia.org/wiki/Negamax
//! [Alpha-Beta pruning]: https://en.wikipedia.org/wiki/Alpha%E2%80%93beta_pruning
// TODO: Implement iterative deepening.
// TODO: Implement alpha-beta pruning.
// TODO: Implement move ordering.

use crate::chess::position::Position;
use crate::evaluation::Value;
use crate::search::Score;
use crate::search::SearchResult;

// TODO: Document.
pub(crate) fn negamax(
    state: &mut crate::search::SearchState,
    depth: u8,
    static_evaluator: &dyn Fn(&Position) -> Value,
) -> SearchResult {
    debug_assert!(!state.stack.is_empty());
    let position = state.stack.last_mut().unwrap();
    if position.is_checkmate() {
        // Players alternate turns every ply and the root node is player's turn.
        let moves = (depth + 1) as i16 / 2;
        let win = depth % 2 == 1;

        let _ = state.stack.pop();
        return SearchResult {
            score: Score::Checkmate(if win { moves } else { -moves }),
            best_move: None,
        };
    }
    if position.is_stalemate() {
        let _ = state.stack.pop();
        // TODO: Maybe handle stalemate differently since it's a "precise"
        // evaluation.
        return SearchResult {
            score: Score::Relative(0),
            best_move: None,
        };
    }
    if depth == 0 {
        let value = static_evaluator(position);
        let _ = state.stack.pop();
        return SearchResult {
            score: Score::Relative(value),
            best_move: None,
        };
    }
    // TODO: Check transposition table for existing evaluation.
    // TODO: Check tablebase for existing evaluation.
    let mut best_result = SearchResult {
        score: Score::Relative(Value::MIN),
        best_move: None,
    };
    // TODO: Do not copy here, figure out how to beat the borrow checker.
    let current_position = state.stack.last().unwrap().clone();
    for next_move in current_position.generate_moves() {
        let mut new_position = current_position.clone();
        new_position.make_move(&next_move);
        state.stack.push(new_position);

        let mut search_result = negamax(state, depth - 1, static_evaluator);
        search_result.score = -search_result.score;

        // Update the best score and move that achieves it if the explored move
        // leads to the best result so far.
        if search_result.score > best_result.score {
            best_result.score = search_result.score;
            best_result.best_move = Some(next_move);
        }
    }
    let _ = state.stack.pop();
    best_result
}

#[cfg(test)]
mod test {
    use crate::{evaluation::material::material_advantage, search::SearchState};

    use super::*;

    #[test]
    fn starting_position() {
        let mut state = SearchState::new();
        state.stack.push(Position::starting());
        assert_eq!(
            negamax(&mut state, 0, &material_advantage),
            SearchResult {
                score: Score::Relative(0),
                best_move: None
            }
        );
    }

    #[test]
    fn losing_position() {}
}
