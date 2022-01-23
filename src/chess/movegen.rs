//! Move handling: [generation] (using [BMI PEXT Bitboards]), application and
//! (de)serialization.
//!
//! NOTE: [BMI Instruction Set] (and specifically efficient [PEXT]) is not
//! widely available on all processors (e.g. the AMD only started providing an
//! *efficient* PEXT since Ryzen 3). The current implementation will rely on
//! PEXT because it is the most efficient move generator technique available.
//!
//! [generation]: https://www.chessprogramming.org/Table-driven_Move_Generation
//! [BMI2 Pext Bitboards]: https://www.chessprogramming.org/BMI2#PEXTBitboards
//! [BMI Instruction Set]: https://en.wikipedia.org/wiki/X86_Bit_manipulation_instruction_set
//! [PEXT]: https://en.wikipedia.org/wiki/X86_Bit_manipulation_instruction_set#Parallel_bit_deposit_and_extract

// TODO: Fall back to Fancy Magic Bitboards if BMI2 + PEXT are not available for
// portability? TODO: Look at and compare speed with https://github.com/jordanbray/chess
// TODO: Also implement divide and use <https://github.com/jniemann66/juddperft> to validate the
// results.
// TODO: Maybe use python-chess testset of perft moves:
// https://github.com/niklasf/python-chess/blob/master/examples/perft/random.perft
// TODO: Compare with other engines and perft generattors, e.g. Berserk,
// shakmaty (https://github.com/jordanbray/chess_perft).
// TODO: Check movegen comparison (https://github.com/Gigantua/Chess_Movegen).

use strum::IntoEnumIterator;

use crate::chess::core::{CastlingSide, PieceKind, Square};
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
    pub fn to_san(&self, position: &Position) -> String {
        todo!();
    }
}

#[derive(Clone, Copy, Debug)]
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

// TODO: This is quite slow - look at https://github.com/Gigantua/Chess_Movegen
// for more efficient ideas.
pub fn generate_moves(position: &Position) -> Vec<Move> {
    let mut result = vec![];
    // Cache squares occupied by each player.
    for square in Square::iter() {
        // Only generate moves when the square is non-empty and has the piece of a
        // correct color on it.
        let piece = match position.board.at(square) {
            None => continue,
            Some(piece) => piece,
        };
        if piece.owner != position.side_to_move {
            continue;
        }
        match piece.kind {
            PieceKind::King => todo!(),
            PieceKind::Queen | PieceKind::Rook | PieceKind::Bishop => todo!(),
            PieceKind::Knight => todo!(),
            PieceKind::Pawn => todo!(),
        }
    }
    // Castling.
    // TODO: Check for attacks on relevant squares.
    // TODO: Pawn moves + en passant.
    result
}

#[cfg(test)]
mod test {}
