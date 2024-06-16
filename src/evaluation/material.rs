//! Provides a very basic implementation of evaluation based on material
//! advantage using "[standard piece valuations]".
//!
//! While not very useful in practice, this evaluation function is great for
//! testing search and other infrastructure, because it is stable (will not
//! change because of the fixed piece "values"), easy to understand and
//! deterministic.
//!
//! [standard piece valuations]: https://en.wikipedia.org/wiki/Chess_piece_relative_value

use crate::chess::core::PieceKind::{Bishop, Knight, Pawn, Queen, Rook};
use crate::chess::position::Position;
use crate::evaluation::Score;

const PAWN_VALUE: i32 = 100;
const KNIGHT_VALUE: i32 = 300;
const BISHOP_VALUE: i32 = 300;
const ROOK_VALUE: i32 = 500;
const QUEEN_VALUE: i32 = 900;

fn piece_value(pieces: &crate::chess::bitboard::Pieces) -> i32 {
    let mut value = 0;
    value += PAWN_VALUE * pieces.bitboard_for(Pawn).count() as i32;
    value += KNIGHT_VALUE * pieces.bitboard_for(Knight).count() as i32;
    value += BISHOP_VALUE * pieces.bitboard_for(Bishop).count() as i32;
    value += ROOK_VALUE * pieces.bitboard_for(Rook).count() as i32;
    value += QUEEN_VALUE * pieces.bitboard_for(Queen).count() as i32;
    value
}

pub(crate) fn material_advantage(position: &Position) -> Score {
    let (us, them) = (position.us(), position.them());
    let (our_pieces, their_pieces) = (position.pieces(us), position.pieces(them));
    let advantage = piece_value(our_pieces) - piece_value(their_pieces);
    Score::from(advantage)
}

// TODO: Test.
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn starting_position() {
        assert_eq!(material_advantage(&Position::starting()), Score::from(0));
    }

    #[test]
    fn white_advantage() {
        assert_eq!(
            material_advantage(
                &Position::from_fen(
                    "rnb1kbnr/ppp2p1p/6p1/3pN1B1/3P4/2N5/PPP1PPPP/R2QKB1R b KQkq - 0 5"
                )
                .unwrap()
            ),
            Score::from(-1000)
        );
    }

    #[test]
    fn black_advantage() {
        assert_eq!(
            material_advantage(
                &Position::from_fen("rn1qkbnr/ppp1pppp/8/8/2BP4/4P3/PP3PPP/RbBQK1NR w KQkq - 0 5")
                    .unwrap()
            ),
            Score::from(-300)
        );
    }

    #[test]
    fn black_king_in_center() {
        assert_eq!(
            material_advantage(
                &Position::from_fen("rnbq1bnr/pp2k1pp/5p2/2pp4/8/N7/PPPPPP1P/R1BQK2R b - - 2 10")
                    .unwrap()
            ),
            Score::from(600)
        );
    }
}
