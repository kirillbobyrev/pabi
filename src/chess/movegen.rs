//! Move handling: [generation] (using [BMI PEXT Bitboards]), application and
//! (de)serialization.
//!
//! NOTE: [BMI Instruction Set] (and specifically efficient [PEXT]) is not
//! widely available on all processors (e.g. the AMD only started providing an
//! *efficient* PEXT since Ryzen 3). The current implementation will rely on
//! PEXT for performance because it is the most efficient move generator
//! technique available.
//!
//! [generation]: https://www.chessprogramming.org/Table-driven_Move_Generation
//! [BMI2 Pext Bitboards]: https://www.chessprogramming.org/BMI2#PEXTBitboards
//! [BMI Instruction Set]: https://en.wikipedia.org/wiki/X86_Bit_manipulation_instruction_set
//! [PEXT]: https://en.wikipedia.org/wiki/X86_Bit_manipulation_instruction_set#Parallel_bit_deposit_and_extract

// TODO: Fall back to Fancy Magic Bitboards if BMI2 + PEXT are not available for
// portability? Maybe for now just implement
// https://www.chessprogramming.org/BMI2#Serial_Implementation#Serial_Implementation2
// TODO: Look at and compare speed with https://github.com/jordanbray/chess
// TODO: Also implement divide and use <https://github.com/jniemann66/juddperft> to validate the
// results.
// TODO: Maybe use python-chess testset of perft moves:
// https://github.com/niklasf/python-chess/blob/master/examples/perft/random.perft
// TODO: Compare with other engines and perft generators, e.g. Berserk,
// shakmaty (https://github.com/jordanbray/chess_perft).
// TODO: Check movegen comparison (https://github.com/Gigantua/Chess_Movegen).

use strum::IntoEnumIterator;

use crate::chess::bitboard::Bitboard;
use crate::chess::core::{CastlingSide, PieceKind, Square, BOARD_SIZE};
use crate::chess::position::Position;

#[derive(Debug)]
pub enum Move {
    Regular {
        from: Square,
        to: Square,
        /// A pawn can be promoted into [`PieceKind::Queen`],
        /// [`PieceKind::Rook`], [`PieceKind::Bishop`] or [`PieceKind::Knight`].
        promotion: Option<Promotion>,
    },
    EnPassant {
        from: Square,
        to: Square,
    },
    Castle {
        side: CastlingSide,
    },
}

impl ToString for Move {
    fn to_string(&self) -> String {
        match &self {
            Move::Regular {
                from,
                to,
                promotion,
            } => {
                format!(
                    "{}{}{}",
                    from,
                    to,
                    match promotion {
                        Some(promotion) => promotion.to_string(),
                        None => "".to_string(),
                    }
                )
            },
            Move::EnPassant { from, to } => from.to_string() + &to.to_string(),
            Move::Castle { side } => match *side {
                CastlingSide::Short => "O-O".to_string(),
                CastlingSide::Long => "O-O-O".to_string(),
            },
        }
    }
}

impl Move {
    /// Serializes a move in [Standard Algebraic Notation] format.
    ///
    /// [Standard Algebraic Notation]: https://en.wikipedia.org/wiki/Algebraic_notation_(chess)
    #[must_use]
    pub fn to_san(&self, _: &Position) -> String {
        todo!();
    }
}

/// The only valid promotions are to Queen, Rook, Bishop or Knight: this is a
/// subset of [`PieceKind`].
// TODO: Maybe somehow narrow down PieceKind enum instead of creating a separate type?
#[derive(Clone, Copy, Debug)]
#[allow(missing_docs)]
pub enum Promotion {
    Queen,
    Rook,
    Bishop,
    Knight,
}

// TODO: Calling .to_string() and then taking references of the literals again
// might not be a great idea but the ergonomics of ToString is attractive. Check
// and profile if inlining the literals will be faster.
impl ToString for Promotion {
    fn to_string(&self) -> String {
        match &self {
            Promotion::Queen => "q".to_string(),
            Promotion::Rook => "r".to_string(),
            Promotion::Bishop => "b".to_string(),
            Promotion::Knight => "k".to_string(),
        }
    }
}

/// Produces a list of legal moves (i.e. the moves that do not leave the King in
/// check).
///
/// This is a performance and correctness-critical path: every modification
/// should be benchmarked and carefully tested.
#[must_use]
pub fn generate_moves(position: &Position) -> Vec<Move> {
    // TODO: let mut vec = Vec::with_capacity(N=60); and tweak the specific number.
    let mut result = vec![];
    // TODO: Completely delegate to Position since it already has side_to_move?
    let our_pieces = position.board.our_pieces(position.side_to_move);
    // Cache squares occupied by each player.
    // TODO: for from in our_pieces.iter()?
    for from in Square::iter() {
        // TODO: Filter out pins.
        // Only generate moves when the square is non-empty and has the piece of a
        // correct color on it.
        let piece = match position.board.at(from) {
            None => continue,
            Some(piece) => piece,
        };
        if piece.owner != position.side_to_move {
            continue;
        }
        let targets = match piece.kind {
            PieceKind::King => todo!(),
            // Sliding pieces.
            PieceKind::Queen | PieceKind::Rook | PieceKind::Bishop => todo!(),
            PieceKind::Knight => KNIGHT_ATTACKS[from as usize],
            PieceKind::Pawn => todo!(),
        };
        // Loop over the target squares and produce moves for those which are
        // not occupied by our pieces. The empty squares or opponent pieces
        // (captures) are valid.
        for to in targets.iter() {
            if !our_pieces.all().is_set(to) {
                result.push(Move::Regular {
                    from,
                    to,
                    promotion: None,
                })
            }
        }
    }
    // TODO: Check afterstate? Our king should not be checked.
    // TODO: Castling.
    // TODO: Pawn moves + en passant.
    result
}

