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
// TODO: Another source for comparison:
// https://github.com/sfleischman105/Pleco/blob/b825cecc258ad25cba65919208727994f38a06fb/pleco/src/board/movegen.rs#L68-L85
// TODO: Maybe use python-chess testset of perft moves:
// https://github.com/niklasf/python-chess/blob/master/examples/perft/random.perft
// TODO: Compare with other engines and perft generators, e.g. Berserk,
// shakmaty (https://github.com/jordanbray/chess_perft).
// TODO: Check movegen comparison (https://github.com/Gigantua/Chess_Movegen).

use strum::IntoEnumIterator;

use crate::chess::bitboard::Bitboard;
use crate::chess::core::{CastlingSide, PieceKind, Square, BOARD_SIZE};
use crate::chess::position::Position;

/// Represents any kind of a legal chess move. A move is the only way to mutate
/// the [`Position`] and change the board state. Moves are not sorted according
/// to their potential "value" by the move generator. The move representation
/// has one-to-one correspodance with the UCI move representation and can be
/// (de)serialized from to it. The moves can also be indexed to be fed as an
/// input to the Neural Network evaluators that would be able assess their
/// potential without evaluation the post-state.
///
/// For a move to be serialized in Standard Algebraic Notation (SAN), a move
/// also requires the [`Position`] it will be applied in, because SAN requires
/// additional flags (e.g. indicating "check"/"checkmate" or moving piece
/// disambiguation).
// TODO: Implement bijection for a move and a numeric index.
#[derive(Debug)]
pub enum Move {
    /// Regular moves are "normal" (not originating from en-passant or castling
    /// rules) chess moves.
    Regular {
        /// The square a piece is moving from.
        from: Square,
        /// The square the piece will occupy after the move is made.
        to: Square,
        /// A pawn can be promoted into [`PieceKind::Queen`],
        /// [`PieceKind::Rook`], [`PieceKind::Bishop`] or [`PieceKind::Knight`].
        promotion: Option<Promotion>,
    },
    /// [En passant] is a capture of opponent's pawn "in passing" (when it
    /// advances two squares from its original position).
    ///
    /// [En passant]: https://en.wikipedia.org/wiki/En_passant
    EnPassant {
        /// The square a piece is moving from.
        from: Square,
        /// The square the piece will occupy after the move is made.
        to: Square,
    },
    /// The [Castling] move that will involve a king and a rook "jummping" over
    /// each other.
    ///
    /// > Castling may be done only if the king has never moved, the rook
    /// involved has never moved,   the squares between the king and the
    /// rook involved are unoccupied, the king is not in   check, and the
    /// king does not cross over or end on a square attacked by an enemy piece.
    ///
    /// [Castling]: https://en.wikipedia.org/wiki/Castling
    Castle {
        /// The king can castle to one of the rooks: either a kingside rook
        /// ("short castle" or "O-O") or queenside rook ("long castle" or
        /// "O-O-O").
        side: CastlingSide,
    },
}

impl ToString for Move {
    /// Serializes a move in [UCI format].
    ///
    /// [UCI format]: http://wbec-ridderkerk.nl/html/UCIProtocol.html
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
            // > Castling is one of the rules of chess and is technically a king move (Hooper &
            // Whyld 1992:71).
            //
            // TODO: Castling target square depends on the rook position. In other chess variants
            // (most notably, Fischer Random Chess or FRC) this will be different.
            Move::Castle { side } => match *side {
                // TODO: This should actually be "e1g1", "e1c1", "e8g8", "e8c8" dependnig on the
                // side. Maybe `Move::to_string()` is the wrong API.
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
// TODO: Should this be `Position::generate_moves` instead?
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
        } - our_pieces.all();
        // Loop over the target squares and produce moves for those which are
        // not occupied by our pieces. The empty squares or opponent pieces
        // (captures) are valid.
        for to in targets.iter() {
            result.push(Move::Regular {
                from,
                to,
                promotion: None,
            })
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
mod test {
    use pretty_assertions::assert_eq;
}
