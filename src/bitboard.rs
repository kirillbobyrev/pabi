//! Bitboard representation for [Board]. Bitboard utilizes the fact that modern
//! processors operate on 64 bit integers, and the bit operations can be
//! performed simultaneously. This results in very efficient calculation of
//! possible attack vectors and other meaningful features that are calculated to
//! evaluate a position on the board. The disadvantage is complexity that comes
//! with bitboard implementation and inefficiency of some operations like "get
//! piece type on given square" (efficiently handled by Square-centric board
//! implementations).
//!
//! [Bitboard]: https://www.chessprogramming.org/Bitboards

use std::fmt;
use std::ops::{BitAnd, BitOr, BitOrAssign, BitXor};

use itertools::Itertools;

use crate::board::{Square, BOARD_SIZE, BOARD_WIDTH};

#[derive(Copy, Clone, Default)]
/// Represents a set of squares and provides common operations (e.g. AND, OR,
/// XOR) over these sets. Each bit corresponds to one of 64 squares of the chess
/// board.
///
/// Mirroring [Square] semantics, the least significant bit corresponds to A1,
/// and the most significant bit - to H8. [BitboardSet] is the primary user of
/// the bitboard.
///
/// ```
/// use pabi::board::Square;
/// use pabi::bitboard::Bitboard;
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
/// use pabi::bitboard::Bitboard;
///
/// assert_eq!(std::mem::size_of::<Bitboard>(), 8);
/// ```
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

    pub(crate) fn with_squares(squares: &[Square]) -> Self {
        let mut result = Default::default();
        for square in squares {
            result |= Bitboard::from(square.clone());
        }
        result
    }

    pub(crate) fn is_set(&self, square: Square) -> bool {
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

#[derive(Copy, Clone, Default)]
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
pub struct BitboardSet {
    king: Bitboard,
    queen: Bitboard,
    rooks: Bitboard,
    bishops: Bitboard,
    knights: Bitboard,
    pawns: Bitboard,
}

impl BitboardSet {
    pub(crate) fn new_white() -> Self {
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

    pub(crate) fn new_black() -> Self {
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

    pub(crate) fn all(&self) -> Bitboard {
        self.king | self.queen | self.rooks | self.bishops | self.knights | self.pawns
    }
}

#[cfg(test)]
mod test {
    use super::{Bitboard, BitboardSet};

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