// Pre-calculated attacks of a knight from each square.
const KNIGHT_ATTACKS: [Bitboard; BOARD_SIZE as usize] = [
    Bitboard::from_bits(0x0000_0000_0002_0400),
    Bitboard::from_bits(0x0000_0000_0005_0800),
    Bitboard::from_bits(0x0000_0000_000A_1100),
    Bitboard::from_bits(0x0000_0000_0014_2200),
    Bitboard::from_bits(0x0000_0000_0028_4400),
    Bitboard::from_bits(0x0000_0000_0050_8800),
    Bitboard::from_bits(0x0000_0000_00A0_1000),
    Bitboard::from_bits(0x0000_0000_0040_2000),
    Bitboard::from_bits(0x0000_0000_0204_0004),
    Bitboard::from_bits(0x0000_0000_0508_0008),
    Bitboard::from_bits(0x0000_0000_0A11_0011),
    Bitboard::from_bits(0x0000_0000_1422_0022),
    Bitboard::from_bits(0x0000_0000_2844_0044),
    Bitboard::from_bits(0x0000_0000_5088_0088),
    Bitboard::from_bits(0x0000_0000_A010_0010),
    Bitboard::from_bits(0x0000_0000_4020_0020),
    Bitboard::from_bits(0x0000_0002_0400_0402),
    Bitboard::from_bits(0x0000_0005_0800_0805),
    Bitboard::from_bits(0x0000_000A_1100_110A),
    Bitboard::from_bits(0x0000_0014_2200_2214),
    Bitboard::from_bits(0x0000_0028_4400_4428),
    Bitboard::from_bits(0x0000_0050_8800_8850),
    Bitboard::from_bits(0x0000_00A0_1000_10A0),
    Bitboard::from_bits(0x0000_0040_2000_2040),
    Bitboard::from_bits(0x0000_0204_0004_0200),
    Bitboard::from_bits(0x0000_0508_0008_0500),
    Bitboard::from_bits(0x0000_0A11_0011_0A00),
    Bitboard::from_bits(0x0000_1422_0022_1400),
    Bitboard::from_bits(0x0000_2844_0044_2800),
    Bitboard::from_bits(0x0000_5088_0088_5000),
    Bitboard::from_bits(0x0000_A010_0010_A000),
    Bitboard::from_bits(0x0000_4020_0020_4000),
    Bitboard::from_bits(0x0002_0400_0402_0000),
    Bitboard::from_bits(0x0005_0800_0805_0000),
    Bitboard::from_bits(0x000A_1100_110A_0000),
    Bitboard::from_bits(0x0014_2200_2214_0000),
    Bitboard::from_bits(0x0028_4400_4428_0000),
    Bitboard::from_bits(0x0050_8800_8850_0000),
    Bitboard::from_bits(0x00A0_1000_10A0_0000),
    Bitboard::from_bits(0x0040_2000_2040_0000),
    Bitboard::from_bits(0x0204_0004_0200_0000),
    Bitboard::from_bits(0x0508_0008_0500_0000),
    Bitboard::from_bits(0x0A11_0011_0A00_0000),
    Bitboard::from_bits(0x1422_0022_1400_0000),
    Bitboard::from_bits(0x2844_0044_2800_0000),
    Bitboard::from_bits(0x5088_0088_5000_0000),
    Bitboard::from_bits(0xA010_0010_A000_0000),
    Bitboard::from_bits(0x4020_0020_4000_0000),
    Bitboard::from_bits(0x0400_0402_0000_0000),
    Bitboard::from_bits(0x0800_0805_0000_0000),
    Bitboard::from_bits(0x1100_110A_0000_0000),
    Bitboard::from_bits(0x2200_2214_0000_0000),
    Bitboard::from_bits(0x4400_4428_0000_0000),
    Bitboard::from_bits(0x8800_8850_0000_0000),
    Bitboard::from_bits(0x1000_10A0_0000_0000),
    Bitboard::from_bits(0x2000_2040_0000_0000),
    Bitboard::from_bits(0x0004_0200_0000_0000),
    Bitboard::from_bits(0x0008_0500_0000_0000),
    Bitboard::from_bits(0x0011_0A00_0000_0000),
    Bitboard::from_bits(0x0022_1400_0000_0000),
    Bitboard::from_bits(0x0044_2800_0000_0000),
    Bitboard::from_bits(0x0088_5000_0000_0000),
    Bitboard::from_bits(0x0010_A000_0000_0000),
    Bitboard::from_bits(0x0020_4000_0000_0000),
];

#[cfg(test)]
mod test {}
