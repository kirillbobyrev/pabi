//! Chess primitives commonly used within [`crate::chess`].

use std::fmt::{self, Write};
use std::mem;

use anyhow::bail;
use itertools::Itertools;

use crate::chess::bitboard::Bitboard;

#[allow(missing_docs)]
pub const BOARD_WIDTH: u8 = 8;
#[allow(missing_docs)]
pub const BOARD_SIZE: u8 = BOARD_WIDTH * BOARD_WIDTH;

/// Represents any kind of a legal chess move. A move is the only way to mutate
/// [`crate::chess::position::Position`] and change the board state. Moves are
/// not sorted according to their potential "value" by the move generator. The
/// move representation has one-to-one correspondence with the UCI move
/// representation. The moves can also be indexed and fed as an input to the
/// Neural Network evaluators that would be able assess their potential without
/// evaluating post-states.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Move(u16);

impl Move {
    // First 6 bits are reserved for the `from` square.
    const FROM_MASK: u16 = 0b0000_0000_0011_1111;
    // Next 3 bits are reserved for the promotion (if any).
    const PROMOTION_MASK: u16 = 0b0111_0000_0000_0000;
    const PROMOTION_OFFSET: u8 = 12;
    const TO_MASK: u16 = 0b0000_1111_1100_0000;
    // Next 6 bits are reserved for the `to` square.
    const TO_OFFSET: u8 = 6;

    #[must_use]
    pub(super) fn new(from: Square, to: Square, promotion: Option<Promotion>) -> Self {
        let mut packed = from as u16 | ((to as u16) << Self::TO_OFFSET);
        if let Some(promo) = promotion {
            packed |= (promo as u16) << Self::PROMOTION_OFFSET;
        }
        Self(packed)
    }

    #[must_use]
    pub(super) fn from(&self) -> Square {
        let square = self.0 & Self::FROM_MASK;
        Square::try_from(square as u8).unwrap()
    }

    #[must_use]
    pub(super) fn to(&self) -> Square {
        let square = (self.0 & Self::TO_MASK) >> Self::TO_OFFSET;
        Square::try_from(square as u8).unwrap()
    }

    #[must_use]
    pub(super) fn promotion(&self) -> Option<Promotion> {
        let promo = (self.0 & Self::PROMOTION_MASK) >> Self::PROMOTION_OFFSET;
        unsafe { std::mem::transmute(promo as u8) }
    }

    /// Converts the move from UCI format to the internal representation. This
    /// is important for the communication between the engine and UCI server in
    /// `position` command.
    pub fn from_uci(uci: &str) -> anyhow::Result<Self> {
        Self::try_from(uci)
    }

    #[must_use]
    pub(super) fn as_packed_int(&self) -> u16 {
        self.0
    }
}

impl TryFrom<&str> for Move {
    type Error = anyhow::Error;

    fn try_from(uci: &str) -> anyhow::Result<Self> {
        match uci.len() {
            4 => Ok(Self::new(
                Square::try_from(&uci[..2])?,
                Square::try_from(&uci[2..4])?,
                None,
            )),
            5 => Ok(Self::new(
                Square::try_from(&uci[..2])?,
                Square::try_from(&uci[2..4])?,
                Some(Promotion::from(uci.chars().nth(4).unwrap())),
            )),
            _ => bail!("UCI move should be 4 or 5 characters long, got {uci}"),
        }
    }
}

impl fmt::Display for Move {
    /// Serializes a move in UCI format (used by [`pabi::uci`]).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.from(), self.to())?;
        if let Some(promotion) = self.promotion() {
            write!(f, "{}", PieceKind::from(promotion))?;
        }
        Ok(())
    }
}

/// Size of [`MoveList`] and an upper bound of moves in a chess position (which
/// [seems to be 218](https://www.chessprogramming.org/Chess_Position). 256 provides the best
/// performance through optimal memory alignment.
const MAX_MOVES: usize = 256;

/// Moves are stored on stack to avoid memory allocations and improve
/// performance. This is important for performance reasons and also prevents
/// unnecessary copying that would occur if the moves would be stored in
/// `std::Vec` with unknown capacity.
pub type MoveList = arrayvec::ArrayVec<Move, { MAX_MOVES }>;

