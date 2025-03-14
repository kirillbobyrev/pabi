//! [`Bitboard`]-based representation for [`crate::chess::position::Position`].
//! [Bitboards] utilize the fact that modern processors operate on 64 bit
//! integers, and the bit operations can be performed simultaneously. This
//! results in very efficient calculation of possible attack vectors and other
//! meaningful features that are required to calculate possible moves and
//! evaluate position. The disadvantage is complexity that comes with bitboard
//! implementation and inefficiency of some operations like "get piece type on
//! given square" (efficiently handled by Square-centric board implementations
//! that can be used together bitboard-based approach to compensate its
//! shortcomings).
//!
//! The implementation is based on [PEXT Bitboards] idea, which is an
//! improvement over Fancy Magic Bitboards.
//!
//! For visualizing and debugging the bitboards, there is a [BitboardCalculator]
//! tool.
//!
//! [Bitboards]: https://www.chessprogramming.org/Bitboards
//! [BitboardCalculator]: https://gekomad.github.io/Cinnamon/BitboardCalculator/
//! [PEXT Bitboards]: https://www.chessprogramming.org/BMI2#PEXTBitboards

use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, Not, Shl, Shr, Sub, SubAssign};
use std::{fmt, mem};

use itertools::Itertools;

use crate::chess::core::{BOARD_SIZE, BOARD_WIDTH, Direction, PieceKind, Square};
use crate::environment::Player;

/// Represents a set of squares and provides common operations (e.g. AND, OR,
/// XOR) over these sets. Each bit corresponds to one of 64 squares of the chess
/// board.
///
/// Mirroring [`Square`] semantics, the least significant bit corresponds to
/// [`Square::A1`], and the most significant bit - to [`Square::H8`].
///
/// Bitboard is a thin wrapper around [u64]:
///
/// ```
/// use std::mem::size_of;
///
/// use pabi::chess::bitboard::Bitboard;
///
/// assert_eq!(size_of::<Bitboard>(), 8);
/// ```
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Bitboard {
    bits: u64,
}

impl Bitboard {
    /// Constructs Bitboard from pre-calculated bits.
    #[must_use]
    pub const fn from_bits(bits: u64) -> Self {
        Self { bits }
    }

    /// Constructs a bitboard representing empty set of squares.
    #[must_use]
    pub const fn empty() -> Self {
        Self::from_bits(0)
    }

    /// Constructs a bitboard representing the universal set, it contains all
    /// squares by setting all bits to binary one.
    #[must_use]
    pub const fn full() -> Self {
        Self::from_bits(u64::MAX)
    }

    /// Returns raw bits.
    #[must_use]
    pub const fn bits(self) -> u64 {
        self.bits
    }

    #[must_use]
    pub fn from_squares(squares: &[Square]) -> Self {
        let mut result = Self::empty();
        for square in squares {
            result |= Self::from(*square);
        }
        result
    }

    /// Adds given square to the set.
    pub(super) fn extend(&mut self, square: Square) {
        *self |= Self::from(square);
    }

    /// Clears given square from the set.
    pub(super) fn clear(&mut self, square: Square) {
        *self &= !Self::from(square);
    }

    /// Returns true if this bitboard contains given square.
    #[must_use]
    pub(super) const fn contains(self, square: Square) -> bool {
        (self.bits & (1u64 << square as u8)) != 0
    }

    #[must_use]
    pub(super) const fn as_square(self) -> Square {
        debug_assert!(self.bits.count_ones() == 1);
        unsafe { mem::transmute(self.bits.trailing_zeros() as u8) }
    }

    #[must_use]
    pub(crate) const fn count(self) -> u32 {
        self.bits.count_ones()
    }

    #[must_use]
    pub(super) const fn is_empty(self) -> bool {
        self.bits == 0
    }

    #[must_use]
    pub(super) const fn has_any(self) -> bool {
        self.bits != 0
    }

    #[must_use]
    pub(super) fn shift(self, direction: Direction) -> Self {
        match direction {
            Direction::Up => self << u32::from(BOARD_WIDTH),
            Direction::Down => self >> u32::from(BOARD_WIDTH),
        }
    }

