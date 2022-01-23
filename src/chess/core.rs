//! Board primitives commonly used within [`crate::chess`].

use std::error::Error;
use std::{fmt, mem};

#[allow(missing_docs)]
pub const BOARD_WIDTH: u8 = 8;
#[allow(missing_docs)]
pub const BOARD_SIZE: u8 = BOARD_WIDTH * BOARD_WIDTH;

/// Represents a column (vertical row) of the chessboard. In chess notation, it
/// is normally represented with a lowercase letter.
// TODO: Re-export in lib.rs for convenience?
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, strum::EnumIter)]
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

// TODO: Here and in Rank: implement From<u8> and see whether/how much faster it
// is than the safe checked version.
impl TryFrom<char> for File {
    type Error = ParseError;

    fn try_from(file: char) -> Result<Self, Self::Error> {
        match file {
            'a'..='h' => Ok(unsafe { mem::transmute(file as u8 - b'a') }),
            _ => Err(ParseError(format!(
                "Unknown file ({file}): needs to be in 'a'..='h'."
            ))),
        }
    }
}

impl TryFrom<u8> for File {
    type Error = ParseError;

    fn try_from(column: u8) -> Result<Self, Self::Error> {
        match column {
            0..=7 => Ok(unsafe { mem::transmute(column) }),
            _ => Err(ParseError(format!(
                "Unknown file ({column}): needs to be in 0..BOARD_WIDTH."
            ))),
        }
    }
}

/// Represents a horizontal row of the chessboard. In chess notation, it is
/// represented with a number. The implementation assumes zero-based values
/// (i.e. rank 1 would be 0).
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, strum::EnumIter)]
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

impl TryFrom<char> for Rank {
    type Error = ParseError;

    fn try_from(rank: char) -> Result<Self, Self::Error> {
        match rank {
            '1'..='8' => Ok(unsafe { mem::transmute(rank as u8 - b'1') }),
            _ => Err(ParseError(format!(
                "Unknown rank ({rank}): needs to be in '1'..='8'."
            ))),
        }
    }
}

impl TryFrom<u8> for Rank {
    type Error = ParseError;

    fn try_from(row: u8) -> Result<Self, Self::Error> {
        match row {
            0..=7 => Ok(unsafe { mem::transmute(row) }),
            _ => Err(ParseError(format!(
                "Unknown rank ({row}): needs to be in 0..BOARD_WIDTH."
            ))),
        }
    }
}

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", *self as u8 + 1)
    }
}

/// Board squares: from left to right, from bottom to the top:
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
/// use std::mem;
///
/// assert_eq!(std::mem::size_of::<Square>(), 1);
/// ```
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, strum::EnumIter)]
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
    pub fn new(file: File, rank: Rank) -> Self {
        unsafe { mem::transmute(file as u8 + (rank as u8) * BOARD_WIDTH) }
    }

    /// Returns file (column) on which the square is located.
    pub fn file(self) -> File {
        unsafe { mem::transmute(self as u8 % BOARD_WIDTH) }
    }

    /// Returns rank (row) on which the square is located.
    pub fn rank(self) -> Rank {
        unsafe { mem::transmute(self as u8 / BOARD_WIDTH) }
    }

    pub(in crate::chess) fn shift(self, direction: Direction) -> Option<Square> {
        // TODO: Maybe extend this to all cases and don't check for candidate < 0. Check
        // if it's faster on the benchmarks.
        match direction {
            Direction::UpLeft | Direction::Right | Direction::DownLeft => {
                if self.file() == File::H {
                    return None;
                }
            },
            Direction::UpRight | Direction::Left | Direction::DownRight => {
                if self.file() == File::A {
                    return None;
                }
            },
            _ => (),
        }
        let shift: i8 = match direction {
            Direction::UpLeft => BOARD_WIDTH as i8 + 1,
            Direction::Up => BOARD_WIDTH as i8,
            Direction::UpRight => BOARD_WIDTH as i8 - 1,
            Direction::Right => 1,
            Direction::Left => -1,
            Direction::DownLeft => -(BOARD_WIDTH as i8 - 1),
            Direction::Down => -(BOARD_WIDTH as i8),
            Direction::DownRight => -(BOARD_WIDTH as i8 + 1),
        };
        // TODO: Should this be TryFrom<i8> instead?
        let candidate = self as i8 + shift;
        if candidate < 0 {
            return None;
        }
        match Square::try_from(candidate as u8) {
            Ok(square) => Some(square),
            Err(_) => None,
        }
    }
}

