//! [Bitboard]-based piece-centric implementation of the chess board. This is
//! the "back-end" of the chess engine, efficient board representation is
//! crucial for performance.
//!
//! Bitboard representation utilizes the fact that modern processors operate on
//! 64 bit integers, and the bit operations can be performed simultaneously.
//! This results in very efficient calculation of possible attack vectors and
//! other meaningful features that are calculated to evaluate a position on the
//! board. The disadvantage is complexity that comes with bitboard
//! implementation and inefficiency of some operations like "get piece type on
//! given square" (efficiently handled by Square-centric board implementations).
//!
//! [Bitboard]: https://www.chessprogramming.org/Bitboards

use std::ops::{BitAnd, BitOr, BitOrAssign, BitXor};
use std::{fmt, mem};

use itertools::Itertools;

const BOARD_WIDTH: u8 = 8;
const BOARD_SIZE: u8 = BOARD_WIDTH * BOARD_WIDTH;

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
        assert!(file < 8);
        unsafe { mem::transmute(file) }
    }
}

/// Represents a horizontal row of the chessboard. In chess notation, it is
/// represented with a number. The implementation assumes zero-based values
/// (i.e. rank 1 would be 0).
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[allow(missing_docs)]
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
        assert!(rank < 8);
        unsafe { mem::transmute(rank) }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
#[rustfmt::skip]
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
#[allow(missing_docs)]
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
        Self::from(file as u8 + (rank as u8) * 7)
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
    fn from(position: u8) -> Self {
        debug_assert!(position < 64);
        unsafe { mem::transmute(position) }
    }
}

/// Represents a set of squares and provides common operations (e.g. AND, OR,
/// XOR) over these sets. Each bit corresponds to one of 64 squares of the chess
/// board.
///
/// Mirroring [Square] semantics, the least significant bit corresponds to A1,
/// and the most significant bit - to H8. See [BitboardSet] for more
/// information.
///
/// ```
/// use pabi::board::{Bitboard, Square};
///
/// assert_eq!(Bitboard::from(Square::A1).data(), 1);
/// assert_eq!(Bitboard::from(Square::B1).data(), 2);
/// assert_eq!(Bitboard::from(Square::D1).data(), 8);
/// assert_eq!(Bitboard::from(Square::H8).data(), 1u64 << 63);
/// ```
///
/// Bitboard is a wrapper around [u64] and takes only 8 bytes. Defaults to empty
/// set.
///
/// ```
/// use std::mem;
///
/// use pabi::board::Bitboard;
///
/// assert_eq!(std::mem::size_of::<Bitboard>(), 8);
/// ```
#[derive(Copy, Clone, Default)]
pub struct Bitboard {
    data: u64,
}

impl Bitboard {
    // TODO: Conceal this and only provide debug strings for doctest.
    pub fn data(&self) -> u64 {
        self.data
    }

    pub fn full() -> Self {
        Self { data: u64::MAX }
    }

    fn with_squares(squares: &[Square]) -> Self {
        let mut result = Default::default();
        for square in squares {
            result |= Bitboard::from(square.clone());
        }
        result
    }

    fn is_set(&self, square: Square) -> bool {
        (self.data & (1u64 << square as u8)) > 0
    }
}

impl BitOr for Bitboard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            data: self.data.bitor(rhs.data),
        }
    }
}

impl BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, rhs: Self) {
        self.data.bitor_assign(rhs.data);
    }
}

impl BitAnd for Bitboard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self {
            data: self.data.bitand(rhs.data),
        }
    }
}

impl BitXor for Bitboard {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self {
            data: self.data.bitxor(rhs.data),
        }
    }
}

impl From<Square> for Bitboard {
    fn from(square: Square) -> Self {
        (1u64 << square as u8).into()
    }
}

impl From<u64> for Bitboard {
    fn from(data: u64) -> Self {
        Bitboard { data }
    }
}

impl fmt::Debug for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: This is quite verbose. Refactor or explain what is happening.
        write!(
            f,
            "{}",
            format!("{:#066b}", self.data)
                .chars()
                .rev()
                .take(BOARD_SIZE as usize)
                .chunks(BOARD_WIDTH as usize)
                .into_iter()
                .map(|rank| rank.collect::<String>())
                .collect::<Vec<String>>()
                .iter()
                .rev()
                .join("\n")
        )
    }
}