/// Board squares: from left to right, from bottom to the top ([Little-Endian Rank-File Mapping]):
///
/// ```
/// use pabi::chess::core::Square;
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
/// use pabi::chess::core::Square;
/// use std::mem::size_of;
///
/// assert_eq!(size_of::<Square>(), 1);
/// ```
///
/// [Little-Endian Rank-File Mapping]: https://www.chessprogramming.org/Square_Mapping_Considerations#LittleEndianRankFileMapping
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[rustfmt::skip]
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
    /// Connects file (column) and rank (row) to form a full square.
    #[must_use]
    pub const fn new(file: File, rank: Rank) -> Self {
        unsafe { mem::transmute(file as u8 + (rank as u8) * BOARD_WIDTH) }
    }

    /// Returns file (column) on which the square is located.
    #[must_use]
    pub const fn file(self) -> File {
        unsafe { mem::transmute(self as u8 % BOARD_WIDTH) }
    }

    /// Returns rank (row) on which the square is located.
    #[must_use]
    pub const fn rank(self) -> Rank {
        unsafe { mem::transmute(self as u8 / BOARD_WIDTH) }
    }

    #[must_use]
    pub fn shift(self, direction: Direction) -> Option<Self> {
        let shift: i8 = match direction {
            Direction::Up => BOARD_WIDTH as i8,
            Direction::Down => -(BOARD_WIDTH as i8),
        };
        match Self::try_from(self as i8 + shift) {
            Ok(square) => Some(square),
            Err(_) => None,
        }
    }

    fn next(self) -> Option<Self> {
        let next = self as u8 + 1;
        if next == BOARD_SIZE {
            None
        } else {
            Some(unsafe { mem::transmute(next) })
        }
    }

    /// Creates an iterator over all squares, starting from A1 (0) to H8 (63).
    #[must_use]
    pub fn iter() -> SquareIterator {
        SquareIterator {
            current: Some(Self::A1),
        }
    }
}

impl TryFrom<u8> for Square {
    type Error = anyhow::Error;

    /// Creates a square given its position on the board.
    ///
    /// # Errors
    ///
    /// If given square index is outside 0..[`BOARD_SIZE`] range.
    fn try_from(square_index: u8) -> anyhow::Result<Self> {
        // Exclusive range patterns are not allowed until Rust 1.80.
        // https://github.com/rust-lang/rust/issues/37854
        const MAX_INDEX: u8 = BOARD_SIZE - 1;
        match square_index {
            0..=MAX_INDEX => Ok(unsafe { mem::transmute(square_index) }),
            _ => bail!("square index should be in 0..BOARD_SIZE, got {square_index}"),
        }
    }
}

impl TryFrom<i8> for Square {
    type Error = anyhow::Error;

    /// Creates a square given its position on the board.
    ///
    /// # Errors
    ///
    /// If given square index is outside 0..[`BOARD_SIZE`] range.
    fn try_from(square_index: i8) -> anyhow::Result<Self> {
        // Exclusive range patterns are not allowed until Rust 1.80.
        // https://github.com/rust-lang/rust/issues/37854
        const MAX_INDEX: i8 = BOARD_SIZE as i8 - 1;
        match square_index {
            0..=MAX_INDEX => Ok(unsafe { mem::transmute(square_index) }),
            _ => bail!("square index should be in 0..BOARD_SIZE, got {square_index}"),
        }
    }
}

impl TryFrom<&str> for Square {
    type Error = anyhow::Error;

    fn try_from(square: &str) -> anyhow::Result<Self> {
        let (file, rank) = match square.chars().collect_tuple() {
            Some((file, rank)) => (file, rank),
            None => bail!(
                "square should be two-char, got {square} with {} chars",
                square.bytes().len()
            ),
        };
        Ok(Self::new(file.try_into()?, rank.try_into()?))
    }
}

/// Iterates over squares in the order from A1 to H8, from left to right, from
/// bottom to the top.
pub struct SquareIterator {
    current: Option<Square>,
}

impl Iterator for SquareIterator {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.current;
        self.current = self.current.and_then(Square::next);
        result
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.file(), self.rank())
    }
}

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

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", (b'a' + *self as u8) as char)
    }
}

impl TryFrom<char> for File {
    type Error = anyhow::Error;

    fn try_from(file: char) -> anyhow::Result<Self> {
        match file {
            'a'..='h' => Ok(unsafe { mem::transmute(file as u8 - b'a') }),
            _ => bail!("file should be within 'a'..='h', got '{file}'"),
        }
    }
}

impl TryFrom<u8> for File {
    type Error = anyhow::Error;

    fn try_from(column: u8) -> anyhow::Result<Self> {
        match column {
            0..=7 => Ok(unsafe { mem::transmute(column) }),
            _ => bail!("file should be within 0..BOARD_WIDTH, got {column}"),
        }
    }
}

