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
use crate::chess::core::Player::{Black, White};
use crate::chess::position::Position;
use crate::evaluation::Value;

const PAWN_VALUE: Value = 100;
const KNIGHT_VALUE: Value = 300;
const BISHOP_VALUE: Value = 300;
const ROOK_VALUE: Value = 500;
const QUEEN_VALUE: Value = 900;

fn piece_value(pieces: &crate::chess::bitboard::Pieces) -> Value {
    let mut value = 0;
    value += PAWN_VALUE * pieces.bitboard_for(Pawn).count() as Value;
    value += KNIGHT_VALUE * pieces.bitboard_for(Knight).count() as Value;
    value += BISHOP_VALUE * pieces.bitboard_for(Bishop).count() as Value;
    value += ROOK_VALUE * pieces.bitboard_for(Rook).count() as Value;
    value += QUEEN_VALUE * pieces.bitboard_for(Queen).count() as Value;
    value
}

pub(crate) fn material_advantage(position: &Position) -> Value {
    piece_value(position.board().player_pieces(White))
        - piece_value(position.board().player_pieces(Black))
}

// TODO: Test.
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn starting_position() {
        assert_eq!(material_advantage(&Position::starting()), 0);
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
            1000
        );
    }

    #[test]
    fn black_advantage() {
        assert_eq!(
            material_advantage(
                &Position::from_fen("rn1qkbnr/ppp1pppp/8/8/2BP4/4P3/PP3PPP/RbBQK1NR w KQkq - 0 5")
                    .unwrap()
            ),
            -300
        );
    }
}
