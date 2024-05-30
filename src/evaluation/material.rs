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

const PAWN_VALUE: u32 = 1;
const KNIGHT_VALUE: u32 = 3;
const BISHOP_VALUE: u32 = 3;
const ROOK_VALUE: u32 = 5;
const QUEEN_VALUE: u32 = 9;

fn value(pieces: &crate::chess::bitboard::Pieces) -> u32 {
    let mut value = 0;
    value += PAWN_VALUE * pieces.bitboard_for(Pawn).count();
    value += KNIGHT_VALUE * pieces.bitboard_for(Knight).count();
    value += BISHOP_VALUE * pieces.bitboard_for(Bishop).count();
    value += ROOK_VALUE * pieces.bitboard_for(Rook).count();
    value += QUEEN_VALUE * pieces.bitboard_for(Queen).count();
    value
}

pub(crate) fn material_advantage(position: &Position) -> i32 {
    value(position.board().player_pieces(White)) as i32
        - value(position.board().player_pieces(Black)) as i32
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
            10
        );
    }

    #[test]
    fn black_advantage() {
        assert_eq!(
            material_advantage(
                &Position::from_fen("rn1qkbnr/ppp1pppp/8/8/2BP4/4P3/PP3PPP/RbBQK1NR w KQkq - 0 5")
                    .unwrap()
            ),
            -3
        );
    }
}