/// Represents a horizontal row of the chessboard. In chess notation, it is
/// represented with a number. The implementation assumes zero-based values
/// (i.e. rank 1 would be 0).
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[allow(missing_docs)]
pub enum Rank {
    Rank1,
    Rank2,
    Rank3,
    Rank4,
    Rank5,
    Rank6,
    Rank7,
    Rank8,
}

impl Rank {
    /// Returns a pre-calculated bitboard mask with 1s set for squares of the
    /// given rank.
    pub(super) const fn mask(self) -> Bitboard {
        match self {
            Self::Rank1 => Bitboard::from_bits(0x0000_0000_0000_00FF),
            Self::Rank2 => Bitboard::from_bits(0x0000_0000_0000_FF00),
            Self::Rank3 => Bitboard::from_bits(0x0000_0000_00FF_0000),
            Self::Rank4 => Bitboard::from_bits(0x0000_0000_FF00_0000),
            Self::Rank5 => Bitboard::from_bits(0x0000_00FF_0000_0000),
            Self::Rank6 => Bitboard::from_bits(0x0000_FF00_0000_0000),
            Self::Rank7 => Bitboard::from_bits(0x00FF_0000_0000_0000),
            Self::Rank8 => Bitboard::from_bits(0xFF00_0000_0000_0000),
        }
    }

    pub(super) const fn backrank(color: Color) -> Self {
        match color {
            Color::White => Self::Rank1,
            Color::Black => Self::Rank8,
        }
    }

    pub(super) const fn pawns_starting(color: Color) -> Self {
        match color {
            Color::White => Self::Rank2,
            Color::Black => Self::Rank7,
        }
    }
}

impl TryFrom<char> for Rank {
    type Error = anyhow::Error;

    fn try_from(rank: char) -> anyhow::Result<Self> {
        match rank {
            '1'..='8' => Ok(unsafe { mem::transmute(rank as u8 - b'1') }),
            _ => bail!("rank should be within '1'..='8', got '{rank}'"),
        }
    }
}

impl TryFrom<u8> for Rank {
    type Error = anyhow::Error;

    fn try_from(row: u8) -> anyhow::Result<Self> {
        match row {
            0..=7 => Ok(unsafe { mem::transmute(row) }),
            _ => bail!("rank should be within 0..BOARD_WIDTH, got {row}"),
        }
    }
}

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", *self as u8 + 1)
    }
}

/// A standard game of chess is played between two players: White (having the
/// advantage of the first turn) and Black.
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
}

impl Color {
    /// "Flips" the color.
    #[must_use]
    pub const fn opponent(self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }

    pub(super) const fn pawn_push_direction(self) -> Direction {
        match self {
            Self::White => Direction::Up,
            Self::Black => Direction::Down,
        }
    }
}

impl TryFrom<&str> for Color {
    type Error = anyhow::Error;

    fn try_from(color: &str) -> anyhow::Result<Self> {
        match color {
            "w" => Ok(Self::White),
            "b" => Ok(Self::Black),
            _ => bail!("color should be 'w' or 'b', got '{color}'"),
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                Self::White => 'w',
                Self::Black => 'b',
            }
        )
    }
}

/// Standard [chess pieces] types for one player.
///
/// [chess pieces]: https://en.wikipedia.org/wiki/Chess_piece
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd)]
pub enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl From<Promotion> for PieceKind {
    fn from(promotion: Promotion) -> Self {
        match promotion {
            Promotion::Knight => Self::Knight,
            Promotion::Bishop => Self::Bishop,
            Promotion::Rook => Self::Rook,
            Promotion::Queen => Self::Queen,
        }
    }
}

impl fmt::Display for PieceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char(match &self {
            Self::Pawn => 'p',
            Self::Knight => 'n',
            Self::Bishop => 'b',
            Self::Rook => 'r',
            Self::Queen => 'q',
            Self::King => 'k',
        })
    }
}

/// Represents a specific piece owned by a player.
pub struct Piece {
    #[allow(missing_docs)]
    pub color: Color,
    #[allow(missing_docs)]
    pub kind: PieceKind,
}

impl Piece {
    #[must_use]
    pub const fn plane(&self) -> usize {
        self.color as usize * 6 + self.kind as usize
    }
}

impl TryFrom<char> for Piece {
    type Error = anyhow::Error;

