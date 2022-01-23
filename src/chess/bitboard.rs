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

use std::fmt;
use std::ops::{BitAnd, BitOr, BitOrAssign, BitXor};

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
pub struct Bitboard(u64);

impl Bitboard {
    pub fn data(&self) -> u64 {
        self.0
    }

    pub fn empty() -> Self {
        Self::default()
    }

    pub fn full() -> Self {
        Self(u64::MAX)
    }

    pub(in crate::chess) fn with_squares(squares: &[Square]) -> Self {
        let mut result = Self::default();
        for square in squares {
            result |= Bitboard::from(*square);
        }
        result
    }

    pub(in crate::chess) fn is_set(&self, square: Square) -> bool {
        (self.data() & (1u64 << square as u8)) > 0
    }
}

impl BitOr for Bitboard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.data().bitor(rhs.data()))
    }
}

impl BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0.bitor_assign(rhs.data());
    }
}

impl BitAnd for Bitboard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.data().bitand(rhs.data()))
    }
}

impl BitXor for Bitboard {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.data().bitxor(rhs.data()))
    }
}

impl From<Square> for Bitboard {
    fn from(square: Square) -> Self {
        (1u64 << square as u8).into()
    }
}

impl From<u64> for Bitboard {
    fn from(data: u64) -> Self {
        Bitboard(data)
    }
}

const LINE_SEPARATOR: &str = "\n";
const SQUARE_SEPARATOR: &str = " ";

impl fmt::Debug for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: This is quite verbose. Refactor or explain what is happening.
        write!(
            f,
            "{}",
            format!("{:#066b}", self.data())
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

    pub(in crate::chess) fn new_black() -> Self {
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
pub struct Board {
    pub(in crate::chess) white_pieces: BitboardSet,
    pub(in crate::chess) black_pieces: BitboardSet,
}

impl Board {
    pub fn starting() -> Self {
        Self {
            white_pieces: BitboardSet::new_white(),
            black_pieces: BitboardSet::new_black(),
        }
    }

    pub fn empty() -> Self {
        Self {
            white_pieces: BitboardSet::empty(),
            black_pieces: BitboardSet::empty(),
        }
    }

    /// WARNING: This is slow and inefficient for Bitboard-based piece-centric
    /// representation. Use with caution.
    pub fn at(self, square: Square) -> Option<Piece> {
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

#[cfg(test)]
mod test {
    use super::{Bitboard, BitboardSet, Board};
    use crate::chess::core::Square;

    #[test]
    fn basics() {
        assert_eq!(std::mem::size_of::<Bitboard>(), 8);
        assert_eq!(Bitboard::full().data(), u64::MAX);
        assert_eq!(Bitboard::default().data(), u64::MIN);

        assert_eq!(Bitboard::from(Square::A1).data(), 1);
        assert_eq!(Bitboard::from(Square::B1).data(), 2);
        assert_eq!(Bitboard::from(Square::D1).data(), 8);
        assert_eq!(Bitboard::from(Square::H8).data(), 1u64 << 63);

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
        assert_eq!(white.all().data().count_ones(), 16);
        assert_eq!(black.all().data().count_ones(), 16);
        // Check that each player has correct number of pieces (previous check
        // was not enough to confirm there are no overlaps).
        assert_eq!(white.king.data().count_ones(), 1);
        assert_eq!(black.king.data().count_ones(), 1);
        assert_eq!(white.queen.data().count_ones(), 1);
        assert_eq!(black.queen.data().count_ones(), 1);
        assert_eq!(white.rooks.data().count_ones(), 2);
        assert_eq!(black.rooks.data().count_ones(), 2);
        assert_eq!(white.bishops.data().count_ones(), 2);
        assert_eq!(black.bishops.data().count_ones(), 2);
        assert_eq!(white.knights.data().count_ones(), 2);
        assert_eq!(black.knights.data().count_ones(), 2);
        assert_eq!(white.pawns.data().count_ones(), 8);
        assert_eq!(black.pawns.data().count_ones(), 8);

        // Check few positions manually.
        assert_eq!(white.queen.data(), 1 << 3);
        assert_eq!(black.queen.data(), 1 << (3 + 8 * 7));
    }

    #[test]
    // Check the debug output for few bitboards.
    fn bitboard_dump() {
        #[rustfmt::skip]
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
        #[rustfmt::skip]
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
        #[rustfmt::skip]
        assert_eq!(
            format!("{:?}", Bitboard::from(Square::G5) | Bitboard::from(Square::B8)),
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

        #[rustfmt::skip]
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
        #[rustfmt::skip]
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

        #[rustfmt::skip]
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
        #[rustfmt::skip]
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
        #[rustfmt::skip]
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
        #[rustfmt::skip]
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
        #[rustfmt::skip]
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
