//! This module implements "static" [evaluation], i.e. predicting the relative
//! value/score of given position without [`crate::search`].
//!
//! For convenience, the score is returned in centipawn units.
//!
//! The current implementation is a classical [Tapered Eval] based on the
//! [PeSTO] piece-square tables. It serves as a baseline value function for the
//! search until the Neural Network evaluation is implemented.
//!
//! [evaluation]: https://www.chessprogramming.org/Evaluation
//! [Tapered Eval]: https://www.chessprogramming.org/Tapered_Eval
//! [PeSTO]: https://www.chessprogramming.org/PeSTO%27s_Evaluation_Function

pub(crate) mod features;
pub(crate) mod network;

use crate::chess::position::Position;
use crate::environment::Player;

/// Piece-square tables combined with material values, generated in `build.rs`.
/// Indexed by [`crate::chess::core::Piece::plane()`] and square.
static MIDDLEGAME_TABLE: [[i32; 64]; 12] =
    include!(concat!(env!("OUT_DIR"), "/pesto_middlegame_table"));
static ENDGAME_TABLE: [[i32; 64]; 12] = include!(concat!(env!("OUT_DIR"), "/pesto_endgame_table"));

/// Game phase contribution of each piece kind in `PieceKind` order (pawn to
/// king). The maximum phase (all pieces on the board) is 24.
const PHASE_WEIGHT: [i32; 6] = [0, 1, 1, 2, 4, 0];
const MAX_PHASE: i32 = 24;

/// Returns the static evaluation of the position in centipawns from the
/// perspective of the player to move: positive means the player to move is
/// better.
#[must_use]
pub fn evaluate(position: &Position) -> i32 {
    let mut middlegame = 0;
    let mut endgame = 0;
    let mut phase = 0;

    position.for_each_piece(|square, piece| {
        let plane = piece.plane();
        let sign = match piece.player {
            Player::White => 1,
            Player::Black => -1,
        };
        middlegame += sign * MIDDLEGAME_TABLE[plane][square as usize];
        endgame += sign * ENDGAME_TABLE[plane][square as usize];
        phase += PHASE_WEIGHT[piece.kind as usize];
    });

    // Guard against promotions pushing the phase above the cap.
    let phase = phase.min(MAX_PHASE);
    let white_score = (middlegame * phase + endgame * (MAX_PHASE - phase)) / MAX_PHASE;

    match position.us() {
        Player::White => white_score,
        Player::Black => -white_score,
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn starting_position_is_balanced() {
        assert_eq!(evaluate(&Position::starting()), 0);
    }

    #[test]
    fn material_advantage() {
        // White is up a queen: strongly positive for White to move, strongly
        // negative for Black to move.
        let up_a_queen = "rnb1kbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let position = Position::from_fen(up_a_queen).unwrap();
        assert!(evaluate(&position) > 500);

        let up_a_queen_black_to_move = "rnb1kbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1";
        let position = Position::from_fen(up_a_queen_black_to_move).unwrap();
        assert!(evaluate(&position) < -500);
    }

    #[test]
    fn symmetric_for_both_players() {
        // The same position should evaluate to the exact opposite score when
        // only the side to move is flipped.
        let fen_white = "r1bqkbnr/pppp1ppp/2n5/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3";
        let fen_black = "r1bqkbnr/pppp1ppp/2n5/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 2 3";
        let white_to_move = evaluate(&Position::from_fen(fen_white).unwrap());
        let black_to_move = evaluate(&Position::from_fen(fen_black).unwrap());
        assert_eq!(white_to_move, -black_to_move);
    }

    #[test]
    fn advanced_pawns_are_valuable() {
        // A white pawn on the 7th rank should be worth much more than one on
        // its starting square.
        let advanced = evaluate(&Position::from_fen("4k3/2P5/8/8/8/8/8/4K3 w - - 0 1").unwrap());
        let starting = evaluate(&Position::from_fen("4k3/8/8/8/8/8/2P5/4K3 w - - 0 1").unwrap());
        assert!(advanced > starting);
    }
}