    fn try_from(symbol: char) -> anyhow::Result<Self> {
        match symbol {
            'P' => Ok(Self {
                color: Color::White,
                kind: PieceKind::Pawn,
            }),
            'N' => Ok(Self {
                color: Color::White,
                kind: PieceKind::Knight,
            }),
            'B' => Ok(Self {
                color: Color::White,
                kind: PieceKind::Bishop,
            }),
            'R' => Ok(Self {
                color: Color::White,
                kind: PieceKind::Rook,
            }),
            'Q' => Ok(Self {
                color: Color::White,
                kind: PieceKind::Queen,
            }),
            'K' => Ok(Self {
                color: Color::White,
                kind: PieceKind::King,
            }),
            'p' => Ok(Self {
                color: Color::Black,
                kind: PieceKind::Pawn,
            }),
            'n' => Ok(Self {
                color: Color::Black,
                kind: PieceKind::Knight,
            }),
            'b' => Ok(Self {
                color: Color::Black,
                kind: PieceKind::Bishop,
            }),
            'r' => Ok(Self {
                color: Color::Black,
                kind: PieceKind::Rook,
            }),
            'k' => Ok(Self {
                color: Color::Black,
                kind: PieceKind::King,
            }),
            'q' => Ok(Self {
                color: Color::Black,
                kind: PieceKind::Queen,
            }),
            _ => bail!("piece symbol should be in \"PNBRQKpnbrqk\", got '{symbol}'"),
        }
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_char(match (&self.color, &self.kind) {
            // White: uppercase symbols.
            (Color::White, PieceKind::Pawn) => 'P',
            (Color::White, PieceKind::Knight) => 'N',
            (Color::White, PieceKind::Bishop) => 'B',
            (Color::White, PieceKind::Rook) => 'R',
            (Color::White, PieceKind::Queen) => 'Q',
            (Color::White, PieceKind::King) => 'K',
            // Black: lowercase symbols.
            (Color::Black, PieceKind::Pawn) => 'p',
            (Color::Black, PieceKind::Knight) => 'n',
            (Color::Black, PieceKind::Bishop) => 'b',
            (Color::Black, PieceKind::Rook) => 'r',
            (Color::Black, PieceKind::Queen) => 'q',
            (Color::Black, PieceKind::King) => 'k',
        })
    }
}

bitflags::bitflags! {
    /// Track the ability to [castle] each side (kingside is often referred to
    /// as O-O or h-side castle, queenside -- O-O-O or a-side castle). When the
    /// king moves, player loses ability to castle. When the rook moves, player
    /// loses ability to castle to the side from which the rook moved.
    ///
    /// Castling is relatively straightforward in the Standard Chess but is
    /// often misunderstood in Chess960 (also known as Fischer Random Chess). An
    /// easy mnemonic is that the king and the rook end up on the same files for
    /// both Standard and FRC:
    ///
    /// - When castling h-side (short), the king ends up on [`File::G`] and the
    ///   rook on [`File::F`]
    /// - When castling a-side (long), the king ends up on [`File::C`] and the
    ///   rook on [`File::D`]
    ///
    /// The full rules are:
    ///
    /// - The king and the castling rook must not have previously moved.
    /// - No square from the king's initial square to its final square may be under
    ///   attack by an enemy piece.
    /// - All the squares between the king's initial and final squares
    ///   (including the final square), and all the squares between the castling
    ///   rook's initial and final squares (including the final square), must be
    ///   vacant except for the king and castling rook.
    ///
    /// [castle]: https://www.chessprogramming.org/Castling
    // TODO: Update with castling squares for Chess960.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct CastleRights : u8 {
        #[allow(missing_docs)]
        const NONE = 0;
        #[allow(missing_docs)]
        const WHITE_SHORT = 0b1000;
        #[allow(missing_docs)]
        const WHITE_LONG = 0b0100;
        #[allow(missing_docs)]
        const WHITE_BOTH = Self::WHITE_SHORT.bits() | Self::WHITE_LONG.bits();
        #[allow(missing_docs)]
        const BLACK_SHORT = 0b0010;
        #[allow(missing_docs)]
        const BLACK_LONG = 0b0001;
        #[allow(missing_docs)]
        const BLACK_BOTH = Self::BLACK_SHORT.bits() | Self::BLACK_LONG.bits();
        #[allow(missing_docs)]
        const ALL = Self::WHITE_BOTH.bits() | Self::BLACK_BOTH.bits();
    }
}

impl TryFrom<&str> for CastleRights {
    type Error = anyhow::Error;

