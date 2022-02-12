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
//! [Bitboards]: https://www.chessprogramming.org/Bitboards

use std::fmt::Write;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, Not, Shl, Shr, Sub, SubAssign};
use std::{fmt, mem};

use itertools::Itertools;
use strum::IntoEnumIterator;

use crate::chess::core::{
    Direction,
    File,
    Piece,
    PieceKind,
    Player,
    Rank,
    Square,
    BOARD_SIZE,
    BOARD_WIDTH,
};

/// Represents a set of squares and provides common operations (e.g. AND, OR,
/// XOR) over these sets. Each bit corresponds to one of 64 squares of the chess
/// board.
///
/// Mirroring [`Square`] semantics, the least significant
/// bit corresponds to A1, and the most significant bit - to H8.
///
/// Bitboard is a thin wrapper around [u64].
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Bitboard {
    bits: u64,
}

impl Bitboard {
    /// Constructs Bitboard from pre-calculated bits.
    #[must_use]
    pub(super) const fn from_bits(bits: u64) -> Self {
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
    pub(super) fn from_squares(squares: &[Square]) -> Self {
        let mut result = Self::empty();
        for square in squares {
            result |= Self::from(*square);
        }
        result
    }

    /// Adds given square to the set.
    pub(super) fn extend(&mut self, square: Square) {
        *self |= Self::from(square)
    }

    /// Adds given square to the set.
    pub(super) fn clear(&mut self, square: Square) {
        *self &= !Self::from(square)
    }

    /// Returns true if this bitboard contains given square.
    #[must_use]
    pub(super) fn contains(self, square: Square) -> bool {
        (self.bits & (1u64 << square as u8)) != 0
    }

    #[must_use]
    pub(super) fn as_square(self) -> Square {
        debug_assert!(self.bits.count_ones() == 1);
        unsafe { std::mem::transmute(self.bits.trailing_zeros() as u8) }
    }

    #[must_use]
    pub(super) fn count(self) -> u32 {
        self.bits.count_ones()
    }

    #[must_use]
    pub(super) fn is_empty(self) -> bool {
        self.count() == 0
    }

    #[must_use]
    pub(super) fn has_any(self) -> bool {
        self.count() != 0
    }

    #[must_use]
    pub(super) fn shift(self, direction: Direction) -> Self {
        match direction {
            Direction::Up => self << u32::from(BOARD_WIDTH),
            Direction::Down => self >> u32::from(BOARD_WIDTH),
            Direction::Left => self << 1,
            Direction::Right => self >> 1,
            _ => unreachable!(),
        }
    }

    /// An efficient way to iterate over the set squares.
    #[must_use]
    pub(super) fn iter(self) -> BitboardIterator {
        BitboardIterator { bits: self.bits }
    }
}

impl fmt::Debug for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: This is quite verbose. Refactor or explain what is happening.
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
        self.bits.bitand_assign(rhs.bits)
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
        self.bitand_assign(!rhs)
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
// TODO: Try De Brujin Multiplication and see if it's faster (via benchmarks)
// than trailing zeros as reported by some developers (even though intuitively
// trailing zeros should be much faster because it would compile to a processor
// instruction):
// https://www.chessprogramming.org/BitScan#De_Bruijn_Multiplication
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
        self.bits ^= 1 << next_index;
        // For performance reasons, it's better to convert directly: the
        // conversion is safe because trailing_zeros() will return a number in
        // 0..64 range.
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
        Ok(unsafe { std::mem::transmute(self.bits.trailing_zeros() as u8) })
    }
}

/// Piece-centric representation of all material owned by one player. Uses
/// [Bitboard] to store a set of squares occupied by each piece. The main user
/// is [`crate::chess::position::Position`], [Bitboard] is not very useful on
/// its own.
#[derive(Clone, PartialEq, Eq)]
pub(super) struct Pieces {
    pub(super) king: Bitboard,
    pub(super) queens: Bitboard,
    pub(super) rooks: Bitboard,
    pub(super) bishops: Bitboard,
    // TODO: Store "all" instead.
    pub(super) knights: Bitboard,
    pub(super) pawns: Bitboard,
}

impl Pieces {
    pub(super) fn empty() -> Self {
        Self {
            king: Bitboard::empty(),
            queens: Bitboard::empty(),
            rooks: Bitboard::empty(),
            bishops: Bitboard::empty(),
            knights: Bitboard::empty(),
            pawns: Bitboard::empty(),
        }
    }