/// Piece-centric representation of all material owned by one player. Uses
/// [Bitboard] to store a set of squares occupied by each piece.
///
/// Defaults to empty board.
// TODO: Caching all() and either replacing it or adding to the set might
// improve performance. This is what lc0 does:
// https://github.com/LeelaChessZero/lc0/blob/d2e372e59cd9188315d5c02a20e0bdce88033bc5/src/chess/board.h
// Note: There are other formats, e.g. array-based. It might be nice to test
// them out but I doubt it will be faster (Rust arrays have bounds checking) or
// more convenient (Rust has pattern matching).
#[derive(Copy, Clone, Default)]
pub struct BitboardSet {
    king: Bitboard,
    queen: Bitboard,
    rooks: Bitboard,
    bishops: Bitboard,
    knights: Bitboard,
    pawns: Bitboard,
}

impl BitboardSet {
    fn new_white() -> Self {
        Self {
            king: Square::E1.into(),
            queen: Square::D1.into(),
            rooks: Bitboard::with_squares(&[Square::A1, Square::H1]),
            bishops: Bitboard::with_squares(&[Square::C1, Square::F1]),
            knights: Bitboard::with_squares(&[Square::B1, Square::G1]),
            pawns: Bitboard::with_squares(&[
                Square::A2,
                Square::B2,
                Square::C2,
                Square::D2,
                Square::E2,
                Square::F2,
                Square::G2,
                Square::H2,
            ]),
        }
    }

    fn new_black() -> Self {
        // TODO: Implement flip and return new_white().flip() to prevent copying code.
        Self {
            king: Square::E8.into(),
            queen: Square::D8.into(),
            rooks: Bitboard::with_squares(&[Square::A8, Square::H8]),
            bishops: Bitboard::with_squares(&[Square::C8, Square::F8]),
            knights: Bitboard::with_squares(&[Square::B8, Square::G8]),
            pawns: Bitboard::with_squares(&[
                Square::A7,
                Square::B7,
                Square::C7,
                Square::D7,
                Square::E7,
                Square::F7,
                Square::G7,
                Square::H7,
            ]),
        }
    }

    fn all(&self) -> Bitboard {
        self.king | self.queen | self.rooks | self.bishops | self.knights | self.pawns
    }
}

/// Track the ability to [castle] each side (kingside is often referred to as
/// O-O or OO, queenside -- O-O-O or OOO). When the king moves, player loses
/// ability to castle both sides, when the rook moves, player loses ability to
/// castle its corresponding side.
///
/// [castle]: https://www.chessprogramming.org/Castling
pub enum CastlingRights {
    None,
    OnlyKingside,
    OnlyQueenside,
    Both,
}

/// State of the chess game: position, half-move counters and castling rights.
/// It can be (de)serialized from/into [Forsyth-Edwards Notation] (FEN).
///
/// [Forsyth-Edwards Notation]: https://www.chessprogramming.org/Forsyth-Edwards_Notation
// Note: This stores information about pieces in BitboardSets. Stockfish and
// many other engines maintain both piece- and square-centric representations at
// once.
// TODO: Check if this yields any benefits.
pub struct Board {
    white_pieces: BitboardSet,
    black_pieces: BitboardSet,
    white_castling: CastlingRights,
    black_castling: CastlingRights,
    /// [Halfmove[^ply] Clock][1] keeps track of the number of (half-)moves
    /// since the last capture or pawn move and is used to enforce
    /// fifty[^fifty]-move draw rule.
    ///
    ///
    /// [1]: https://www.chessprogramming.org/Halfmove_Clock
    /// [2]: https://www.chessprogramming.org/Ply
    /// [^ply]: "Half-move" or "ply" means a move of only one side.
    /// [^fifty]: 50 _full_ moves
    halfmove_clock: u8,
}

impl Board {
    pub fn new() -> Self {
        Self {
            white_pieces: BitboardSet::new_white(),
            black_pieces: BitboardSet::new_black(),
            white_castling: CastlingRights::Both,
            black_castling: CastlingRights::Both,
            halfmove_clock: 0,
        }
    }

    pub fn parse_fen(fen: &str) -> Result<Self, ()> {
        todo!();
    }

    pub fn fen() -> String {
        todo!();
    }

    fn material_imbalance() {
        todo!();
    }

