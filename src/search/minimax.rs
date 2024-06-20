//! Implementation of [Minimax] algorithm with [Negamax] and [Alpha-Beta
//! pruning] extensions.
//!
//! [Minimax]: https://en.wikipedia.org/wiki/Minimax
//! [Negamax]: https://en.wikipedia.org/wiki/Negamax
//! [Alpha-Beta pruning]: https://en.wikipedia.org/wiki/Alpha%E2%80%93beta_pruning
// TODO: Implement move ordering.

use crate::evaluation::pesto::evaluate;
use crate::evaluation::Score;
use crate::search::Context;

pub(super) fn negamax(context: &mut Context, depth: u8, alpha: Score, beta: Score) -> Score {
    debug_assert!(!context.position_history.is_empty());

    context.num_nodes += 1;

    let position = context.position_history.last_mut().unwrap();

    if position.is_checkmate() {
        // The player to move is in checkmate.
        return Score::MIN;
    }

    // TODO: is_draw: stalemate + 50 move rule + 3 repetitions.
    if position.is_stalemate() {
        // TODO: Maybe handle stalemate differently since it's a "precise"
        // evaluation.
        return Score::from(0);
    }

    if depth == 0 {
        return evaluate(position);
    }

    let mut best_eval = Score::MIN;
    let mut alpha = alpha;

    // TODO: Do not copy here, figure out how to beat the borrow checker.
    let current_position = context.position_history.last().unwrap().clone();
    for next_move in current_position.generate_moves() {
        // Update the search state.
        let mut new_position = current_position.clone();
        new_position.make_move(&next_move);
        context.position_history.push(new_position);

        let eval = -negamax(context, depth - 1, -beta, -alpha);

        drop(context.position_history.pop());

        // Update the best score and move that achieves it if the explored move
        // leads to the best result so far.
        best_eval = std::cmp::max(best_eval, eval);
        alpha = std::cmp::max(alpha, eval);

        // Beta cut-off.
        if alpha >= beta {
            break;
        }
    }

    best_eval
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chess::position::Position;
    use crate::evaluation::pesto::evaluate;

    #[test]
    fn zero_depth() {
        let mut state = Context::new(&Position::starting());
        assert_eq!(
            negamax(&mut state, 0, Score::MIN, Score::MAX),
            evaluate(&Position::starting())
        );
    }

    #[test]
    fn starting_position() {
        let mut state = Context::new(&Position::starting());
        assert!(negamax(&mut state, 1, Score::MIN, Score::MAX) >= Score::from(0));
    }

    /*
    #[test]
    fn symmetric_evaluation() {
        let original_position =
            Position::from_fen("rnbq1bnr/pp4pp/4kp2/2pp4/8/N7/PPPPPP1P/R1BQ1K1R b - - 4 11")
                .expect("valid position");
        let mut state = Context::new(&original_position);
        let original_evaluation = negamax(&mut state, 1, Score::MIN, Score::MAX);

        let symmetric_position =
            Position::from_fen("rnbq1bnr/pp4pp/4kp2/2pp4/8/N7/PPPPPP1P/R1BQ1K1R w - - 4 11")
                .expect("valid position");
        let mut state = Context::new(&symmetric_position);
        let symmetric_evaluation = negamax(&mut state, 1, Score::MIN, Score::MAX);

        assert_eq!(original_evaluation, -symmetric_evaluation);
    }
    */

    // #[test]
    // fn find_mate_losing_position() {
    //     todo!()
    // }

    // #[test]
    // fn find_mate_winning_position() {
    //     todo!()
    // }
}