    pub(super) fn new_white() -> Self {
        Self {
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
        }
    }

    pub(super) fn new_black() -> Self {
        // TODO: Implement flip and return new_white().flip() to prevent copying
        // code.
        Self {
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
        }
    }

    pub(super) fn all(&self) -> Bitboard {
        self.king | self.queens | self.rooks | self.bishops | self.knights | self.pawns
    }

    pub(super) fn bitboard_for(&mut self, piece: PieceKind) -> &mut Bitboard {
        match piece {
            PieceKind::King => &mut self.king,
            PieceKind::Queen => &mut self.queens,
            PieceKind::Rook => &mut self.rooks,
            PieceKind::Bishop => &mut self.bishops,
            PieceKind::Knight => &mut self.knights,
            PieceKind::Pawn => &mut self.pawns,
        }
    }

    // TODO: Maybe completely disallow this? If we have the Square ->
    // Option<Piece> mapping, this is potentially obsolete.
    pub(super) fn at(&self, square: Square) -> Option<PieceKind> {
        if self.all().contains(square) {
            let mut kind = if self.king.contains(square) {
                PieceKind::King
            } else {
                PieceKind::Pawn
            };
            if self.king.contains(square) {
                kind = PieceKind::King;
            }
            if self.queens.contains(square) {
                kind = PieceKind::Queen;
            }
            if self.rooks.contains(square) {
                kind = PieceKind::Rook;
            }
            if self.bishops.contains(square) {
                kind = PieceKind::Bishop;
            }
            if self.knights.contains(square) {
                kind = PieceKind::Knight;
            }
            return Some(kind);
        }
        None
    }

    pub(super) fn clear(&mut self, square: Square) {
        self.king.clear(square);
        self.queens.clear(square);
        self.rooks.clear(square);
        self.bishops.clear(square);
        self.knights.clear(square);
        self.pawns.clear(square);
    }

    pub(super) fn iter(&self) -> PiecesIterator {
        PiecesIterator {
            pieces: self,
            index: 0,
        }
    }
}

// TODO: Document.
pub(super) struct PiecesIterator<'a> {
    pieces: &'a Pieces,
    index: u8,
}

impl Iterator for PiecesIterator<'_> {
    type Item = (PieceKind, Bitboard);

    fn next(&mut self) -> Option<Self::Item> {
        let result = match self.index {
            0 => Some((PieceKind::King, self.pieces.king)),
            1 => Some((PieceKind::Queen, self.pieces.queens)),
            2 => Some((PieceKind::Rook, self.pieces.rooks)),
            3 => Some((PieceKind::Bishop, self.pieces.bishops)),
            4 => Some((PieceKind::Knight, self.pieces.knights)),
            5 => Some((PieceKind::Pawn, self.pieces.pawns)),
            6 => None,
            _ => unreachable!(),
        };
        self.index += 1;
        result
    }
}

/// Piece-centric implementation of the chess board. This is the "back-end" of
/// the chess engine, an efficient board representation is crucial for
/// performance. An alternative implementation would be Square-Piece table but
/// both have different trade-offs and scenarios where they are efficient. It is
/// likely that the best overall performance can be achieved by keeping both to
/// complement each other.
#[derive(Clone, PartialEq, Eq)]
pub(super) struct Board {
    pub(super) white_pieces: Pieces,
    pub(super) black_pieces: Pieces,
}

impl Board {
    #[must_use]
    pub(super) fn starting() -> Self {
        Self {
            white_pieces: Pieces::new_white(),
            black_pieces: Pieces::new_black(),
        }
    }

    // Constructs an empty Board to be filled by the board and position builder.
    #[must_use]
    pub(super) fn empty() -> Self {
        Self {
            white_pieces: Pieces::empty(),
            black_pieces: Pieces::empty(),
        }
    }

    #[must_use]
    pub(super) fn player_pieces(&self, player: Player) -> &Pieces {
        match player {
            Player::White => &self.white_pieces,
            Player::Black => &self.black_pieces,
        }
    }

    // WARNING: This is slow and inefficient for Bitboard-based piece-centric
    // representation. Use with caution.
    // TODO: Completely disallow bitboard.at()?
    #[must_use]
    pub(super) fn at(&self, square: Square) -> Option<Piece> {
        if let Some(kind) = self.white_pieces.at(square) {
            return Some(Piece {
                owner: Player::White,
                kind,
            });
        }
        if let Some(kind) = self.black_pieces.at(square) {
            return Some(Piece {
                owner: Player::Black,
                kind,
            });
        }
        None
    }
}