    // IMPORTANT: This is slow because of the bitboard representation and
    // shouldn't be used in performance-critical scenarios.
    fn at(&self, square: Square) -> Option<Piece> {
        if self.white_pieces.all().is_set(square) {
            let owner = Player::White;
            let mut kind = PieceKind::Pawn;
            if self.white_pieces.king.is_set(square) {
                kind = PieceKind::King;
            }
            if self.white_pieces.queen.is_set(square) {
                kind = PieceKind::Queen;
            }
            if self.white_pieces.rooks.is_set(square) {
                kind = PieceKind::Rook;
            }
            if self.white_pieces.bishops.is_set(square) {
                kind = PieceKind::Bishop;
            }
            if self.white_pieces.knights.is_set(square) {
                kind = PieceKind::Knight;
            }
            return Some(Piece { owner, kind });
        }
        if self.black_pieces.all().is_set(square) {
            let owner = Player::Black;
            let mut kind = PieceKind::Pawn;
            if self.white_pieces.king.is_set(square) {
                kind = PieceKind::King;
            }
            if self.white_pieces.queen.is_set(square) {
                kind = PieceKind::Queen;
            }
            if self.white_pieces.rooks.is_set(square) {
                kind = PieceKind::Rook;
            }
            if self.white_pieces.bishops.is_set(square) {
                kind = PieceKind::Bishop;
            }
            if self.white_pieces.knights.is_set(square) {
                kind = PieceKind::Knight;
            }
            return Some(Piece { owner, kind });
        }
        None
    }
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

/// A standard game of chess is played between two players: White (having the
/// advantage of the first turn) and Black.
#[allow(missing_docs)]
pub enum Player {
    White,
    Black,
}

/// Standard [chess pieces].
///
/// [chess pieces]: https://en.wikipedia.org/wiki/Chess_piece
#[allow(missing_docs)]
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
    use super::{Bitboard, BitboardSet, File, Rank, BOARD_WIDTH};

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
    #[should_panic(expected = "assertion failed: rank < 8")]
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
    #[should_panic(expected = "assertion failed: file < 8")]
    fn invalid_file() {
        let _ = File::from(BOARD_WIDTH);
    }

    #[test]
    fn bitboard_basics() {
        // Create a starting position.
        let white = BitboardSet::new_white();
        let black = BitboardSet::new_black();

        // Check that each player has 16 pieces.
        assert_eq!(white.all().data.count_ones(), 16);
        assert_eq!(black.all().data.count_ones(), 16);
        // Check that each player has correct number of pieces (previous check
        // was not enough to confirm there are no overlaps).
        assert_eq!(white.king.data.count_ones(), 1);
        assert_eq!(black.king.data.count_ones(), 1);
        assert_eq!(white.queen.data.count_ones(), 1);
        assert_eq!(black.queen.data.count_ones(), 1);
        assert_eq!(white.rooks.data.count_ones(), 2);
        assert_eq!(black.rooks.data.count_ones(), 2);
        assert_eq!(white.bishops.data.count_ones(), 2);
        assert_eq!(black.bishops.data.count_ones(), 2);
        assert_eq!(white.knights.data.count_ones(), 2);
        assert_eq!(black.knights.data.count_ones(), 2);
        assert_eq!(white.pawns.data.count_ones(), 8);
        assert_eq!(black.pawns.data.count_ones(), 8);

        // Check few positions manually.
        assert_eq!(white.queen.data, 1 << 3);
        assert_eq!(black.queen.data, 1 << (3 + 8 * 7));
    }

    #[test]
    // Check the debug output for few bitboards.
    fn bitboard_dump() {
        #[rustfmt::skip]
        assert_eq!(
            format!("{:?}", Bitboard::default()),
            "00000000\n\
             00000000\n\
             00000000\n\
             00000000\n\
             00000000\n\
             00000000\n\
             00000000\n\
             00000000"
        );
        #[rustfmt::skip]
        assert_eq!(
            format!("{:?}", Bitboard::full()),
            "11111111\n\
             11111111\n\
             11111111\n\
             11111111\n\
             11111111\n\
             11111111\n\
             11111111\n\
             11111111"
        );

        let white = BitboardSet::new_white();
        let black = BitboardSet::new_black();

        #[rustfmt::skip]
        assert_eq!(
            format!("{:?}", black.all()),
            "11111111\n\
             11111111\n\
             00000000\n\
             00000000\n\
             00000000\n\
             00000000\n\
             00000000\n\
             00000000"
        );
        #[rustfmt::skip]
        assert_eq!(
            format!("{:?}", white.all() | black.all()),
            "11111111\n\
             11111111\n\
             00000000\n\
             00000000\n\
             00000000\n\
             00000000\n\
             11111111\n\
             11111111"
        );

        #[rustfmt::skip]
        assert_eq!(
            format!("{:?}", white.king),
            "00000000\n\
             00000000\n\
             00000000\n\
             00000000\n\
             00000000\n\
             00000000\n\
             00000000\n\
             00001000"
        );
        #[rustfmt::skip]
        assert_eq!(
            format!("{:?}", black.pawns),
            "00000000\n\
             11111111\n\
             00000000\n\
             00000000\n\
             00000000\n\
             00000000\n\
             00000000\n\
             00000000"
        );
        #[rustfmt::skip]
        assert_eq!(
            format!("{:?}", black.knights),
            "01000010\n\
             00000000\n\
             00000000\n\
             00000000\n\
             00000000\n\
             00000000\n\
             00000000\n\
             00000000"
        );
    }
}
