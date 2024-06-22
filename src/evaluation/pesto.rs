//! Provides a very basic implementation of evaluation based on [PeSTO].
//!
//! While it won't be very strong, it should be enough to get started with and
//! will be replaced in the future.
//!
//! The implementation is adjusted to work with the pieces and board
//! representation (different planes are used to encode piece kinds) used in the
//! engine.
//!
//! [PeSTO]: https://www.chessprogramming.org/PeSTO%27s_Evaluation_Function

use crate::chess::core::{Player, Square};
use crate::chess::position::Position;
use crate::evaluation::Score;

const GAMEPHASE_INCREMENT: [i32; 12] = [0, 0, 4, 4, 2, 2, 1, 1, 1, 1, 0, 0];

const MIDDLEGAME_VALUES: [[i32; 64]; 12] =
    include!(concat!(env!("OUT_DIR"), "/pesto_middlegame_table"));
const ENDGAME_VALUES: [[i32; 64]; 12] = include!(concat!(env!("OUT_DIR"), "/pesto_endgame_table"));

pub fn evaluate(position: &Position) -> Score {
    let mut middlegame_white = 0;
    let mut middlegame_black = 0;
    let mut endgame_white = 0;
    let mut endgame_black = 0;

    let mut game_phase = 0;

    for square in Square::iter() {
        if let Some(piece) = position.at(square) {
            if piece.owner == Player::White {
                middlegame_white += MIDDLEGAME_VALUES[piece.plane()][square as usize];
                endgame_white += ENDGAME_VALUES[piece.plane()][square as usize];
            } else {
                middlegame_black += MIDDLEGAME_VALUES[piece.plane()][square as usize];
                endgame_black += ENDGAME_VALUES[piece.plane()][square as usize];
            }
            game_phase += GAMEPHASE_INCREMENT[piece.plane()];
        };
    }

    let middlegame_score = match position.us() {
        Player::White => middlegame_white - middlegame_black,
        Player::Black => middlegame_black - middlegame_white,
    };
    let endgame_score = match position.us() {
        Player::White => endgame_white - endgame_black,
        Player::Black => endgame_black - endgame_white,
    };

    let middlegame_phase = std::cmp::min(game_phase, 24);
    let endgame_phase = 24 - middlegame_phase;

    Score::cp((middlegame_score * middlegame_phase + endgame_score * endgame_phase) / 24)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chess::core::{Piece, PieceKind, Player, Square};

    // Check that the tables are correctly built and loaded.
    #[test]
    fn table_values() {
        let mut piece = Piece {
            owner: Player::White,
            kind: PieceKind::King,
        };
        assert_eq!(MIDDLEGAME_VALUES[piece.plane()][Square::A1 as usize], -65);
        assert_eq!(ENDGAME_VALUES[piece.plane()][Square::A1 as usize], -74);

        piece.owner = Player::Black;
        assert_eq!(MIDDLEGAME_VALUES[piece.plane()][Square::A1 as usize], -15);
        assert_eq!(ENDGAME_VALUES[piece.plane()][Square::A1 as usize], -53);

        piece = Piece {
            owner: Player::White,
            kind: PieceKind::Pawn,
        };
        assert_eq!(
            MIDDLEGAME_VALUES[piece.plane()][Square::B4 as usize],
            13 + 82
        );
        assert_eq!(ENDGAME_VALUES[piece.plane()][Square::E4 as usize], -2 + 94);

        piece = Piece {
            owner: Player::Black,
            kind: PieceKind::Bishop,
        };
        assert_eq!(
            MIDDLEGAME_VALUES[piece.plane()][Square::D3 as usize],
            15 + 365
        );
    }

    #[test]
    fn starting_position() {
        assert_eq!(evaluate(&Position::starting()), Score::cp(0));
    }

    #[test]
    fn simmetry() {
        assert_eq!(
            evaluate(
                &Position::from_fen("rnbq1bnr/pp4pp/4kp2/2pp4/8/N7/PPPPPP1P/R1BQ1K1R b - - 4 11")
                    .expect("valid position")
            ),
            -evaluate(
                &Position::from_fen("rnbq1bnr/pp4pp/4kp2/2pp4/8/N7/PPPPPP1P/R1BQ1K1R w - - 4 11")
                    .expect("valid position")
            )
        );
    }
}
