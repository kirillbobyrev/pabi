//! Piece-centric implementation of the chess board. This is
//! the "back-end" of the chess engine, efficient board representation is
//! crucial for performance.

use std::{fmt, mem};

use itertools::Itertools;

use crate::bitboard::BitboardSet;

pub const BOARD_WIDTH: u8 = 8;
pub const BOARD_SIZE: u8 = BOARD_WIDTH * BOARD_WIDTH;

/// Represents a column (vertical row) of the chessboard. In chess notation, it
/// is normally represented with a lowercase letter.
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[allow(missing_docs)]
pub enum File {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
}

impl From<u8> for File {
    /// Input has to be a number in [0; 7] range.
    fn from(file: u8) -> Self {
        assert!(file < BOARD_WIDTH);
        unsafe { mem::transmute(file) }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[allow(missing_docs)]
/// Represents a horizontal row of the chessboard. In chess notation, it is
/// represented with a number. The implementation assumes zero-based values
/// (i.e. rank 1 would be 0).
pub enum Rank {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
}

impl From<u8> for Rank {
    /// Input has to be a number in [0; 7] range.
    fn from(rank: u8) -> Self {
        assert!(rank < BOARD_WIDTH);
        unsafe { mem::transmute(rank) }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[rustfmt::skip]
#[allow(missing_docs)]
/// Board squares: from left to right, from bottom to the top:
///
/// ```
/// use pabi::board::Square;
///
/// assert_eq!(Square::A1 as u8, 0);
/// assert_eq!(Square::E1 as u8, 4);
/// assert_eq!(Square::H1 as u8, 7);
/// assert_eq!(Square::A4 as u8, 8 * 3);
/// assert_eq!(Square::H8 as u8, 63);
/// ```
///
/// Square is a compact representation using only one byte.
///
/// ```
/// use pabi::board::Square;
/// use std::mem;
///
/// assert_eq!(std::mem::size_of::<Square>(), 1);
/// ```
pub enum Square {
    A1, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8,
}

impl Square {
    pub fn new(file: File, rank: Rank) -> Self {
        Self::from(file as u8 + (rank as u8) * BOARD_WIDTH)
    }

    fn file(&self) -> File {
        File::from(*self as u8 % 8)
    }

    fn rank(&self) -> Rank {
        Rank::from(*self as u8 / 8)
    }
}

impl From<u8> for Square {
    /// Creates a square given its position on the board (input has to be within
    /// [0; 63] range).
    fn from(square: u8) -> Self {
        assert!(square < BOARD_SIZE);
        unsafe { mem::transmute(square) }
    }
}

/// Track the ability to [castle] each side (kingside is often referred to as
/// O-O or OO, queenside -- O-O-O or OOO). When the king moves, player loses
/// ability to castle both sides, when the rook moves, player loses ability to
/// castle its corresponding side.
///
/// [castle]: https://www.chessprogramming.org/Castling
pub enum CastlingAbility {
    Neither,
    OnlyKingside,
    OnlyQueenside,
    Both,
}

/// State of the chess game: position, half-move counters and castling rights,
/// etc. It can be (de)serialized from/into [Forsyth-Edwards Notation] (FEN).
///
/// [Forsyth-Edwards Notation]: https://www.chessprogramming.org/Forsyth-Edwards_Notation
// Note: This stores information about pieces in BitboardSets. Stockfish and
// many other engines maintain both piece- and square-centric representations at
// once.
// TODO: Check if this yields any benefits.
pub struct Board {
    white_pieces: BitboardSet,
    black_pieces: BitboardSet,
    white_castling: CastlingAbility,
    black_castling: CastlingAbility,
    /// [Halfmove Clock][^ply] keeps track of the number of (half-)moves
    /// since the last capture or pawn move and is used to enforce
    /// fifty[^fifty]-move draw rule.
    ///
    ///
    /// [Halfmove Clock]: https://www.chessprogramming.org/Halfmove_Clock
    /// [^ply]: "Half-move" or ["ply"](https://www.chessprogramming.org/Ply) means a move of only
    ///     one side.
    /// [^fifty]: 50 _full_ moves
    halfmove_counter: u8,
    fullmove_counter: u8,
    en_passant_square: Option<Square>,
}

pub enum FENParsingError {
    WrongFormat,
    WrongPosition,
    WrongCastling,
    WrongEnPassant,
    WrongHalfmoveClock,
    WrongFullmoveCounter,
}

impl Board {
    /// Creates a board with the starting position.
    pub fn new() -> Self {
        Self {
            white_pieces: BitboardSet::new_white(),
            black_pieces: BitboardSet::new_black(),
            white_castling: CastlingAbility::Both,
            black_castling: CastlingAbility::Both,
            halfmove_counter: 0,
            fullmove_counter: 0,
            en_passant_square: None,
        }
    }

    /// Parses board from Forsyth-Edwards Notation.
    ///
    /// <FEN> ::=  <Piece Placement>
    ///   ' ' <Side to move>
    ///   ' ' <Castling ability>
    ///   ' ' <En passant target square>
    ///   ' ' <Halfmove clock>
    ///   ' ' <Fullmove counter>
    ///
    /// <Piece Placement> ::=
    /// <rank8>'/'<rank7>'/'<rank6>'/'<rank5>'/'<rank4>'/'<rank3>'/'<rank2>'/'
    /// <rank1> <ranki>       ::= [<digit17>]<piece> {[<digit17>]<piece>}
    /// [<digit17>] | '8' <piece>       ::= <white Piece> | <black Piece>
    /// <digit17>     ::= '1' | '2' | '3' | '4' | '5' | '6' | '7'
    /// <white Piece> ::= 'P' | 'N' | 'B' | 'R' | 'Q' | 'K'
    /// <black Piece> ::= 'p' | 'n' | 'b' | 'r' | 'q' | 'k'
    pub fn from_fen(fen: &str) -> Result<Self, FENParsingError> {
        // FEN should be a one-line ASCII string.
        assert!(fen.is_ascii() && fen.lines().count() == 1);
        let (
            pieces_placement,
            side_to_move,
            casting_ability,
            en_passant_square,
            halfmove_clock,
            fullmove_counter,
        ) = fen.split(' ').collect_tuple().unwrap();
        // Parse Piece Placement.
        if pieces_placement.matches('/').count() != 8 {
            return Err(FENParsingError::WrongPosition);
        }
        let result = Self::new();
        let ranks = pieces_placement.split('/');
        for (rank_id, rank_fen) in ranks.enumerate() {
            dbg!(rank_id, rank_fen);
        }
        Ok(result)
    }

    /// Dumps board in Forsyth-Edwards Notation.
    pub fn fen() -> String {
        todo!();
    }

    fn material_imbalance() {
        todo!();
    }

    // // IMPORTANT: This is slow because of the bitboard representation and
    // // shouldn't be used in performance-critical scenarios.
    // fn at(&self, square: Square) -> Option<Piece> {
    //     if self.white_pieces.all().is_set(square) {
    //         let owner = Player::White;
    //         let mut kind = PieceKind::Pawn;
    //         if self.white_pieces.king.is_set(square) {
    //             kind = PieceKind::King;
    //         }
    //         if self.white_pieces.queen.is_set(square) {
    //             kind = PieceKind::Queen;
    //         }
    //         if self.white_pieces.rooks.is_set(square) {
    //             kind = PieceKind::Rook;
    //         }
    //         if self.white_pieces.bishops.is_set(square) {
    //             kind = PieceKind::Bishop;
    //         }
    //         if self.white_pieces.knights.is_set(square) {
    //             kind = PieceKind::Knight;
    //         }
    //         return Some(Piece { owner, kind });
    //     }
    //     if self.black_pieces.all().is_set(square) {
    //         let owner = Player::Black;
    //         let mut kind = PieceKind::Pawn;
    //         if self.white_pieces.king.is_set(square) {
    //             kind = PieceKind::King;
    //         }
    //         if self.white_pieces.queen.is_set(square) {
    //             kind = PieceKind::Queen;
    //         }
    //         if self.white_pieces.rooks.is_set(square) {
    //             kind = PieceKind::Rook;
    //         }
    //         if self.white_pieces.bishops.is_set(square) {
    //             kind = PieceKind::Bishop;
    //         }
    //         if self.white_pieces.knights.is_set(square) {
    //             kind = PieceKind::Knight;
    //         }
    //         return Some(Piece { owner, kind });
    //     }
    //     None
    // }
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        todo!();
        for square in Square::A1 as u8..Square::H8 as u8 {
            let square: Square = square.into();
        }
        Ok(())
    }
}

#[allow(missing_docs)]
/// A standard game of chess is played between two players: White (having the
/// advantage of the first turn) and Black.
pub enum Player {
    White,
    Black,
}

#[allow(missing_docs)]
/// Standard [chess pieces].
///
/// [chess pieces]: https://en.wikipedia.org/wiki/Chess_piece
pub enum PieceKind {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

impl PieceKind {
    fn relative_value(&self) -> Option<u32> {
        match &self {
            // The value of King is undefined as it cannot be captured.
            PieceKind::King => None,
            PieceKind::Queen => Some(9),
            PieceKind::Rook => Some(6),
            PieceKind::Bishop => Some(3),
            PieceKind::Knight => Some(3),
            PieceKind::Pawn => Some(1),
        }
    }
}

/// Represents a specific piece owned by a player.
pub struct Piece {
    owner: Player,
    kind: PieceKind,
}

impl Piece {
    // Algebraic notation symbol used in FEN. Uppercase for white, lowercase for
    // black.
    fn algebraic_symbol(&self) -> char {
        let result = match &self.kind {
            PieceKind::King => 'k',
            PieceKind::Queen => 'q',
            PieceKind::Rook => 'r',
            PieceKind::Bishop => 'b',
            PieceKind::Knight => 'k',
            PieceKind::Pawn => 'p',
        };
        match &self.owner {
            Player::White => result.to_ascii_uppercase(),
            Player::Black => result,
        }
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.algebraic_symbol())
    }
}

#[cfg(test)]
mod test {
    use super::{File, Rank, Square, BOARD_SIZE, BOARD_WIDTH};

    #[test]
    fn rank() {
        let ranks: Vec<_> = (0..BOARD_WIDTH).map(|rank| Rank::from(rank)).collect();
        assert_eq!(
            ranks,
            vec![
                Rank::One,
                Rank::Two,
                Rank::Three,
                Rank::Four,
                Rank::Five,
                Rank::Six,
                Rank::Seven,
                Rank::Eight,
            ]
        );
    }

    #[test]
    #[should_panic(expected = "assertion failed: rank < BOARD_WIDTH")]
    fn invalid_rank() {
        let _ = Rank::from(BOARD_WIDTH);
    }

    #[test]
    fn file() {
        let files: Vec<_> = (0..BOARD_WIDTH).map(|file| File::from(file)).collect();
        assert_eq!(
            files,
            vec![
                File::A,
                File::B,
                File::C,
                File::D,
                File::E,
                File::F,
                File::G,
                File::H,
            ]
        );
    }

    #[test]
    #[should_panic(expected = "assertion failed: file < BOARD_WIDTH")]
    fn invalid_file() {
        let _ = File::from(BOARD_WIDTH);
    }

    #[test]
    fn square() {
        let squares: Vec<_> = [
            0u8,
            BOARD_SIZE - 1,
            BOARD_WIDTH - 1,
            BOARD_WIDTH,
            BOARD_WIDTH * 2 + 5,
        ]
        .iter()
        .map(|square| Square::from(*square))
        .collect();
        assert_eq!(
            squares,
            vec![Square::A1, Square::H8, Square::H1, Square::A2, Square::F3]
        );
        let squares: Vec<_> = [(1u8, 2u8), (5, 4), (7, 7), (4, 3)]
            .iter()
            .map(|(file, rank)| Square::new(File::from(*file), Rank::from(*rank)))
            .collect();
        assert_eq!(
            squares,
            vec![Square::B3, Square::F5, Square::H8, Square::E4]
        );
    }

    #[test]
    #[should_panic(expected = "assertion failed: square < BOARD_SIZE")]
    fn invalid_square() {
        let _ = Square::from(BOARD_SIZE);
    }
}