    /// Parses [`CastleRights`] for both players from the FEN format. The user
    /// is responsible for providing valid input cleaned up from the actual FEN
    /// chunk.
    ///
    /// # Errors
    ///
    /// Returns [`anyhow::Error`] if given pattern does not match
    ///
    /// [`CastleRights`] := (K)? (Q)? (k)? (q)?
    ///
    /// Note that both letters have to be either uppercase or lowercase.
    fn try_from(input: &str) -> anyhow::Result<Self> {
        // Enumerate all possibilities.
        match input.as_bytes() {
            // K Q k q
            // - - - -
            // 0 0 0 0
            b"-" => Ok(Self::NONE),
            // 0 0 0 1
            b"q" => Ok(Self::BLACK_LONG),
            // 0 0 1 0
            b"k" => Ok(Self::BLACK_SHORT),
            // 0 0 1 1
            b"kq" => Ok(Self::BLACK_BOTH),
            // 0 1 0 0
            b"Q" => Ok(Self::WHITE_LONG),
            // 0 1 0 1
            b"Qq" => Ok(Self::WHITE_LONG | Self::BLACK_LONG),
            // 0 1 1 0
            b"Qk" => Ok(Self::WHITE_LONG | Self::BLACK_SHORT),
            // 0 1 1 1
            b"Qkq" => Ok(Self::WHITE_LONG | Self::BLACK_BOTH),
            // 1 0 0 0
            b"K" => Ok(Self::WHITE_SHORT),
            // 1 0 0 1
            b"Kq" => Ok(Self::WHITE_SHORT | Self::BLACK_LONG),
            // 1 0 1 0
            b"Kk" => Ok(Self::WHITE_SHORT | Self::BLACK_SHORT),
            // 1 0 1 1
            b"Kkq" => Ok(Self::WHITE_SHORT | Self::BLACK_BOTH),
            // 1 1 0 0
            b"KQ" => Ok(Self::WHITE_BOTH),
            // 1 1 0 1
            b"KQq" => Ok(Self::WHITE_BOTH | Self::BLACK_LONG),
            // 1 1 1 0
            b"KQk" => Ok(Self::WHITE_BOTH | Self::BLACK_SHORT),
            // 1 1 1 1
            b"KQkq" => Ok(Self::ALL),
            _ => bail!("unknown castle rights: {input}"),
        }
    }
}

impl fmt::Display for CastleRights {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if *self == Self::NONE {
            return f.write_char('-');
        }
        if *self & Self::WHITE_SHORT != Self::NONE {
            f.write_char('K')?;
        }
        if *self & Self::WHITE_LONG != Self::NONE {
            f.write_char('Q')?;
        }
        if *self & Self::BLACK_SHORT != Self::NONE {
            f.write_char('k')?;
        }
        if *self & Self::BLACK_LONG != Self::NONE {
            f.write_char('q')?;
        }
        Ok(())
    }
}

/// A pawn can be promoted to a queen, rook, bishop or a knight.
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd)]
pub(crate) enum Promotion {
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
}

impl From<char> for Promotion {
    fn from(c: char) -> Self {
        match c {
            'n' => Self::Knight,
            'b' => Self::Bishop,
            'r' => Self::Rook,
            'q' => Self::Queen,
            _ => unreachable!("unknown promotion piece, has to be in 'kbrq': {c}"),
        }
    }
}

/// Directions on the board from a perspective of White player.
///
/// Traditionally those are North (Up), West (Left), East (Right), South (Down)
/// and their combinations. However, using cardinal directions is confusing,
/// hence they are replaced by relative directions.
#[derive(Copy, Clone, Debug)]
pub enum Direction {
    /// Also known as North.
    Up,
    /// Also known as South.
    Down,
}

