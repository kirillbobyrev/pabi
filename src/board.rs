//! Piece-centric implementation of the chess board. This is
//! the "back-end" of the chess engine, efficient board representation is
//! crucial for performance.

use std::fmt::{self, Write};
use std::num::NonZeroU16;

use itertools::Itertools;
use strum::IntoEnumIterator;

use crate::bitboard::{Bitboard, BitboardSet};
use crate::core::{CastlingRights, File, ParseError, Piece, Player, Rank, Square, BOARD_WIDTH};

/// State of the chess game: position, half-move counters and castling rights,
/// etc. It can be (de)serialized from/into [Forsyth-Edwards Notation] (FEN).
///
/// Board can be created by
///
/// - [`Board::from_fen()`] to parse position from FEN.
/// - [`Board::try_from()`] will clean up the input (trim newlines and
///   whitespace) and attempt to parse in either FEN or a version of [Extended
///   Position Description] (EPD). The EPD format Pabi accepts is does not
///   support [Operations]. Even though it is a crucial part of EPD, in practice
///   it is rarely needed. The EPD support is simply for compatibility with some
///   databases which provide trimmed FEN lines (all FEN parts except Halfmove
///   Clock and Fullmove Counter). Parsing these positions is important, so
///   [`Board::try_from()`] wraps common parsing operations in order to parse
///   and use these databases for testing, benchmarks and learning.
///
/// [Forsyth-Edwards Notation]: https://www.chessprogramming.org/Forsyth-Edwards_Notation
/// [Extended Position Description]: https://www.chessprogramming.org/Extended_Position_Description
/// [Operations]: https://www.chessprogramming.org/Extended_Position_Description#Operations
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
    /// FEN ::=
    ///       Piece Placement
    ///   ' ' Side to move
    ///   ' ' Castling ability
    ///   ' ' En passant target square
    ///   ' ' Halfmove clock
    ///   ' ' Fullmove counter
    pub fn from_fen(fen: &str) -> Result<Self, ParseError> {
        if !fen.is_ascii() || fen.lines().count() != 1 {
            return Err(ParseError("FEN should be a single ASCII line.".into()));
        }
        let parts = fen.split_ascii_whitespace();
        if parts.clone().count() != 6 {
            return Err(ParseError("FEN should have 6 parts".into()));
        }
        let (
            pieces_placement,
            side_to_move,
            castling_ability,
            en_passant_square,
            halfmove_clock,
            fullmove_counter,
        ) = parts.collect_tuple().unwrap();
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
                    },
                    Err(e) => {
                        return Err(ParseError(format!("FEN rank has incorrect symbol: {}", e)))
                    },
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
            },
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

    fn render_ascii(&self) -> String {
        let mut result = String::new();
        for rank in Rank::iter().rev() {
            for file in File::iter() {
                let ascii_symbol = match self.at(Square::new(File::from(file), Rank::from(rank))) {
                    Some(piece) => piece.algebraic_symbol(),
                    None => '.',
                };
                result.push(ascii_symbol);
                if file != File::H {
                    result.push(' ');
                }
            }
            result.push('\n');
        }
        result
    }

    fn material_imbalance(&self) -> String {
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
    /// Defaults to an empty board (incorrect state) that can be later filled by
    /// the parser.
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
        let mut input = input.trim();
        for prefix in ["fen", "epd"] {
            if input.starts_with(prefix) {
                input = input.split_at(prefix.len()).1.trim();
                break;
            }
        }
        match input.split_ascii_whitespace().count() {
            6 => Board::from_fen(input),
            4 => Board::from_fen((input.to_string() + " 0 1").as_str()),
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
        write!(f, "{}", self.render_ascii())?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::Board;

    // TODO: Validate the precise contents of the bitboard directly.
    // TODO: Add incorrect ones and validate parsing errors.
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

    #[test]
    fn correct_epd() {
        let epd = "rnbqkb1r/pp2pppp/3p1n2/8/3NP3/2N5/PPP2PPP/R1BQKB1R b KQkq -";
        assert!(Board::from_fen(epd).is_err());
        assert!(Board::try_from(epd).is_ok());
    }

    #[test]
    fn clean_board_str() {
        // Prefix with "fen".
        assert!(Board::try_from(
            "fen rn1qkb1r/pp3ppp/2p1pn2/3p1b2/2PP4/5NP1/PP2PPBP/RNBQK2R w KQkq - 0 1"
        )
        .is_ok());
        // Prefix with "epd" and add more spaces.
        assert!(Board::try_from(
            " \n epd  rnbqkb1r/ppp1pp1p/5np1/3p4/3P1B2/5N2/PPP1PPPP/RN1QKB1R  w  KQkq   -  \n"
        )
        .is_ok());
        // Prefix with "fen" and add spaces.
        assert!(Board::try_from(
            "fen   rn1qkb1r/pp3ppp/2p1pn2/3p1b2/2PP4/5NP1/PP2PPBP/RNBQK2R   w   KQkq -  0 1  "
        )
        .is_ok());
        // No prefix at all: infer EPD.
        assert!(
            Board::try_from(" rnbqkbnr/pp2pppp/8/3p4/3P4/3B4/PPP2PPP/RNBQK1NR b KQkq -   \n")
                .is_ok()
        );
        // No prefix at all: infer FEN.
        assert!(
            Board::try_from(" rnbqkbnr/pp2pppp/8/3p4/3P4/3B4/PPP2PPP/RNBQK1NR b KQkq -   0 1")
                .is_ok()
        );
    }
}