    /// Flips the bitboard vertically.
    ///
    /// This is useful when we want to switch between the board point of view of
    /// White and Black.
    ///
    /// # Example
    ///
    /// ```
    /// use pabi::chess::bitboard::Bitboard;
    ///
    /// let bb = Bitboard::from_bits(0x1E2222120E0A1222);
    /// assert_eq!(
    ///     format!("{:?}", bb),
    ///     ". 1 1 1 1 . . .\n\
    ///      . 1 . . . 1 . .\n\
    ///      . 1 . . . 1 . .\n\
    ///      . 1 . . 1 . . .\n\
    ///      . 1 1 1 . . . .\n\
    ///      . 1 . 1 . . . .\n\
    ///      . 1 . . 1 . . .\n\
    ///      . 1 . . . 1 . ."
    /// );
    /// assert_eq!(
    ///     format!("{:?}", bb.flip_perspective()),
    ///     ". 1 . . . 1 . .\n\
    ///      . 1 . . 1 . . .\n\
    ///      . 1 . 1 . . . .\n\
    ///      . 1 1 1 . . . .\n\
    ///      . 1 . . 1 . . .\n\
    ///      . 1 . . . 1 . .\n\
    ///      . 1 . . . 1 . .\n\
    ///      . 1 1 1 1 . . ."
    /// );
    /// ```
    #[must_use]
    pub fn flip_perspective(&self) -> Self {
        Self::from_bits(self.bits.swap_bytes())
    }

    /// An efficient way to iterate over the set squares.
    #[must_use]
    pub(super) const fn iter(self) -> BitboardIterator {
        BitboardIterator { bits: self.bits }
    }
}

impl fmt::Debug for Bitboard {
    /// The board is printed from A1 to H8, starting from bottom left corner to
    /// the top right corner, just like on the normal chess board from the
    /// perspective of White.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const LINE_SEPARATOR: &str = "\n";
        const SQUARE_SEPARATOR: &str = " ";
        write!(
            f,
            "{}",
            format!("{:#066b}", self.bits)
                .chars()
                .rev()
                .take(BOARD_SIZE as usize)
                .chunks(BOARD_WIDTH as usize)
                .into_iter()
                .map(|chunk| chunk
                    .map(|ch| match ch {
                        '1' => '1',
                        '0' => '.',
                        _ => unreachable!(),
                    })
                    .join(SQUARE_SEPARATOR))
                .collect::<Vec<String>>()
                .iter()
                .rev()
                .join(LINE_SEPARATOR)
        )
    }
}

impl BitOr for Bitboard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self::from_bits(self.bits.bitor(rhs.bits))
    }
}

impl BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, rhs: Self) {
        self.bits.bitor_assign(rhs.bits);
    }
}

impl BitAnd for Bitboard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self::from_bits(self.bits.bitand(rhs.bits))
    }
}

impl BitAndAssign for Bitboard {
    fn bitand_assign(&mut self, rhs: Self) {
        self.bits.bitand_assign(rhs.bits);
    }
}

impl BitXor for Bitboard {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self::from_bits(self.bits.bitxor(rhs.bits))
    }
}

impl Sub for Bitboard {
    type Output = Self;

    /// [Relative component], i.e. Result = LHS \ RHS.
    ///
    /// [Relative component]: https://en.wikipedia.org/wiki/Complement_%28set_theory%29#Relative_complement
    fn sub(self, rhs: Self) -> Self::Output {
        self & !rhs
    }
}

impl SubAssign for Bitboard {
    fn sub_assign(&mut self, rhs: Self) {
        self.bitand_assign(!rhs);
    }
}

impl Not for Bitboard {
    type Output = Self;

    /// Returns [complement
    /// set](https://en.wikipedia.org/wiki/Complement_%28set_theory%29) of Self,
    /// i.e. flipping the set squares to unset and vice versa.
    fn not(self) -> Self::Output {
        Self::from_bits(!self.bits)
    }
}

impl Shl<u32> for Bitboard {
    type Output = Self;

    /// Shifts the bits to the left and ignores overflow.
    fn shl(self, rhs: u32) -> Self::Output {
        let (bits, _) = self.bits.overflowing_shl(rhs);
        Self::from_bits(bits)
    }
}

impl Shr<u32> for Bitboard {
    type Output = Self;

    /// Shifts the bits to the right and ignores overflow.
    fn shr(self, rhs: u32) -> Self::Output {
        let (bits, _) = self.bits.overflowing_shr(rhs);
        Self::from_bits(bits)
    }
}

impl From<Square> for Bitboard {
    fn from(square: Square) -> Self {
        Self::from_bits(1u64 << square as u8)
    }
}

