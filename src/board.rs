//! Piece-centric implementation of the chess board. This is
//! the "back-end" of the chess engine, efficient board representation is
//! crucial for performance.

use std::error::Error;
use std::fmt::{self, Write};
use std::mem;
use std::num::NonZeroU16;

use itertools::Itertools;
use strum::IntoEnumIterator;

use crate::bitboard::{Bitboard, BitboardSet};

#[allow(missing_docs)]
pub const BOARD_WIDTH: u8 = 8;
#[allow(missing_docs)]
pub const BOARD_SIZE: u8 = BOARD_WIDTH * BOARD_WIDTH;

/// Represents a column (vertical row) of the chessboard. In chess notation, it
/// is normally represented with a lowercase letter.
// TODO: This File, Rank and Square into types.rs or something similar,
// otherwise there are circular dependencies with bitboard that look awkward.
// TODO: Move all of this to src/core?
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
        write!(f, "{}", ('a' as u8 + *self as u8) as char)
    }
}

impl From<u8> for File {
    /// # Panics
    ///
    /// Input has to be a number within 0..[BOARD_WIDTH] range.
    fn from(file: u8) -> Self {
        assert!(file < BOARD_WIDTH);
        unsafe { mem::transmute(file) }
    }
}

impl TryFrom<char> for File {
    type Error = ParseError;
    fn try_from(file: char) -> Result<Self, Self::Error> {
        match file {
            'a'..='h' => Ok(Self::from(file as u8 - 'a' as u8)),
            _ => Err(ParseError(format!("Unknown file: {}", file))),
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

impl From<u8> for Rank {
    /// # Panics
    ///
    /// Input has to be a number within 0..[BOARD_WIDTH] range.
    fn from(rank: u8) -> Self {
        assert!(rank < BOARD_WIDTH);
        unsafe { mem::transmute(rank) }
    }
}

impl TryFrom<char> for Rank {
    type Error = ParseError;
    fn try_from(rank: char) -> Result<Self, Self::Error> {
        match rank {
            '1'..='8' => Ok(Self::from(rank as u8 - '0' as u8 - 1)),
            _ => Err(ParseError(format!("Unknown rank: {}", rank))),
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
    /// Creates a square given its position on the board.
    ///
    /// # Panics
    ///
    /// Input has to be a number within 0..[BOARD_SIZE] range.
    fn from(square: u8) -> Self {
        assert!(square < BOARD_SIZE);
        unsafe { mem::transmute(square) }
    }
}

impl TryFrom<&str> for Square {
    type Error = ParseError;
    fn try_from(square: &str) -> Result<Self, ParseError> {
        if square.len() != 2 {
            return Err(ParseError("Square should be two-char.".into()));
        }
        let (file, rank) = (
            square.chars().nth(0).unwrap(),
            square.chars().nth(1).unwrap(),
        );
        Ok(Square::new(file.try_into()?, rank.try_into()?))
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

/// Standard [chess pieces].
///
/// [chess pieces]: https://en.wikipedia.org/wiki/Chess_piece
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug)]
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

/// Wraps a message indicating failure in parsing [Piece] or [Board] from FEN.
#[derive(Debug, Clone)]
pub struct ParseError(String);

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for ParseError {}

impl Piece {
    // Algebraic notation symbol used in FEN. Uppercase for white, lowercase for
    // black.
    fn algebraic_symbol(&self) -> char {
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
            'K' => Ok(Piece {
                owner: Player::White,
                kind: PieceKind::King,
            }),
            'Q' => Ok(Piece {
                owner: Player::White,
                kind: PieceKind::Queen,
            }),
            'R' => Ok(Piece {
                owner: Player::White,
                kind: PieceKind::Rook,
            }),
            'B' => Ok(Piece {
                owner: Player::White,
                kind: PieceKind::Bishop,
            }),
            'N' => Ok(Piece {
                owner: Player::White,
                kind: PieceKind::Knight,
            }),
            'P' => Ok(Piece {
                owner: Player::White,
                kind: PieceKind::Pawn,
            }),
            'k' => Ok(Piece {
                owner: Player::Black,
                kind: PieceKind::King,
            }),
            'q' => Ok(Piece {
                owner: Player::Black,
                kind: PieceKind::Queen,
            }),
            'r' => Ok(Piece {
                owner: Player::Black,
                kind: PieceKind::Rook,
            }),
            'b' => Ok(Piece {
                owner: Player::Black,
                kind: PieceKind::Bishop,
            }),
            'n' => Ok(Piece {
                owner: Player::Black,
                kind: PieceKind::Knight,
            }),
            'p' => Ok(Piece {
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

/// Track the ability to [castle] each side (kingside is often referred to as
/// O-O or OO, queenside -- O-O-O or OOO). When the king moves, player loses
/// ability to castle both sides, when the rook moves, player loses ability to
/// castle its corresponding side.
///
/// [castle]: https://www.chessprogramming.org/Castling
// TODO: This is likely to be much better using bitflags.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum CastlingRights {
    Neither,
    OnlyKingside,
    OnlyQueenside,
    Both,
}

impl TryFrom<&str> for CastlingRights {
    type Error = ParseError;

    /// Parses CastlingRights for one player from the FEN format. The input should be ether
    /// lowercase ASCII letters or uppercase ones. The user is responsible for providing valid
    /// input cleaned up from the actual FEN chunk (can be "-").
    ///
    /// # Errors
    ///
    /// Returns [ParseError] if given pattern does not match
    ///
    /// CastlingRights := [K/k] [Q/q]
    ///
    /// Note that both letters have to be either uppercase or lowercase.
    fn try_from(fen: &str) -> Result<Self, ParseError> {
        if fen.len() > 2 {
            return Err(ParseError(format!(
                "Castling rights should contain up to 2 symbols, got: {}",
                fen
            )));
        }
        match fen.to_ascii_lowercase().as_str() {
            "" => Ok(CastlingRights::Neither),
            "k" => Ok(CastlingRights::OnlyKingside),
            "q" => Ok(CastlingRights::OnlyQueenside),
            "kq" => Ok(CastlingRights::Both),
            _ => return Err(ParseError(format!("Unknown castling rights: {}", fen))),
        }
    }
}

/// State of the chess game: position, half-move counters and castling rights,
/// etc. It can be (de)serialized from/into [Forsyth-Edwards Notation] (FEN).
///
/// Board can be created by
///
/// - [`Board::starting()`] for the starting position of the standard chess.
/// - [`Board::from_fen()`] to get the
///
/// [Forsyth-Edwards Notation]: https://www.chessprogramming.org/Forsyth-Edwards_Notation
// Note: This stores information about pieces in BitboardSets. Stockfish and
// many other engines maintain both piece- and square-centric representations at
// once to speed up querying the piece on a specific square.
// TODO: Check if this yields any benefits.
// TODO: At this point I'm not sure if Position and Board should be separated
// (a-la shakmaty), let's wait for the user code to appear and see if it's
// important/can yield performance benefits.
pub struct Board {
    white_pieces: BitboardSet,
    black_pieces: BitboardSet,
    white_castling: CastlingRights,
    black_castling: CastlingRights,
    side_to_move: Player,
    /// [Halfmove Clock][^ply] keeps track of the number of (half-)moves
    /// since the last capture or pawn move and is used to enforce
    /// fifty[^fifty]-move draw rule.
    ///
    ///
    /// [Halfmove Clock]: https://www.chessprogramming.org/Halfmove_Clock
    /// [^ply]: "Half-move" or ["ply"](https://www.chessprogramming.org/Ply) means a move of only
    ///     one side.
    /// [^fifty]: 50 _full_ moves
    halfmove_clock: u8,
    fullmove_counter: NonZeroU16,
    en_passant_square: Option<Square>,
}

impl Board {
    /// Creates a board with the starting position.
    pub fn starting() -> Self {
        Self {
            white_pieces: BitboardSet::new_white(),
            black_pieces: BitboardSet::new_black(),
            white_castling: CastlingRights::Both,
            black_castling: CastlingRights::Both,
            side_to_move: Player::White,
            halfmove_clock: 0,
            fullmove_counter: NonZeroU16::new(1).unwrap(),
            en_passant_square: None,
        }
    }

    /// Parses board from Forsyth-Edwards Notation.
    ///
    /// <FEN> ::=
    ///       <Piece Placement>
    ///   ' ' <Side to move>
    ///   ' ' <Castling ability>
    ///   ' ' <En passant target square>
    ///   ' ' <Halfmove clock>
    ///   ' ' <Fullmove counter>
    pub fn from_fen(fen: &str) -> Result<Self, ParseError> {
        if !fen.is_ascii() || fen.lines().count() != 1 {
            return Err(ParseError("FEN should be a single ASCII line.".into()));
        }
        if fen.matches(' ').count() != 5 {
            return Err(ParseError("FEN should have 6 parts".into()));
        }
        let (
            pieces_placement,
            side_to_move,
            castling_ability,
            en_passant_square,
            halfmove_clock,
            fullmove_counter,
        ) = fen.split(' ').collect_tuple().unwrap();
        // Parse Piece Placement.
        if pieces_placement.matches('/').count() != BOARD_WIDTH as usize - 1 {
            return Err(ParseError(
                "Pieces Placement FEN should have 8 ranks.".into(),
            ));
        }
        let mut result = Self::default();
        let ranks = pieces_placement.split('/');
        for (rank_id, rank_fen) in itertools::zip((0..BOARD_WIDTH).rev(), ranks) {
            let mut file: u8 = 0;
            for symbol in rank_fen.chars() {
                if file >= BOARD_WIDTH {
                    return Err(ParseError(format!(
                        "FEN rank {} is longer than {}.",
                        rank_fen, BOARD_WIDTH
                    )));
                }
                if let Some(increment) = symbol.to_digit(10) {
                    file += increment as u8;
                    continue;
                }
                match Piece::try_from(symbol) {
                    Ok(piece) => {
                        let owner = match piece.owner {
                            Player::White => &mut result.white_pieces,
                            Player::Black => &mut result.black_pieces,
                        };
                        let square = Square::new(File::from(file), Rank::from(rank_id));
                        *owner.bitboard_for(piece.kind.clone()) |= Bitboard::from(square);
                    }
                    Err(e) => {
                        return Err(ParseError(format!("FEN rank has incorrect symbol: {}", e)))
                    }
                }
                file += 1;
            }
            if file != BOARD_WIDTH {
                return Err(ParseError(format!(
                    "FEN rank {} size should be exactly {}.",
                    rank_fen, BOARD_WIDTH
                )));
            }
        }
        match side_to_move {
            "w" => result.side_to_move = Player::White,
            "b" => result.side_to_move = Player::Black,
            _ => {
                return Err(ParseError(format!(
                    "Side to move can be either 'w' or 'b', got: {}.",
                    side_to_move
                )));
            }
        }
        // "-" is no-op (empty board already has cleared castling rights).
        if castling_ability != "-" {
            result.white_castling = castling_ability
                .chars()
                .filter(|x| x.is_uppercase())
                .collect::<String>()
                .as_str()
                .try_into()?;
            result.black_castling = castling_ability
                .chars()
                .filter(|x| x.is_lowercase())
                .collect::<String>()
                .as_str()
                .try_into()?;
        }
        if en_passant_square != "-" {
            result.en_passant_square = Some(en_passant_square.try_into()?);
        }
        result.halfmove_clock = match halfmove_clock.parse::<u8>() {
            Ok(num) => num,
            Err(e) => return Err(ParseError(format!("Halfmove clock parsing error: {}", e))),
        };
        result.fullmove_counter = match fullmove_counter.parse::<NonZeroU16>() {
            Ok(num) => num,
            Err(e) => return Err(ParseError(format!("Fullmove counter parsing error: {}", e))),
        };
        // TODO: Add checks for board validity:
        // - Pawns should not be on the wrong ranks.
        // - There can't be too many pawns.
        // - There should be exactly two kings.
        // - If there is a check, there should only be one.
        Ok(result)
    }

    /// Dumps board in Forsyth-Edwards Notation.
    pub fn fen() -> String {
        todo!();
    }

    fn material_imbalance() {
        todo!();
    }

    // IMPORTANT: This is slow because of the bitboard representation and
    // shouldn't be used in performance-critical scenarios. Caching square to Piece
    // would solve this problem
    fn at(&self, square: Square) -> Option<Piece> {
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
    // Defaults to an empty board (incorrect state) to be filled.
    fn default() -> Self {
        Self {
            white_pieces: BitboardSet::default(),
            black_pieces: BitboardSet::default(),
            white_castling: CastlingRights::Neither,
            black_castling: CastlingRights::Neither,
            ..Self::starting()
        }
    }
}

impl TryFrom<&str> for Board {
    type Error = ParseError;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        let input = input.trim();
        match input.matches(' ').count() {
            5 => Board::from_fen(input),
            3 => Board::from_fen((input.to_string() + " 0 1").as_str()),
            _ => Err(ParseError(
                "Board representation should be either FEN (6 parts) or EPD body (4 parts)".into(),
            )),
        }
    }
}

impl fmt::Debug for Board {
    // TODO: Use formatter.alternate() for pretty-printing colored output:
    // <https://doc.rust-lang.org/std/fmt/struct.Formatter.html#method.alternate>
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for rank in Rank::iter().rev() {
            for file in File::iter() {
                let square = Square::new(File::from(file), Rank::from(rank));
                let square = match self.at(square) {
                    Some(piece) => piece.algebraic_symbol(),
                    None => '.',
                };
                f.write_char(square)?;
                if file != File::H {
                    f.write_char(' ')?;
                }
            }
            f.write_char('\n')?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::{Board, File, Rank, Square, BOARD_SIZE, BOARD_WIDTH};

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
    fn out_of_bounds_rank() {
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
    fn out_of_bounds_file() {
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
    fn out_of_bounds_square() {
        let _ = Square::from(BOARD_SIZE);
    }

    #[test]
    fn correct_fen() {
        assert!(
            Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").is_ok()
        );
        assert!(Board::from_fen("2r3r1/p3k3/1p3pp1/1B5p/5P2/2P1p1P1/PP4KP/3R4 w - - 0 34").is_ok());
        assert!(Board::from_fen(
            "rnbqk1nr/p3bppp/1p2p3/2ppP3/3P4/P7/1PP1NPPP/R1BQKBNR w KQkq c6 0 7"
        )
        .is_ok());
        assert!(Board::from_fen(
            "r2qkb1r/1pp1pp1p/p1np1np1/1B6/3PP1b1/2N1BN2/PPP2PPP/R2QK2R w KQkq - 0 7"
        )
        .is_ok());
    }

    // TODO: Validate the precise contents of the bitboard directly.
    #[test]
    fn correct_epd() {
        let epd = "rnbqkb1r/pp2pppp/3p1n2/8/3NP3/2N5/PPP2PPP/R1BQKB1R b KQkq -";
        assert!(Board::from_fen(epd).is_err());
        assert!(Board::try_from(epd).is_ok());
    }

    #[test]
    fn clean_board_str() {
        let epd = " rnbqkbnr/pp2pppp/8/3p4/3P4/3B4/PPP2PPP/RNBQK1NR b KQkq -   \n";
        assert!(Board::try_from(epd).is_ok());
    }
}
