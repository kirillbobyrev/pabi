//! [`Bitboard`]-based representation for [`Board`]. Bitboard utilizes the fact
//! that modern processors operate on 64 bit integers, and the bit operations
//! can be performed simultaneously. This results in very efficient calculation
//! of possible attack vectors and other meaningful features that are calculated
//! to evaluate a position on the board. The disadvantage is complexity that
//! comes with bitboard implementation and inefficiency of some operations like
//! "get piece type on given square" (efficiently handled by Square-centric
//! board implementations).
//!
//! [Bitboard]: https://www.chessprogramming.org/Bitboards
// TODO: This comment needs revamp.

use std::ops::{BitAnd, BitOr, BitOrAssign, BitXor, Not, Sub};
use std::{fmt, mem};

use itertools::Itertools;
use strum::IntoEnumIterator;

use crate::chess::core::{File, Piece, PieceKind, Player, Rank, Square, BOARD_SIZE, BOARD_WIDTH};

/// Represents a set of squares and provides common operations (e.g. AND, OR,
/// XOR) over these sets. Each bit corresponds to one of 64 squares of the chess
/// board.
///
/// Mirroring [`Square`] semantics, the least significant
/// bit corresponds to A1, and the most significant bit - to H8.
///
/// Bitboard is a thin wrapper around [u64].
#[derive(Copy, Clone, Default, PartialEq, Eq)]
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
        Self { bits: 0 }
    }

    /// Constructs a bitboard representing the universal set, it contains all
    /// squares by setting all bits to binary one.
    #[must_use]
    pub const fn full() -> Self {
        Self { bits: u64::MAX }
    }

    pub(in crate::chess) fn from_squares(squares: &[Square]) -> Self {
        let mut result = Self::default();
        for square in squares {
            result |= Self::from(*square);
        }
        result
    }

    pub(in crate::chess) fn is_set(self, square: Square) -> bool {
        (self.bits & (1u64 << square as u8)) > 0
    }

    pub(in crate::chess) fn iter(self) -> BitboardIterator {
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
        Self {
            bits: self.bits.bitor(rhs.bits),
        }
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
        Self {
            bits: self.bits.bitand(rhs.bits),
        }
    }
}

impl BitXor for Bitboard {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self {
            bits: self.bits.bitxor(rhs.bits),
        }
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

impl Not for Bitboard {
    type Output = Self;

    /// Returns [complement
    /// set](https://en.wikipedia.org/wiki/Complement_%28set_theory%29) of Self,
    /// i.e. flipping the set squares to unset and vice versa.
    fn not(self) -> Self::Output {
        Self { bits: !self.bits }
    }
}

impl From<Square> for Bitboard {
    fn from(square: Square) -> Self {
        (1u64 << square as u8).into()
    }
}

impl From<u64> for Bitboard {
    fn from(bits: u64) -> Self {
        Self { bits }
    }
}

/// Iterates over set squares in a given [Bitboard] from least significant bits
/// to most significant bits.
pub(in crate::chess) struct BitboardIterator {
    // TODO: Check if operating on the actual Bitboard will not hurt the
    // performance. This iterator is likely to be on the hot path, so changing
    // this code is sensitive. The optimizer might be smart enough do deal with
    // back-and-forth redundant integer conversions but that has to be
    // benchmarked.
    bits: u64,
}

impl Iterator for BitboardIterator {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bits == 0 {
            return None;
        }
        // Get the least significant set 1 and consume it from the iterator by
        // resetting the bit.
        let next_index = self.bits.trailing_zeros();
        self.bits ^= 1 << next_index;
        // For performance reasons, it's better to convert directly: the
        // conversion is safe because trailing_zeros() will return a number in
        // 0..64 range.
        Some(unsafe { mem::transmute(next_index as u8) })
    }
}

