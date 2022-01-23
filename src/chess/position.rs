//! Provides fully-specified [Chess Position] implementation: stores information
//! about the board and tracks the state of castling, 50-move rule draw, etc.
//!
//! [Chess Position]: https://www.chessprogramming.org/Chess_Position

use std::fmt;
use std::num::NonZeroU16;

use itertools::Itertools;

use crate::chess::bitboard::{Bitboard, Board};
use crate::chess::core::{CastlingRights, ParseError, Piece, Player, Rank, Square, BOARD_WIDTH};

/// State of the chess game: board, half-move counters and castling rights,
/// etc. It has 1:1 relationship with [Forsyth-Edwards Notation] (FEN).
///
/// [`Position::try_from()`] provides a convenient interface for creating a
/// [`Position`]. It will clean up the input (trim newlines and whitespace) and
/// attempt to parse in either FEN or a version of [Extended Position
/// Description] (EPD). The EPD format Pabi accepts does not support
/// [Operations]: even though it is an important part of EPD, in practice it is
/// rarely needed. The EPD support exists for compatibility with some databases
/// which provide trimmed FEN lines (all FEN parts except Halfmove Clock and
/// Fullmove Counter). Parsing these positions is important to utilize that
/// data.
///
/// Similarly, [`Position::to_string()`] will dump position in FEN
/// representation.
///
/// [Forsyth-Edwards Notation]: https://www.chessprogramming.org/Forsyth-Edwards_Notation
/// [Extended Position Description]: https://www.chessprogramming.org/Extended_Position_Description
/// [Operations]: https://www.chessprogramming.org/Extended_Position_Description#Operations
// TODO: This only stores information about pieces in BitboardSets. Stockfish
// and many other engines maintain both piece- and square-centric
// representations at once to speed up querying the piece on a specific square.
// Implement and benchmark this.
// TODO: Add checks for board validity? Not sure if it'd be useful, but here are
// the heuristics:
// - Pawns can't be on ranks 1 and 8.
// - There can't be more than 8 pawns per side.
// - There should be exactly two kings.
// - If there is a check, there should only be one.
// - En passant squares can not be ni ranks other than 3 and 6.
// Theoretically, this can still not be "correct" position of the classical
// chess. However, this is probably sufficient for Pabi's needs. This should
// probably be a debug assertion.
pub struct Position {
    pub(in crate::chess) board: Board,
    pub(in crate::chess) white_castling: CastlingRights,
    pub(in crate::chess) black_castling: CastlingRights,
    pub(in crate::chess) side_to_move: Player,
    /// [Halfmove Clock][^ply] keeps track of the number of (half-)moves
    /// since the last capture or pawn move and is used to enforce
    /// fifty[^fifty]-move draw rule.
    ///
    ///
    /// [Halfmove Clock]: https://www.chessprogramming.org/Halfmove_Clock
    /// [^ply]: "Half-move" or ["ply"](https://www.chessprogramming.org/Ply) means a move of only
    ///     one side.
    /// [^fifty]: 50 __full__ moves
    pub(in crate::chess) halfmove_clock: u8,
    pub(in crate::chess) fullmove_counter: NonZeroU16,
    pub(in crate::chess) en_passant_square: Option<Square>,
}

impl Position {
    // Creates an empty board to be filled by parser.
    fn empty() -> Self {
        Self {
            board: Board::empty(),
            white_castling: CastlingRights::Neither,
            black_castling: CastlingRights::Neither,
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
    // TODO: Delegate from_fen to the fields (board, etc)?
    fn from_fen(fen: &str) -> Result<Self, ParseError> {
        let parts = fen.split_ascii_whitespace();
        let (
            pieces_placement,
            side_to_move,
            castling_ability,
            en_passant_square,
            halfmove_clock,
            fullmove_counter,
        ) = match parts.collect_tuple() {
            Some(t) => t,
            None => return Err(ParseError("FEN should have 6 parts".into())),
        };
        // Parse Piece Placement.
        let mut result = Self::empty();
        let ranks = pieces_placement.split('/');
        let mut rank_id = 8;
        for rank_fen in ranks {
            if rank_id == 0 {
                return Err(ParseError("There should be 8 ranks".into()));
            }
            rank_id -= 1;
            let rank = Rank::try_from(rank_id)?;
            let mut file: u8 = 0;
            for symbol in rank_fen.chars() {
                // The increment is a small number: casting to u8 will not truncate.
                #[allow(clippy::cast_possible_truncation)]
                if let Some(increment) = symbol.to_digit(10) {
                    file += increment as u8;
                    if file >= BOARD_WIDTH {
                        break;
                    }
                    continue;
                }
                match Piece::try_from(symbol) {
                    Ok(piece) => {
                        let owner = match piece.owner {
                            Player::White => &mut result.board.white_pieces,
                            Player::Black => &mut result.board.black_pieces,
                        };
                        let square = Square::new(file.try_into()?, rank);
                        *owner.bitboard_for(piece.kind) |= Bitboard::from(square);
                    },
                    Err(e) => {
                        return Err(ParseError(format!("FEN rank has incorrect symbol: {e}.")))
                    },
                }
                file += 1;
            }
            if file != BOARD_WIDTH {
                return Err(ParseError(format!(
                    "FEN rank {rank_fen} size should be exactly {BOARD_WIDTH}, got: {file}."
                )));
            }
        }
        if rank_id != 0 {
            return Err(ParseError("There should be 8 ranks".into()));
        }
        result.side_to_move = side_to_move.try_into()?;
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
            Err(e) => return Err(ParseError(format!("Halfmove clock parsing error: {e}."))),
        };
        result.fullmove_counter = match fullmove_counter.parse::<NonZeroU16>() {
            Ok(num) => num,
            Err(e) => return Err(ParseError(format!("Fullmove counter parsing error: {e}."))),
        };
        Ok(result)
    }

    fn patch_epd(epd: &str) -> String {
        epd.trim().to_string() + " 0 1"
    }
}

// TODO: There are many &str <-> String conversions. Memory allocations are
// expensive, it would be better to consume strings and avoid allocations.
impl TryFrom<&str> for Position {
    type Error = ParseError;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        let mut input = input;
        for prefix in ["fen", "epd"] {
            if input.starts_with(prefix) {
                input = input.split_at(prefix.len()).1;
                break;
            }
        }
        input = input.trim();
        match input.split_ascii_whitespace().count() {
            6 => Self::from_fen(input),
            4 => Self::from_fen(&Position::patch_epd(input)),
            parts => Err(ParseError(format!(
                "Board representation should be either FEN (6 parts) or EPD body (4 parts), got: \
                 {parts}."
            ))),
        }
    }
}

