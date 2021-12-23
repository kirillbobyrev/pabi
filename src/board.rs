/// [Bitboard][1]-based implementation of the chess board.
/// [1]: https://www.chessprogramming.org/Bitboards
use itertools::Itertools;
use std::fmt;
use std::mem;
use std::ops::{BitAnd, BitOr, BitXor};

const BOARD_WIDTH: u8 = 8;
const BOARD_SIZE: u8 = BOARD_WIDTH * BOARD_WIDTH;

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
/// [`Square`] is a compact representation using only one byte.
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

impl From<u8> for Square {
    /// Creates a square given its position on the board (0 through 63).
    fn from(position: u8) -> Self {
        debug_assert!(position <= 63);
        // This can also be done via num::FromPrimitive
        // (https://enodev.fr/posts/rusticity-convert-an-integer-to-an-enum.html).
        // TODO: Test this out and benchmark to see if the performance would not
        // be harmed?
        unsafe { mem::transmute(position) }
    }
}

/// Each bit represents one of 64 squares of the chess board. Mirroring [Square]
/// semantics, the least significant bit corresponds to A1, and the most
/// significant bit - to H8. Therefore, each [Square] can be converted into
/// [Bitboard] with a single bit being set at square's position. However,
/// [Bitboard] does not always correspond to a single square: see [BitboardSet].
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
/// [Bitboard] is a wrapper around [u64] and takes 8 only bytes.
///
/// ```
/// use pabi::board::Bitboard;
/// use std::mem;
///
/// assert_eq!(std::mem::size_of::<Bitboard>(), 8);
/// ```

#[derive(Copy, Clone)]
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

    pub fn empty() -> Self {
        Self { data: 0 }
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

/// Piece-centric representation of all material owned by a player.
// TODO: Caching all() and either replacing it or adding to the set might
// improve performance. This needs a benchmark.
// NOTE: There are other formats, e.g. array-based. It might be nice to test
// them out but I doubt it will be faster (Rust arrays have bounds checking) or
// more convenient (Rust has pattern matching).
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
            // TODO: Implement Bitboard::FromOr(&[Square]).
            rooks: Bitboard::from(Square::A1) | Bitboard::from(Square::H1),
            bishops: Bitboard::from(Square::C1) | Bitboard::from(Square::F1),
            knights: Bitboard::from(Square::B1) | Bitboard::from(Square::G1),
            pawns: Bitboard::from(Square::A2)
                | Bitboard::from(Square::B2)
                | Bitboard::from(Square::C2)
                | Bitboard::from(Square::D2)
                | Bitboard::from(Square::E2)
                | Bitboard::from(Square::F2)
                | Bitboard::from(Square::G2)
                | Bitboard::from(Square::H2),
        }
    }

    fn new_black() -> Self {
        // TODO: Implement flip and return new_white().flip() to prevent copying code.
        Self {
            king: Square::E8.into(),
            queen: Square::D8.into(),
            rooks: Bitboard::from(Square::A8) | Bitboard::from(Square::H8),
            bishops: Bitboard::from(Square::C1) | Bitboard::from(Square::F8),
            knights: Bitboard::from(Square::B8) | Bitboard::from(Square::G8),
            pawns: Bitboard::from(Square::A7)
                | Bitboard::from(Square::B7)
                | Bitboard::from(Square::C7)
                | Bitboard::from(Square::D7)
                | Bitboard::from(Square::E7)
                | Bitboard::from(Square::F7)
                | Bitboard::from(Square::G7)
                | Bitboard::from(Square::H7),
        }
    }

    fn all(&self) -> Bitboard {
        self.king | self.queen | self.rooks | self.bishops | self.knights | self.pawns
    }
}

/// Position on the board without the state (castling, 50-move draw rule, etc).
struct Position {
    white_pieces: BitboardSet,
    black_pieces: BitboardSet,
}

impl Position {
    fn new() -> Self {
        Self {
            white_pieces: BitboardSet::new_white(),
            black_pieces: BitboardSet::new_black(),
        }
    }
}

enum Player {
    White,
    Black,
}

enum PieceKind {
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

struct Piece {
    owner: Player,
    kind: PieceKind,
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: Switch to emoji/algebraic symbols? Figurine is hard to read in
        // the terminal.
        // TODO: Alternatively, render an image for the whole position.
        let figurine_symbol = match &self.owner {
            Player::White => match &self.kind {
                PieceKind::King => "♔",
                PieceKind::Queen => "♕",
                PieceKind::Rook => "♖",
                PieceKind::Bishop => "♗",
                PieceKind::Knight => "♞",
                PieceKind::Pawn => "♙",
            },
            Player::Black => match &self.kind {
                PieceKind::King => "♚",
                PieceKind::Queen => "♛",
                PieceKind::Rook => "♜",
                PieceKind::Bishop => "♝",
                PieceKind::Knight => "K",
                PieceKind::Pawn => "♟︎",
            },
        };
        write!(f, "{}", figurine_symbol)
    }
}

enum CastlingAvailability {
    None,
    OnlyKingside,
    OnlyQueenside,
    Both,
}

#[cfg(test)]
mod test {
    use super::BitboardSet;

    #[test]
    fn bitboard() {
        let white = BitboardSet::new_white();
        let black = BitboardSet::new_black();

        assert_eq!(white.all().data.count_ones(), 16);
        assert_eq!(black.all().data.count_ones(), 16);

        assert_eq!(
            format!("{:?}", white.king),
            "00000000
00000000
00000000
00000000
00000000
00000000
00000000
00001000"
        )
    }
}