/// Piece-centric representation of all material owned by one player. Uses
/// [Bitboard] to store a set of squares occupied by each piece. The main user
/// is [`crate::chess::position::Position`], [Bitboard] is not very useful on
/// its own.
///
/// Defaults to empty board.
// TODO: Caching all() and either replacing it or adding to the set might
// improve performance. This is what lc0 does:
// https://github.com/LeelaChessZero/lc0/blob/d2e372e59cd9188315d5c02a20e0bdce88033bc5/src/chess/board.h
#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub(in crate::chess) struct BitboardSet {
    pub(in crate::chess) king: Bitboard,
    pub(in crate::chess) queen: Bitboard,
    pub(in crate::chess) rooks: Bitboard,
    pub(in crate::chess) bishops: Bitboard,
    pub(in crate::chess) knights: Bitboard,
    pub(in crate::chess) pawns: Bitboard,
}

impl BitboardSet {
    pub(in crate::chess) fn empty() -> Self {
        Self::default()
    }

    pub(in crate::chess) fn new_white() -> Self {
        Self {
            king: Square::E1.into(),
            queen: Square::D1.into(),
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

    pub(in crate::chess) fn new_black() -> Self {
        // TODO: Implement flip and return new_white().flip() to prevent copying code.
        Self {
            king: Square::E8.into(),
            queen: Square::D8.into(),
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

    pub(in crate::chess) fn all(self) -> Bitboard {
        self.king | self.queen | self.rooks | self.bishops | self.knights | self.pawns
    }

    pub(in crate::chess) fn bitboard_for(&mut self, piece: PieceKind) -> &mut Bitboard {
        match piece {
            PieceKind::King => &mut self.king,
            PieceKind::Queen => &mut self.queen,
            PieceKind::Rook => &mut self.rooks,
            PieceKind::Bishop => &mut self.bishops,
            PieceKind::Knight => &mut self.knights,
            PieceKind::Pawn => &mut self.pawns,
        }
    }

    // TODO: Maybe completely disallow this? If we have the Square ->
    // Option<Piece> mapping, this is potentially obsolete.
    pub(in crate::chess) fn at(self, square: Square) -> Option<PieceKind> {
        if self.all().is_set(square) {
            let mut kind = if self.king.is_set(square) {
                PieceKind::King
            } else {
                PieceKind::Pawn
            };
            if self.king.is_set(square) {
                kind = PieceKind::King;
            }
            if self.queen.is_set(square) {
                kind = PieceKind::Queen;
            }
            if self.rooks.is_set(square) {
                kind = PieceKind::Rook;
            }
            if self.bishops.is_set(square) {
                kind = PieceKind::Bishop;
            }
            if self.knights.is_set(square) {
                kind = PieceKind::Knight;
            }
            return Some(kind);
        }
        None
    }
}

/// Piece-centric implementation of the chess board. This is
/// the "back-end" of the chess engine, efficient board representation is
/// crucial for performance.
#[derive(Copy, Clone, PartialEq, Eq)]
pub(in crate::chess) struct Board {
    pub(in crate::chess) white_pieces: BitboardSet,
    pub(in crate::chess) black_pieces: BitboardSet,
}

impl Board {
    #[must_use]
    pub(in crate::chess) fn starting() -> Self {
        Self {
            white_pieces: BitboardSet::new_white(),
            black_pieces: BitboardSet::new_black(),
        }
    }

    // Constructs an empty Board to be filled by the board and position builder.
    #[must_use]
    pub(in crate::chess) fn empty() -> Self {
        Self {
            white_pieces: BitboardSet::empty(),
            black_pieces: BitboardSet::empty(),
        }
    }

    #[must_use]
    pub(in crate::chess) fn our_pieces(&self, player: Player) -> BitboardSet {
        match player {
            Player::White => self.white_pieces,
            Player::Black => self.black_pieces,
        }
    }

    // WARNING: This is slow and inefficient for Bitboard-based piece-centric
    // representation. Use with caution.
    // TODO: Completely disallow bitboard.at()?
    #[must_use]
    pub(in crate::chess) fn at(self, square: Square) -> Option<Piece> {
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

impl Default for Board {
    /// Returns a `Board` for starting position.
    fn default() -> Self {
        Self::empty()
    }
}

impl ToString for Board {
    /// Returns board representation in FEN format.
    fn to_string(&self) -> String {
        let mut result = String::new();
        for rank in Rank::iter().rev() {
            let mut empty_squares = 0i32;
            for file in File::iter() {
                let square = Square::new(file, rank);
                if let Some(piece) = self.at(square) {
                    if empty_squares != 0 {
                        result.push_str(format!("{empty_squares}").as_str());
                        empty_squares = 0;
                    }
                    result.push(piece.algebraic_symbol());
                } else {
                    empty_squares += 1;
                }
            }
            if empty_squares != 0 {
                result.push_str(format!("{empty_squares}").as_str());
            }
            if rank != Rank::One {
                const RANK_SEPARATOR: char = '/';
                result.push(RANK_SEPARATOR);
            }
        }
        result
    }
}

impl fmt::Debug for Board {
    /// Dumps the board in a simple format ('.' for empty square, FEN algebraic
    /// symbol for piece) a-la Stockfish "debug" command in UCI mode.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for rank in Rank::iter().rev() {
            for file in File::iter() {
                let ascii_symbol = match self.at(Square::new(file, rank)) {
                    Some(piece) => piece.algebraic_symbol(),
                    None => '.',
                };
                write!(f, "{}", ascii_symbol)?;
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

    use super::{Bitboard, BitboardSet, Board};
    use crate::chess::core::Square;

    #[test]
    fn basics() {
        assert_eq!(std::mem::size_of::<Bitboard>(), 8);
        assert_eq!(Bitboard::full().bits, u64::MAX);
        assert_eq!(Bitboard::default().bits, u64::MIN);

        assert_eq!(Bitboard::from(Square::A1).bits, 1);
        assert_eq!(Bitboard::from(Square::B1).bits, 2);
        assert_eq!(Bitboard::from(Square::D1).bits, 8);
        assert_eq!(Bitboard::from(Square::H8).bits, 1u64 << 63);

        assert_eq!(
            Bitboard::from(Square::D1) | Bitboard::from(Square::B1),
            Bitboard::from(0b10 | 0b1000)
        );
    }

    #[test]
    fn set_basics() {
        // Create a starting position.
        let white = BitboardSet::new_white();
        let black = BitboardSet::new_black();

        // Check that each player has 16 pieces.
        assert_eq!(white.all().bits.count_ones(), 16);
        assert_eq!(black.all().bits.count_ones(), 16);
        // Check that each player has correct number of pieces (previous check
        // was not enough to confirm there are no overlaps).
        assert_eq!(white.king.bits.count_ones(), 1);
        assert_eq!(black.king.bits.count_ones(), 1);
        assert_eq!(white.queen.bits.count_ones(), 1);
        assert_eq!(black.queen.bits.count_ones(), 1);
        assert_eq!(white.rooks.bits.count_ones(), 2);
        assert_eq!(black.rooks.bits.count_ones(), 2);
        assert_eq!(white.bishops.bits.count_ones(), 2);
        assert_eq!(black.bishops.bits.count_ones(), 2);
        assert_eq!(white.knights.bits.count_ones(), 2);
        assert_eq!(black.knights.bits.count_ones(), 2);
        assert_eq!(white.pawns.bits.count_ones(), 8);
        assert_eq!(black.pawns.bits.count_ones(), 8);

        // Check few positions manually.
        assert_eq!(white.queen.bits, 1 << 3);
        assert_eq!(black.queen.bits, 1 << (3 + 8 * 7));
    }

    #[test]
    fn bitboard_iterator() {
        let white = BitboardSet::new_white();

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
            format!("{:?}", Bitboard::default()),
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
        let white = BitboardSet::new_white();
        let black = BitboardSet::new_black();

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
    fn board_dump() {
        assert_eq!(
            format!("{:?}", Board::starting()),
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
            Board::starting().to_string(),
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR"
        );
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