/// Iterates over set squares in a given [Bitboard] from least significant 1
/// bits (LS1B) to most significant 1 bits (MS1B) through implementing
/// [bitscan] forward operation.
///
/// [bitscan]: https://www.chessprogramming.org/BitScan
pub(super) struct BitboardIterator {
    bits: u64,
}

impl Iterator for BitboardIterator {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bits == 0 {
            return None;
        }
        // Get the LS1B and consume it from the iterator.
        let next_index = self.bits.trailing_zeros();
        // Clear LS1B.
        self.bits &= self.bits - 1;
        // For performance reasons, it's better to convert directly: the
        // conversion is safe because trailing_zeros() will return a number in
        // the 0..64 range.
        Some(unsafe { mem::transmute(next_index as u8) })
    }
}

impl ExactSizeIterator for BitboardIterator {
    fn len(&self) -> usize {
        self.bits.count_ones() as usize
    }
}

impl TryInto<Square> for Bitboard {
    type Error = anyhow::Error;

    fn try_into(self) -> anyhow::Result<Square> {
        if self.bits.count_ones() != 1 {
            anyhow::bail!(
                "bitboard should contain exactly 1 bit, got {}",
                self.bits.count_ones()
            );
        }
        Ok(unsafe { mem::transmute(self.bits.trailing_zeros() as u8) })
    }
}

/// Piece-centric representation of all material owned by one player. Uses
/// [Bitboard] to store a set of squares occupied by each piece. The main user
/// is [`crate::chess::position::Position`], [Bitboard] is not very useful on
/// its own.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Pieces {
    pub(super) king: Bitboard,
    pub(super) queens: Bitboard,
    pub(super) rooks: Bitboard,
    pub(super) bishops: Bitboard,
    pub(super) knights: Bitboard,
    pub(super) pawns: Bitboard,
}

impl Pieces {
    pub(super) const fn empty() -> Self {
        Self {
            king: Bitboard::empty(),
            queens: Bitboard::empty(),
            rooks: Bitboard::empty(),
            bishops: Bitboard::empty(),
            knights: Bitboard::empty(),
            pawns: Bitboard::empty(),
        }
    }

    pub(super) fn starting(player: Player) -> Self {
        match player {
            Player::White => Self {
                king: Square::E1.into(),
                queens: Square::D1.into(),
                rooks: Bitboard::from_squares(&[Square::A1, Square::H1]),
                bishops: Bitboard::from_squares(&[Square::C1, Square::F1]),
                knights: Bitboard::from_squares(&[Square::B1, Square::G1]),
                pawns: Bitboard::from_squares(&[
                    Square::A2,
                    Square::B2,
                    Square::C2,
                    Square::D2,
                    Square::E2,
                    Square::F2,
                    Square::G2,
                    Square::H2,
                ]),
            },
            Player::Black => Self {
                king: Square::E8.into(),
                queens: Square::D8.into(),
                rooks: Bitboard::from_squares(&[Square::A8, Square::H8]),
                bishops: Bitboard::from_squares(&[Square::C8, Square::F8]),
                knights: Bitboard::from_squares(&[Square::B8, Square::G8]),
                pawns: Bitboard::from_squares(&[
                    Square::A7,
                    Square::B7,
                    Square::C7,
                    Square::D7,
                    Square::E7,
                    Square::F7,
                    Square::G7,
                    Square::H7,
                ]),
            },
        }
    }

    #[must_use]
    pub(super) fn all(&self) -> Bitboard {
        self.king | self.queens | self.rooks | self.bishops | self.knights | self.pawns
    }

    #[must_use]
    pub(super) fn bitboard_for_mut(&mut self, piece: PieceKind) -> &mut Bitboard {
        match piece {
            PieceKind::King => &mut self.king,
            PieceKind::Queen => &mut self.queens,
            PieceKind::Rook => &mut self.rooks,
            PieceKind::Bishop => &mut self.bishops,
            PieceKind::Knight => &mut self.knights,
            PieceKind::Pawn => &mut self.pawns,
        }
    }