impl fmt::Display for Board {
    /// Prints board representation in FEN format.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for rank in Rank::iter().rev() {
            let mut empty_squares = 0i32;
            for file in File::iter() {
                let square = Square::new(file, rank);
                if let Some(piece) = self.at(square) {
                    if empty_squares != 0 {
                        write!(f, "{empty_squares}")?;
                        empty_squares = 0;
                    }
                    write!(f, "{}", piece)?;
                } else {
                    empty_squares += 1;
                }
            }
            if empty_squares != 0 {
                write!(f, "{empty_squares}")?;
            }
            if rank != Rank::One {
                const RANK_SEPARATOR: char = '/';
                write!(f, "{RANK_SEPARATOR}")?;
            }
        }
        Ok(())
    }
}

impl fmt::Debug for Board {
    /// Dumps the board in a simple format ('.' for empty square, FEN algebraic
    /// symbol for piece) a-la Stockfish "debug" command in UCI mode.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for rank in Rank::iter().rev() {
            for file in File::iter() {
                match self.at(Square::new(file, rank)) {
                    Some(piece) => write!(f, "{piece}"),
                    None => f.write_char('.'),
                }?;
                if file != File::H {
                    write!(f, "{}", SQUARE_SEPARATOR)?;
                }
            }
            if rank != Rank::One {
                write!(f, "{}", LINE_SEPARATOR)?;
            }
        }
        Ok(())
    }
}

const LINE_SEPARATOR: &str = "\n";
const SQUARE_SEPARATOR: &str = " ";

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::chess::core::{Rank, Square, BOARD_WIDTH};

    #[test]
    fn basics() {
        assert_eq!(std::mem::size_of::<Bitboard>(), 8);
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
        let white = Pieces::new_white();
        let black = Pieces::new_black();

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
        assert_eq!(Rank::One.mask() << u32::from(BOARD_WIDTH), Rank::Two.mask());
        assert_eq!(
            Rank::Five.mask() >> u32::from(BOARD_WIDTH),
            Rank::Four.mask()
        );
    }

    #[test]
    fn bitboard_iterator() {
        let white = Pieces::new_white();

        let mut it = white.king.iter();
        assert_eq!(it.next(), Some(Square::E1));
        assert_eq!(it.next(), None);

        let mut it = white.bishops.iter();
        assert_eq!(it.next(), Some(Square::C1));
        assert_eq!(it.next(), Some(Square::F1));
        assert_eq!(it.next(), None);

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
        let bitboard = Bitboard::from_squares(&[
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
            format!("{:?}", bitboard),
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
            format!("{:?}", !bitboard),
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
                bitboard - Bitboard::from_squares(&[Square::A1, Square::E4, Square::G8])
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
        assert_eq!(!!bitboard, bitboard);
        assert_eq!(bitboard - !bitboard, bitboard);
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
        let white = Pieces::new_white();
        let black = Pieces::new_black();

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
    fn starting_board() {
        let starting_board = Board::starting();
        assert_eq!(
            format!("{:?}", starting_board),
            "r n b q k b n r\n\
             p p p p p p p p\n\
             . . . . . . . .\n\
             . . . . . . . .\n\
             . . . . . . . .\n\
             . . . . . . . .\n\
             P P P P P P P P\n\
             R N B Q K B N R"
        );
        assert_eq!(
            starting_board.white_pieces.all() | starting_board.black_pieces.all(),
            Rank::One.mask() | Rank::Two.mask() | Rank::Seven.mask() | Rank::Eight.mask()
        );
        assert_eq!(
            !(starting_board.white_pieces.all() | starting_board.black_pieces.all()),
            Rank::Three.mask() | Rank::Four.mask() | Rank::Five.mask() | Rank::Six.mask()
        );
        assert_eq!(
            starting_board.to_string(),
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR"
        );
    }

    #[test]
    fn empty_board() {
        assert_eq!(
            format!("{:?}", Board::empty()),
            ". . . . . . . .\n\
             . . . . . . . .\n\
             . . . . . . . .\n\
             . . . . . . . .\n\
             . . . . . . . .\n\
             . . . . . . . .\n\
             . . . . . . . .\n\
             . . . . . . . ."
        );
        assert_eq!(Board::empty().to_string(), "8/8/8/8/8/8/8/8");
    }
}