impl TryFrom<u8> for Square {
    type Error = ParseError;

    /// Creates a square given its position on the board.
    ///
    /// # Errors
    ///
    /// If given square index is outside 0..[`BOARD_SIZE`] range.
    fn try_from(square_index: u8) -> Result<Self, Self::Error> {
        // Exclusive range patterns are not allowed: https://github.com/rust-lang/rust/issues/37854
        const MAX_INDEX: u8 = BOARD_SIZE - 1;
        match square_index {
            0..=MAX_INDEX => Ok(unsafe { mem::transmute(square_index) }),
            _ => Err(ParseError(format!(
                "Unknown square_index ({square_index}): needs to be in 0..BOARD_SIZE."
            ))),
        }
    }
}

impl TryFrom<&str> for Square {
    type Error = ParseError;

    fn try_from(square: &str) -> Result<Self, ParseError> {
        if square.bytes().len() != 2 {
            return Err(ParseError(format!(
                "Unknown square ({square}): should be two-char."
            )));
        }
        let (file, rank) = (
            square.bytes().next().unwrap() as char,
            square.bytes().nth(1).unwrap() as char,
        );
        Ok(Square::new(file.try_into()?, rank.try_into()?))
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.file(), self.rank())
    }
}

/// A standard game of chess is played between two players: White (having the
/// advantage of the first turn) and Black.
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Player {
    White,
    Black,
}

impl TryFrom<&str> for Player {
    type Error = ParseError;

    fn try_from(player: &str) -> Result<Self, Self::Error> {
        match player {
            "w" => Ok(Player::White),
            "b" => Ok(Player::Black),
            _ => Err(ParseError(format!(
                "Unknown player ({player}): should be either 'w' or 'b'."
            ))),
        }
    }
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                Player::White => 'w',
                Player::Black => 'b',
            }
        )
    }
}

