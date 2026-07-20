//! Extracts features from the position to be used as the input of the
//! policy+value Neural Network.
//!
//! All features are encoded from the perspective of the player to move: the
//! board is flipped vertically for Black, so that "our" pawns always move up.
//! This halves the input space the network has to learn and matches the
//! encoding of the action space ([`crate::chess::core::Move::policy_index`]).
#![allow(
    dead_code,
    reason = "Neural Network plumbing that is not wired into the search yet"
)]

use crate::chess::bitboard::Bitboard;
use crate::chess::core::{CastleRights, File, PieceKind};
use crate::chess::position::Position;
use crate::environment::Player;

/// Number of piece occupancy planes: 6 piece kinds for each of the two
/// players.
pub(crate) const NUM_PIECE_PLANES: usize = 12;

/// Input features of a position from the perspective of the player to move.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Features {
    /// Piece occupancy bitboards: planes 0-5 are our pieces in `PieceKind`
    /// order (pawn to king), planes 6-11 are the opponent's pieces in the same
    /// order. The board is always oriented so that our pawns move towards
    /// higher ranks.
    pub(crate) pieces: [Bitboard; NUM_PIECE_PLANES],
    /// Castling availability for both sides.
    pub(crate) our_short_castle: bool,
    pub(crate) our_long_castle: bool,
    pub(crate) their_short_castle: bool,
    pub(crate) their_long_castle: bool,
    /// File of the en passant target square, if any. The rank is implied: it
    /// is always the 6th rank from our perspective.
    pub(crate) en_passant_file: Option<File>,
    /// Number of halfmoves since the last capture or pawn move (the 50-move
    /// rule counter).
    pub(crate) halfmove_clock: u8,
}

impl Features {
    /// Extracts the features from the position.
    pub(crate) fn extract(position: &Position) -> Self {
        let (us, them) = (position.us(), position.them());
        let flip = us == Player::Black;

        let mut pieces = [Bitboard::empty(); NUM_PIECE_PLANES];
        for (side, player) in [(0, us), (6, them)] {
            let player_pieces = position.pieces(player);
            for kind in [
                PieceKind::Pawn,
                PieceKind::Knight,
                PieceKind::Bishop,
                PieceKind::Rook,
                PieceKind::Queen,
                PieceKind::King,
            ] {
                let bitboard = player_pieces.bitboard_for(kind);
                pieces[side + kind as usize] = if flip {
                    bitboard.flip_perspective()
                } else {
                    bitboard
                };
            }
        }

        let castling = position.castling();
        let (our_short, our_long, their_short, their_long) = match us {
            Player::White => (
                CastleRights::WHITE_SHORT,
                CastleRights::WHITE_LONG,
                CastleRights::BLACK_SHORT,
                CastleRights::BLACK_LONG,
            ),
            Player::Black => (
                CastleRights::BLACK_SHORT,
                CastleRights::BLACK_LONG,
                CastleRights::WHITE_SHORT,
                CastleRights::WHITE_LONG,
            ),
        };

        Self {
            pieces,
            our_short_castle: castling.contains(our_short),
            our_long_castle: castling.contains(our_long),
            their_short_castle: castling.contains(their_short),
            their_long_castle: castling.contains(their_long),
            // A vertical flip does not change the file.
            en_passant_file: position.en_passant().map(|square| square.file()),
            halfmove_clock: position.halfmove_clock(),
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn starting_position_is_symmetric() {
        // The starting position looks identical for both players, so the
        // features from White's and Black's perspective must be equal.
        let white_to_move = Features::extract(&Position::starting());
        let black_to_move = Features::extract(
            &Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1")
                .unwrap(),
        );
        assert_eq!(white_to_move, black_to_move);

        // Our pawns occupy the second rank from our perspective.
        assert_eq!(
            white_to_move.pieces[PieceKind::Pawn as usize],
            Bitboard::from_bits(0x0000_0000_0000_FF00)
        );
        assert!(white_to_move.our_short_castle);
        assert!(white_to_move.their_long_castle);
        assert_eq!(white_to_move.en_passant_file, None);
    }

    #[test]
    fn black_perspective_is_flipped() {
        // After 1. e4 it is Black to move: from Black's perspective the white
        // e4 pawn is a "their" pawn on the 5th rank (e5 after the flip).
        let position =
            Position::from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1")
                .unwrap();
        let features = Features::extract(&position);
        let their_pawns = features.pieces[6 + PieceKind::Pawn as usize];
        assert!(
            their_pawns.bits() & (1 << 36) != 0,
            "expected an opponent pawn on e5 after the flip"
        );
    }

    #[test]
    fn en_passant_and_castling() {
        let position = Position::from_fen(
            "rnbqk1nr/p3bppp/1p2p3/2ppP3/3P4/P7/1PP1NPPP/R1BQKBNR w KQkq c6 0 7",
        )
        .unwrap();
        let features = Features::extract(&position);
        assert_eq!(features.en_passant_file, Some(File::C));
        assert!(features.our_short_castle);
        assert!(features.their_short_castle);
    }
}