    #[must_use]
    pub(super) fn at(&self, square: Square) -> Option<PieceKind> {
        if self.all().contains(square) {
            let kind = if self.king.contains(square) {
                PieceKind::King
            } else if self.pawns.contains(square) {
                PieceKind::Pawn
            } else if self.queens.contains(square) {
                PieceKind::Queen
            } else if self.rooks.contains(square) {
                PieceKind::Rook
            } else if self.bishops.contains(square) {
                PieceKind::Bishop
            } else {
                PieceKind::Knight
            };
            return Some(kind);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::chess::core::{BOARD_WIDTH, Rank, Square};

    #[test]
    fn basics() {
        assert_eq!(Bitboard::full().bits, u64::MAX);
        assert_eq!(Bitboard::empty().bits, u64::MIN);

        assert_eq!(Bitboard::from(Square::A1).bits, 1);
        assert_eq!(Bitboard::from(Square::B1).bits, 2);
        assert_eq!(Bitboard::from(Square::D1).bits, 8);
        assert_eq!(Bitboard::from(Square::H8).bits, 1u64 << 63);

        assert_eq!(
            Bitboard::from(Square::D1) | Bitboard::from(Square::B1),
            Bitboard::from_bits(0b10 | 0b1000)
        );
    }

    #[test]
    fn set_basics() {
        // Create a starting position.
        let white = Pieces::starting(Player::White);
        let black = Pieces::starting(Player::Black);

        // Check that each player has 16 pieces.
        assert_eq!(white.all().bits.count_ones(), 16);
        assert_eq!(black.all().bits.count_ones(), 16);
        // Check that each player has correct number of pieces (previous check
        // was not enough to confirm there are no overlaps).
        assert_eq!(white.king.bits.count_ones(), 1);
        assert_eq!(black.king.bits.count_ones(), 1);
        assert_eq!(white.queens.bits.count_ones(), 1);
        assert_eq!(black.queens.bits.count_ones(), 1);
        assert_eq!(white.rooks.bits.count_ones(), 2);
        assert_eq!(black.rooks.bits.count_ones(), 2);
        assert_eq!(white.bishops.bits.count_ones(), 2);
        assert_eq!(black.bishops.bits.count_ones(), 2);
        assert_eq!(white.knights.bits.count_ones(), 2);
        assert_eq!(black.knights.bits.count_ones(), 2);
        assert_eq!(white.pawns.bits.count_ones(), 8);
        assert_eq!(black.pawns.bits.count_ones(), 8);

        // Check few positions manually.
        assert_eq!(white.queens.bits, 1 << 3);
        assert_eq!(black.queens.bits, 1 << (3 + 8 * 7));

        // Rank masks.
        assert_eq!(
            Rank::Rank1.mask() << u32::from(BOARD_WIDTH),
            Rank::Rank2.mask()
        );
        assert_eq!(
            Rank::Rank5.mask() >> u32::from(BOARD_WIDTH),
            Rank::Rank4.mask()
        );
    }

    #[test]
    fn bitboard_iterator() {
        let white = Pieces::starting(Player::White);

        let mut it = white.king.iter();
        assert_eq!(it.next(), Some(Square::E1));
        assert!(it.next().is_none());

        let mut it = white.bishops.iter();
        assert_eq!(it.next(), Some(Square::C1));
        assert_eq!(it.next(), Some(Square::F1));
        assert!(it.next().is_none());

        // The order is important here: we are iterating from least significant
        // bits to most significant bits.
        assert_eq!(
            white.pawns.iter().collect::<Vec<_>>(),
            vec![
                Square::A2,
                Square::B2,
                Square::C2,
                Square::D2,
                Square::E2,
                Square::F2,
                Square::G2,
                Square::H2,
            ]
        );
    }

    #[test]
    fn set_ops() {
        let bb = Bitboard::from_squares(&[
            Square::A1,
            Square::B1,
            Square::C1,
            Square::D1,
            Square::E1,
            Square::F1,
            Square::H1,
            Square::A2,
            Square::B2,
            Square::C2,
            Square::D2,
            Square::G2,
            Square::F2,
            Square::H2,
            Square::F3,
            Square::E4,
            Square::E5,
            Square::C6,
            Square::A7,
            Square::B7,
            Square::C7,
            Square::D7,
            Square::F7,
            Square::G7,
            Square::H7,
            Square::A8,
            Square::C8,
            Square::D8,
            Square::E8,
            Square::F8,
            Square::G8,
            Square::H8,
        ]);
        assert_eq!(
            format!("{:?}", bb),
            "1 . 1 1 1 1 1 1\n\
            1 1 1 1 . 1 1 1\n\
            . . 1 . . . . .\n\
            . . . . 1 . . .\n\
            . . . . 1 . . .\n\
            . . . . . 1 . .\n\
            1 1 1 1 . 1 1 1\n\
            1 1 1 1 1 1 . 1"
        );
        assert_eq!(
            format!("{:?}", !bb),
            ". 1 . . . . . .\n\
            . . . . 1 . . .\n\
            1 1 . 1 1 1 1 1\n\
            1 1 1 1 . 1 1 1\n\
            1 1 1 1 . 1 1 1\n\
            1 1 1 1 1 . 1 1\n\
            . . . . 1 . . .\n\
            . . . . . . 1 ."
        );
        assert_eq!(
            format!(
                "{:?}",
                bb - Bitboard::from_squares(&[Square::A1, Square::E4, Square::G8])
            ),
            "1 . 1 1 1 1 . 1\n\
            1 1 1 1 . 1 1 1\n\
            . . 1 . . . . .\n\
            . . . . 1 . . .\n\
            . . . . . . . .\n\
            . . . . . 1 . .\n\
            1 1 1 1 . 1 1 1\n\
            . 1 1 1 1 1 . 1"
        );
        assert_eq!(!!bb, bb);
        assert_eq!(bb - !bb, bb);
    }

    #[test]
    // Check the debug output for few bitboards.
    fn bitboard_dump() {
        assert_eq!(
            format!("{:?}", Bitboard::empty()),
            ". . . . . . . .\n\
             . . . . . . . .\n\
             . . . . . . . .\n\
             . . . . . . . .\n\
             . . . . . . . .\n\
             . . . . . . . .\n\
             . . . . . . . .\n\
             . . . . . . . ."
        );
        assert_eq!(
            format!("{:?}", Bitboard::full()),
            "1 1 1 1 1 1 1 1\n\
             1 1 1 1 1 1 1 1\n\
             1 1 1 1 1 1 1 1\n\
             1 1 1 1 1 1 1 1\n\
             1 1 1 1 1 1 1 1\n\
             1 1 1 1 1 1 1 1\n\
             1 1 1 1 1 1 1 1\n\
             1 1 1 1 1 1 1 1"
        );
        assert_eq!(
            format!(
                "{:?}",
                Bitboard::from(Square::G5) | Bitboard::from(Square::B8)
            ),
            ". 1 . . . . . .\n\
             . . . . . . . .\n\
             . . . . . . . .\n\
             . . . . . . 1 .\n\
             . . . . . . . .\n\
             . . . . . . . .\n\
             . . . . . . . .\n\
             . . . . . . . ."
        );
    }

    #[test]
    fn set_dump() {
        let white = Pieces::starting(Player::White);
        let black = Pieces::starting(Player::Black);

        assert_eq!(
            format!("{:?}", black.all()),
            "1 1 1 1 1 1 1 1\n\
             1 1 1 1 1 1 1 1\n\
             . . . . . . . .\n\
             . . . . . . . .\n\
             . . . . . . . .\n\
             . . . . . . . .\n\
             . . . . . . . .\n\
             . . . . . . . ."
        );
        assert_eq!(
            format!("{:?}", white.all() | black.all()),
            "1 1 1 1 1 1 1 1\n\
             1 1 1 1 1 1 1 1\n\
             . . . . . . . .\n\
             . . . . . . . .\n\
             . . . . . . . .\n\
             . . . . . . . .\n\
             1 1 1 1 1 1 1 1\n\
             1 1 1 1 1 1 1 1"
        );

        assert_eq!(
            format!("{:?}", white.king),
            ". . . . . . . .\n\
             . . . . . . . .\n\
             . . . . . . . .\n\
             . . . . . . . .\n\
             . . . . . . . .\n\
             . . . . . . . .\n\
             . . . . . . . .\n\
             . . . . 1 . . ."
        );
        assert_eq!(
            format!("{:?}", black.pawns),
            ". . . . . . . .\n\
             1 1 1 1 1 1 1 1\n\
             . . . . . . . .\n\
             . . . . . . . .\n\
             . . . . . . . .\n\
             . . . . . . . .\n\
             . . . . . . . .\n\
             . . . . . . . ."
        );
        assert_eq!(
            format!("{:?}", black.knights),
            ". 1 . . . . 1 .\n\
             . . . . . . . .\n\
             . . . . . . . .\n\
             . . . . . . . .\n\
             . . . . . . . .\n\
             . . . . . . . .\n\
             . . . . . . . .\n\
             . . . . . . . ."
        );
    }

    #[test]
    fn flip_perspective() {}
}