/// Standard [chess pieces].
///
/// [chess pieces]: https://en.wikipedia.org/wiki/Chess_piece
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum PieceKind {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

/// Represents a specific piece owned by a player.
pub struct Piece {
    #[allow(missing_docs)]
    pub owner: Player,
    #[allow(missing_docs)]
    pub kind: PieceKind,
}

/// Wraps a message indicating failure in parsing part of a chess position
/// (FEN/EPD).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError(pub String);

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for ParseError {}

impl Piece {
    /// Algebraic notation symbol used in FEN. Uppercase for white, lowercase
    /// for black.
    pub(in crate::chess) fn algebraic_symbol(&self) -> char {
        let result = match &self.kind {
            PieceKind::King => 'k',
            PieceKind::Queen => 'q',
            PieceKind::Rook => 'r',
            PieceKind::Bishop => 'b',
            PieceKind::Knight => 'n',
            PieceKind::Pawn => 'p',
        };
        match &self.owner {
            Player::White => result.to_ascii_uppercase(),
            Player::Black => result,
        }
    }
}

impl TryFrom<char> for Piece {
    type Error = ParseError;

    fn try_from(symbol: char) -> Result<Self, ParseError> {
        match symbol {
            'K' => Ok(Self {
                owner: Player::White,
                kind: PieceKind::King,
            }),
            'Q' => Ok(Self {
                owner: Player::White,
                kind: PieceKind::Queen,
            }),
            'R' => Ok(Self {
                owner: Player::White,
                kind: PieceKind::Rook,
            }),
            'B' => Ok(Self {
                owner: Player::White,
                kind: PieceKind::Bishop,
            }),
            'N' => Ok(Self {
                owner: Player::White,
                kind: PieceKind::Knight,
            }),
            'P' => Ok(Self {
                owner: Player::White,
                kind: PieceKind::Pawn,
            }),
            'k' => Ok(Self {
                owner: Player::Black,
                kind: PieceKind::King,
            }),
            'q' => Ok(Self {
                owner: Player::Black,
                kind: PieceKind::Queen,
            }),
            'r' => Ok(Self {
                owner: Player::Black,
                kind: PieceKind::Rook,
            }),
            'b' => Ok(Self {
                owner: Player::Black,
                kind: PieceKind::Bishop,
            }),
            'n' => Ok(Self {
                owner: Player::Black,
                kind: PieceKind::Knight,
            }),
            'p' => Ok(Self {
                owner: Player::Black,
                kind: PieceKind::Pawn,
            }),
            _ => Err(ParseError(
                "Piece symbols should be within \"KQRBNPkqrbnp\"".into(),
            )),
        }
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.algebraic_symbol())
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum CastlingSide {
    Short,
    Long,
}

/// Directions on the board from a perspective of White player.
///
/// Traditionally those are North (Up), West (Left), East (Right), South (Down)
/// and their combinations. However, using cardinal directions is unnecessarily
/// confusing, hence relative directions are more straightforward to use and
/// argue about.
#[derive(Copy, Clone, Debug)]
#[allow(missing_docs)]
pub(in crate::chess) enum Direction {
    UpLeft,
    Up,
    UpRight,
    Right,
    Left,
    DownLeft,
    Down,
    DownRight,
}

/// Track the ability to [castle] each side (kingside is often referred to as
/// O-O or OO, queenside -- O-O-O or OOO). When the king moves, player loses
/// ability to castle both sides, when the rook moves, player loses ability to
/// castle its corresponding side.
///
/// [castle]: https://www.chessprogramming.org/Castling
// TODO: This is likely to be cleaner using bitflags.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum CastlingRights {
    Neither,
    OnlyShort,
    OnlyLong,
    Both,
}

impl TryFrom<&str> for CastlingRights {
    type Error = ParseError;

    /// Parses [`CastlingRights`] for one player from the FEN format. The input
    /// should be ether lowercase ASCII letters or uppercase ones. The user
    /// is responsible for providing valid input cleaned up from the actual
    /// FEN chunk (can be "-").
    ///
    /// # Errors
    ///
    /// Returns [`ParseError`] if given pattern does not match
    ///
    /// [`CastlingRights`] := [K/k] [Q/q]
    ///
    /// Note that both letters have to be either uppercase or lowercase.
    fn try_from(fen: &str) -> Result<Self, ParseError> {
        if fen.len() > 2 {
            return Err(ParseError(format!(
                "Castling rights should contain up to 2 symbols, got: {fen}."
            )));
        }
        match fen {
            "-" | "" => Ok(Self::Neither),
            "k" | "K" => Ok(Self::OnlyShort),
            "q" | "Q" => Ok(Self::OnlyLong),
            "kq" | "KQ" => Ok(Self::Both),
            _ => return Err(ParseError(format!("Unknown castling rights: {fen}."))),
        }
    }
}

impl CastlingRights {
    /// Print castling rights of both sides in FEN format.
    pub(in crate::chess) fn fen(white: CastlingRights, black: CastlingRights) -> String {
        if white == CastlingRights::Neither && black == CastlingRights::Neither {
            return "-".into();
        }
        let render_rights = |rights: CastlingRights| match rights {
            Self::Neither => "",
            Self::OnlyShort => "k",
            Self::OnlyLong => "q",
            Self::Both => "kq",
        };
        format!(
            "{}{}",
            render_rights(white).to_uppercase(),
            render_rights(black)
        )
    }
}

#[cfg(test)]
mod test {
    use std::mem::{size_of, size_of_val};

    use super::{Direction, File, ParseError, PieceKind, Rank, Square, BOARD_SIZE, BOARD_WIDTH};

    #[test]
    fn rank() {
        let ranks: Vec<_> = ('1'..='9').map(|rank| Rank::try_from(rank)).collect();
        assert_eq!(
            ranks,
            vec![
                Ok(Rank::One),
                Ok(Rank::Two),
                Ok(Rank::Three),
                Ok(Rank::Four),
                Ok(Rank::Five),
                Ok(Rank::Six),
                Ok(Rank::Seven),
                Ok(Rank::Eight),
                Err(ParseError(
                    "Unknown rank (9): needs to be in '1'..='8'.".to_string()
                )),
            ]
        );
        let ranks: Vec<_> = (0..=BOARD_WIDTH).map(|rank| Rank::try_from(rank)).collect();
        assert_eq!(
            ranks,
            vec![
                Ok(Rank::One),
                Ok(Rank::Two),
                Ok(Rank::Three),
                Ok(Rank::Four),
                Ok(Rank::Five),
                Ok(Rank::Six),
                Ok(Rank::Seven),
                Ok(Rank::Eight),
                Err(ParseError(
                    "Unknown rank (8): needs to be in 0..BOARD_WIDTH.".to_string()
                )),
            ]
        );
    }

    #[test]
    fn file() {
        let files: Vec<_> = ('a'..='i').map(|file| File::try_from(file)).collect();
        assert_eq!(
            files,
            vec![
                Ok(File::A),
                Ok(File::B),
                Ok(File::C),
                Ok(File::D),
                Ok(File::E),
                Ok(File::F),
                Ok(File::G),
                Ok(File::H),
                Err(ParseError(
                    "Unknown file (i): needs to be in 'a'..='h'.".to_string()
                ))
            ]
        );
        let files: Vec<_> = (0..=BOARD_WIDTH).map(|file| File::try_from(file)).collect();
        assert_eq!(
            files,
            vec![
                Ok(File::A),
                Ok(File::B),
                Ok(File::C),
                Ok(File::D),
                Ok(File::E),
                Ok(File::F),
                Ok(File::G),
                Ok(File::H),
                Err(ParseError(
                    "Unknown file (8): needs to be in 0..BOARD_WIDTH.".to_string()
                ))
            ]
        );
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
        .map(|square| Square::try_from(*square))
        .collect();
        assert_eq!(
            squares,
            vec![
                Ok(Square::A1),
                Ok(Square::H8),
                Ok(Square::H1),
                Ok(Square::A2),
                Ok(Square::F3),
                Err(ParseError(
                    "Unknown square_index (64): needs to be in 0..BOARD_SIZE.".to_string()
                ))
            ]
        );
        let squares: Vec<_> = [
            (File::B, Rank::Three),
            (File::F, Rank::Five),
            (File::H, Rank::Eight),
            (File::E, Rank::Four),
        ]
        .iter()
        .map(|(file, rank)| {
            Square::new(
                File::try_from(*file).unwrap(),
                Rank::try_from(*rank).unwrap(),
            )
        })
        .collect();
        assert_eq!(
            squares,
            vec![Square::B3, Square::F5, Square::H8, Square::E4]
        );
    }

    #[test]
    fn primitive_size() {
        assert_eq!(size_of::<Square>(), 1);
        // Primitives will have small size thanks to the niche optimizations:
        // https://rust-lang.github.io/unsafe-code-guidelines/layout/enums.html#layout-of-a-data-carrying-enums-without-a-repr-annotation
        assert_eq!(size_of::<PieceKind>(), size_of::<Option<PieceKind>>());
        // This is going to be very useful for square-centric board implementation.
        let square_to_pieces: [Option<PieceKind>; BOARD_SIZE as usize] =
            [None; BOARD_SIZE as usize];
        assert_eq!(size_of_val(&square_to_pieces), BOARD_SIZE as usize);
    }

    #[test]
    fn within_board_shift() {
        // D1.
        let square = Square::E4;
        assert_eq!(square.shift(Direction::Left), Some(Square::D4));
        assert_eq!(square.shift(Direction::Up), Some(Square::E5));
        assert_eq!(square.shift(Direction::UpRight), Some(Square::D5));
        assert_eq!(square.shift(Direction::UpLeft), Some(Square::F5));
        assert_eq!(square.shift(Direction::Right), Some(Square::F4));
        assert_eq!(square.shift(Direction::Down), Some(Square::E3));
        assert_eq!(square.shift(Direction::DownRight), Some(Square::D3));
        assert_eq!(square.shift(Direction::DownLeft), Some(Square::F3));
    }

    #[test]
    fn border_squares_shift() {
        // D1.
        let square = Square::D1;
        assert_eq!(square.shift(Direction::Left), Some(Square::C1));
        assert_eq!(square.shift(Direction::Up), Some(Square::D2));
        assert_eq!(square.shift(Direction::UpRight), Some(Square::C2));
        assert_eq!(square.shift(Direction::UpLeft), Some(Square::E2));
        assert_eq!(square.shift(Direction::Right), Some(Square::E1));
        for direction in [Direction::Down, Direction::DownRight, Direction::DownLeft] {
            assert_eq!(square.shift(direction), None);
        }

        // A2.
        let square = Square::A2;
        assert_eq!(square.shift(Direction::Up), Some(Square::A3));
        assert_eq!(square.shift(Direction::UpLeft), Some(Square::B3));
        assert_eq!(square.shift(Direction::Down), Some(Square::A1));
        assert_eq!(square.shift(Direction::DownLeft), Some(Square::B1));
        assert_eq!(square.shift(Direction::Right), Some(Square::B2));
        for direction in [Direction::Left, Direction::UpRight, Direction::DownRight] {
            assert_eq!(square.shift(direction), None);
        }

        // F8.
        let square = Square::F8;
        assert_eq!(square.shift(Direction::Left), Some(Square::E8));
        assert_eq!(square.shift(Direction::Down), Some(Square::F7));
        assert_eq!(square.shift(Direction::DownRight), Some(Square::E7));
        assert_eq!(square.shift(Direction::DownLeft), Some(Square::G7));
        assert_eq!(square.shift(Direction::Right), Some(Square::G8));
        for direction in [Direction::Up, Direction::UpRight, Direction::UpLeft] {
            assert_eq!(square.shift(direction), None);
        }

        // H6.
        let square = Square::H6;
        assert_eq!(square.shift(Direction::Left), Some(Square::G6));
        assert_eq!(square.shift(Direction::Up), Some(Square::H7));
        assert_eq!(square.shift(Direction::UpRight), Some(Square::G7));
        assert_eq!(square.shift(Direction::Down), Some(Square::H5));
        assert_eq!(square.shift(Direction::DownRight), Some(Square::G5));
        for direction in [Direction::UpLeft, Direction::DownLeft, Direction::Right] {
            assert_eq!(square.shift(direction), None);
        }
    }

    #[test]
    fn corner_squares_shift() {
        // A1.
        let square = Square::A1;
        assert_eq!(square.shift(Direction::Up), Some(Square::A2));
        assert_eq!(square.shift(Direction::UpLeft), Some(Square::B2));
        assert_eq!(square.shift(Direction::Right), Some(Square::B1));
        for direction in [
            Direction::Left,
            Direction::UpRight,
            Direction::Down,
            Direction::DownRight,
            Direction::DownLeft,
        ] {
            assert_eq!(square.shift(direction), None);
        }

        // A8.
        let square = Square::A8;
        assert_eq!(square.shift(Direction::Down), Some(Square::A7));
        assert_eq!(square.shift(Direction::DownLeft), Some(Square::B7));
        assert_eq!(square.shift(Direction::Right), Some(Square::B8));
        for direction in [
            Direction::Left,
            Direction::Up,
            Direction::UpRight,
            Direction::UpLeft,
            Direction::DownRight,
        ] {
            assert_eq!(square.shift(direction), None);
        }

        // H8.
        let square = Square::H8;
        assert_eq!(square.shift(Direction::Left), Some(Square::G8));
        assert_eq!(square.shift(Direction::Down), Some(Square::H7));
        assert_eq!(square.shift(Direction::DownRight), Some(Square::G7));
        for direction in [
            Direction::Up,
            Direction::UpRight,
            Direction::UpLeft,
            Direction::DownLeft,
            Direction::Right,
        ] {
            assert_eq!(square.shift(direction), None);
        }

        // H1.
        let square = Square::H1;
        assert_eq!(square.shift(Direction::Left), Some(Square::G1));
        assert_eq!(square.shift(Direction::Up), Some(Square::H2));
        assert_eq!(square.shift(Direction::UpRight), Some(Square::G2));
        for direction in [
            Direction::UpLeft,
            Direction::Right,
            Direction::DownRight,
            Direction::Down,
            Direction::DownLeft,
        ] {
            assert_eq!(square.shift(direction), None);
        }

        let square = Square::E4;
        assert_eq!(square.shift(Direction::Up), Some(Square::E5));
    }
}