impl ToString for Position {
    /// Dumps board in Forsyth-Edwards Notation.
    fn to_string(&self) -> String {
        format!(
            "{} {} {} {} {} {}",
            self.board.to_string(),
            self.side_to_move,
            CastlingRights::fen(self.white_castling, self.black_castling),
            match self.en_passant_square {
                Some(square) => format!("{square}"),
                None => "-".to_string(),
            },
            self.halfmove_clock,
            self.fullmove_counter
        )
    }
}

impl fmt::Debug for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.board)
    }
}

#[cfg(test)]
mod test {
    use super::Position;

    fn check_correct_fen(fen: &str) {
        let position = Position::from_fen(fen);
        assert!(position.is_ok());
        let position = position.unwrap();
        assert_eq!(position.to_string(), fen);
    }

    // TODO: Validate the precise contents of the bitboard directly.
    // TODO: Add incorrect ones and validate parsing errors.
    #[test]
    fn correct_fen() {
        check_correct_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        check_correct_fen("2r3r1/p3k3/1p3pp1/1B5p/5P2/2P1p1P1/PP4KP/3R4 w - - 0 34");
        check_correct_fen("rnbqk1nr/p3bppp/1p2p3/2ppP3/3P4/P7/1PP1NPPP/R1BQKBNR w KQkq c6 0 7");
        check_correct_fen(
            "r2qkb1r/1pp1pp1p/p1np1np1/1B6/3PP1b1/2N1BN2/PPP2PPP/R2QK2R w KQkq - 0 7",
        );
        check_correct_fen("r3k3/5p2/2p5/p7/P3r3/2N2n2/1PP2P2/2K2B2 w q - 0 24");
    }

    #[test]
    fn correct_epd() {
        let epd = "rnbqkb1r/pp2pppp/3p1n2/8/3NP3/2N5/PPP2PPP/R1BQKB1R b KQkq -";
        assert!(Position::from_fen(epd).is_err());
        assert!(Position::try_from(epd).is_ok());
    }

    #[test]
    fn clean_board_str() {
        // Prefix with "fen".
        assert!(Position::try_from(
            "fen rn1qkb1r/pp3ppp/2p1pn2/3p1b2/2PP4/5NP1/PP2PPBP/RNBQK2R w KQkq - 0 1"
        )
        .is_ok());
        // Prefix with "epd" and add more spaces.
        assert!(Position::try_from(
            "epd  rnbqkb1r/ppp1pp1p/5np1/3p4/3P1B2/5N2/PPP1PPPP/RN1QKB1R  w  KQkq   -  \n"
        )
        .is_ok());
        // Prefix with "fen" and add spaces.
        assert!(Position::try_from(
            "fen   rn1qkb1r/pp3ppp/2p1pn2/3p1b2/2PP4/5NP1/PP2PPBP/RNBQK2R   w   KQkq -  0 1  "
        )
        .is_ok());
        // No prefix: infer EPD.
        assert!(Position::try_from(
            " rnbqkbnr/pp2pppp/8/3p4/3P4/3B4/PPP2PPP/RNBQK1NR b KQkq -   \n"
        )
        .is_ok());
        // No prefix: infer FEN.
        assert!(Position::try_from(
            " rnbqkbnr/pp2pppp/8/3p4/3P4/3B4/PPP2PPP/RNBQK1NR b KQkq -   0 1"
        )
        .is_ok());
        // No prefix: infer FEN.
        assert!(Position::try_from(
            " rnbqkbnr/pp2pppp/8/3p4/3P4/3B4/PPP2PPP/RNBQK1NR b KQkq -   0 1"
        )
        .is_ok());
        // Whitespaces at the start of "fen"/"epd" are not accepted.
        assert!(Position::try_from(
            " \n epd  rnbqkb1r/ppp1pp1p/5np1/3p4/3P1B2/5N2/PPP1PPPP/RN1QKB1R  w  KQkq   -  \n"
        )
        .is_err());
        // Don't crash on unicode symbols.
        assert!(Position::try_from("8/8/8/8/8/8/8/8 b 88 🔠 🔠 ").is_err());
    }
}
