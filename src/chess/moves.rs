//! Move generation, application and parsing.
use strum::IntoEnumIterator;

use crate::chess::core::{Castling, PieceKind, Square, Piece};
use crate::chess::position::Position;
use crate::chess::bitboard::Board;

pub enum Move {
    Regular(RegularMove),
    EnPassant(EnPassantMove),
    Castle(Castling),
}

// TODO: Figure out what to do about "unmaking moves". They are useful for Perft
// (otherwise the boards should be copied and put on stack, the recursion
// works). Other than that, "unmaking" moves is not useful for anything else and
// takes up more memory.
#[derive(Copy, Clone, Debug)]
pub struct RegularMove {
    from: Square,
    to: Square,
    promotion: Option<PieceKind>,
}

#[derive(Copy, Clone, Debug)]
pub struct EnPassantMove {
    from: Square,
    to: Square,
}

pub fn generate_moves(position: &Position) -> Vec<Move> {
    let result = vec![];
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
            King => todo!(),
            Queen => todo!(),
            Rook => todo!(),
            Bishop => todo!(),
            Knight => todo!(),
            Pawn => todo!(),
        }
    }
    result
}

fn generate_sliding_moves(result: &mut Vec<Move>, board: &Board, piece: &Piece) {

}