impl Direction {
    pub(super) const fn opposite(self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::mem::{size_of, size_of_val};

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn rank() {
        assert_eq!(
            ('1'..='9')
                .filter_map(|ch| Rank::try_from(ch).ok())
                .collect::<Vec<Rank>>(),
            vec![
                Rank::Rank1,
                Rank::Rank2,
                Rank::Rank3,
                Rank::Rank4,
                Rank::Rank5,
                Rank::Rank6,
                Rank::Rank7,
                Rank::Rank8,
            ]
        );
        assert_eq!(
            ('1'..='9')
                .filter_map(|idx| Rank::try_from(idx).ok())
                .collect::<Vec<Rank>>(),
            vec![
                Rank::Rank1,
                Rank::Rank2,
                Rank::Rank3,
                Rank::Rank4,
                Rank::Rank5,
                Rank::Rank6,
                Rank::Rank7,
                Rank::Rank8,
            ]
        );
    }

    #[test]
    #[should_panic(expected = "rank should be within '1'..='8', got '9'")]
    fn rank_from_incorrect_char() {
        let _ = Rank::try_from('9').unwrap();
    }

    #[test]
    #[should_panic(expected = "rank should be within '1'..='8', got '0'")]
    fn rank_from_incorrect_char_zero() {
        let _ = Rank::try_from('0').unwrap();
    }

    #[test]
    #[should_panic(expected = "rank should be within 0..BOARD_WIDTH, got 8")]
    fn rank_from_incorrect_index() {
        let _ = Rank::try_from(BOARD_WIDTH).unwrap();
    }

    #[test]
    fn file() {
        assert_eq!(
            ('a'..='i')
                .filter_map(|ch| File::try_from(ch).ok())
                .collect::<Vec<File>>(),
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
        assert_eq!(
            (0..=BOARD_WIDTH)
                .filter_map(|idx| File::try_from(idx).ok())
                .collect::<Vec<File>>(),
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
    #[should_panic(expected = "file should be within 'a'..='h', got 'i'")]
    fn file_from_incorrect_char() {
        let _ = File::try_from('i').unwrap();
    }

    #[test]
    #[should_panic(expected = "file should be within 0..BOARD_WIDTH, got 8")]
    fn file_from_incorrect_index() {
        let _ = File::try_from(BOARD_WIDTH).unwrap();
    }

    #[test]
    fn square() {
        let squares: Vec<_> = [
            0u8,
            BOARD_SIZE - 1,
            BOARD_WIDTH - 1,
            BOARD_WIDTH,
            BOARD_WIDTH * 2 + 5,
            BOARD_SIZE,
        ]
        .iter()
        .filter_map(|square| Square::try_from(*square).ok())
        .collect();
        assert_eq!(
            squares,
            vec![Square::A1, Square::H8, Square::H1, Square::A2, Square::F3,]
        );
        let squares: Vec<_> = [
            (File::B, Rank::Rank3),
            (File::F, Rank::Rank5),
            (File::H, Rank::Rank8),
            (File::E, Rank::Rank4),
        ]
        .iter()
        .map(|(file, rank)| Square::new(*file, *rank))
        .collect();
        assert_eq!(
            squares,
            vec![Square::B3, Square::F5, Square::H8, Square::E4]
        );

        assert_eq!(Square::try_from(4u8).unwrap(), Square::E1);
        assert_eq!(Square::try_from(4i8).unwrap(), Square::E1);
    }

    #[test]
    #[should_panic(expected = "square index should be in 0..BOARD_SIZE, got 64")]
    fn square_from_incorrect_index() {
        let _ = Square::try_from(BOARD_SIZE).unwrap();
    }

    #[test]
    fn primitive_size() {
        assert_eq!(size_of::<Square>(), 1);
        // Primitives will have small size thanks to the niche optimizations:
        // https://rust-lang.github.io/unsafe-code-guidelines/layout/enums.html#layout-of-a-data-carrying-enums-without-a-repr-annotation
        assert_eq!(size_of::<PieceKind>(), 1);
        assert_eq!(size_of::<Option<PieceKind>>(), 1);
        let square_to_pieces: [Option<PieceKind>; BOARD_SIZE as usize] =
            [None; BOARD_SIZE as usize];
        assert_eq!(size_of_val(&square_to_pieces), BOARD_SIZE as usize);
    }

    #[test]
    fn square_shift() {
        assert_eq!(Square::A2.shift(Direction::Up), Some(Square::A3));
        assert_eq!(Square::B5.shift(Direction::Down), Some(Square::B4));
        assert_eq!(Square::C1.shift(Direction::Down), None);
        assert_eq!(Square::G8.shift(Direction::Up), None);
    }

    #[test]
    fn correct_moves_from_uci() {
        assert_eq!(
            Move::from_uci("e2e4").unwrap(),
            Move::new(Square::E2, Square::E4, None)
        );
        assert_eq!(
            Move::from_uci("e7e8").unwrap(),
            Move::new(Square::E7, Square::E8, None)
        );
        assert_eq!(
            Move::from_uci("e7e8q").unwrap(),
            Move::new(Square::E7, Square::E8, Some(Promotion::Queen))
        );
    }
}
