//! Provides fully-specified [Chess Position] implementation: stores information
//! about the board and tracks the state of castling, 50-move rule draw, etc.
//!
//! [Chess Position]: https://www.chessprogramming.org/Chess_Position
use std::fmt;
use std::num::NonZeroU16;

use itertools::Itertools;

use crate::chess::bitboard::{Bitboard, Board};
use crate::chess::core::{
    CastlingRights,
    File,
    ParseError,
    Piece,
    Player,
    Rank,
    Square,
    BOARD_WIDTH,
};

/// State of the chess game: board, half-move counters and castling rights,
/// etc. It has 1:1 relationship with [Forsyth-Edwards Notation] (FEN).
///
/// Position can be created by
///
/// - [`Position::from_fen()`] to parse position from FEN.
/// - [`Position::try_from()`] will clean up the input (trim newlines and
///   whitespace) and attempt to parse in either FEN or a version of [Extended
///   Position Description] (EPD). The EPD format Pabi accepts does not support
///   [Operations]: even though it is an important part of EPD, in practice it
///   is rarely needed. The EPD support exists for compatibility with some
///   databases which provide trimmed FEN lines (all FEN parts except Halfmove
///   Clock and Fullmove Counter). Parsing these positions is important to
///   utilize that data, so [`Position::try_from()`] provides an interface to
///   this format.
///
/// [Forsyth-Edwards Notation]: https://www.chessprogramming.org/Forsyth-Edwards_Notation
/// [Extended Position Description]: https://www.chessprogramming.org/Extended_Position_Description
/// [Operations]: https://www.chessprogramming.org/Extended_Position_Description#Operations
// Note: This stores information about pieces in BitboardSets. Stockfish and
// many other engines maintain both piece- and square-centric representations at
// once to speed up querying the piece on a specific square.
// TODO: Check if this yields any benefits. It's likely to be very important for
// hashing and some square-centric algorithms.
pub struct Position {
    board: Board,
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
    // TODO: Delegate from_fen to the fields (board, etc).
    // TODO: Conceal this from public API? Only leave the From<&str> entrypoint that
    // would sanitize the string and provide meaningful defaults.
    pub fn from_fen(fen: &str) -> Result<Self, ParseError> {
        let fen = fen.trim();
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
        let mut result = Self::empty();
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
                // The increment is a small number: casting to u8 will not truncate.
                #[allow(clippy::cast_possible_truncation)]
                if let Some(increment) = symbol.to_digit(10) {
                    file += increment as u8;
                    continue;
                }
                match Piece::try_from(symbol) {
                    Ok(piece) => {
                        let owner = match piece.owner {
                            Player::White => &mut result.board.white_pieces,
                            Player::Black => &mut result.board.black_pieces,
                        };
                        let square = Square::new(File::from(file), Rank::from(rank_id));
                        *owner.bitboard_for(piece.kind) |= Bitboard::from(square);
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
        // - Pawns can't be on ranks 1 and 8.
        // - There can't be more than 8 pawns per side.
        // - There should be exactly two kings.
        // - If there is a check, there should only be one.
        // - En passant squares can not be ni ranks other than 3 and 6.
        // Theoretically, this can still not be "correct" position of the
        // classical chess. However, this is probably sufficient for Pabi's
        // needs. This should probably be a debug assertion.
        Ok(result)
    }

    /// Dumps board in Forsyth-Edwards Notation.
    pub fn fen(&self) -> String {
        format!(
            "{} {} {} {} {} {}",
            self.board.fen(),
            self.side_to_move.fen(),
            CastlingRights::fen(self.white_castling, self.black_castling),
            match self.en_passant_square {
                Some(square) => format!("{}", square),
                None => "-".to_string(),
            },
            self.halfmove_clock,
            self.fullmove_counter
        )
    }

    fn patch_epd(epd: &str) -> String {
        epd.trim().to_string() + " 0 1"
    }

    fn material_imbalance(&self) -> String {
        todo!();
    }

    // IMPORTANT: This is slow because of the bitboard representation and
    // shouldn't be used in performance-critical scenarios. Caching square to Piece
    // would solve this problem.
    fn at(&self, square: Square) -> Option<Piece> {
        self.board.at(square)
    }
}

impl TryFrom<&str> for Position {
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
            6 => Self::from_fen(input),
            4 => Self::from_fen(&Position::patch_epd(input)),
            _ => Err(ParseError(
                "Board representation should be either FEN (6 parts) or EPD body (4 parts)".into(),
            )),
        }
    }
}

impl fmt::Debug for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.board.render_ascii())?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::io::Read;
    use std::{fs, path};

    use super::Position;

    fn check_correct_fen(fen: &str) {
        let position = Position::from_fen(fen);
        assert!(position.is_ok());
        let position = position.unwrap();
        assert_eq!(position.fen(), fen);
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

    fn read_compressed_book(book: path::PathBuf) -> String {
        let file = fs::File::open(&book).unwrap();
        let mut archive = zip::read::ZipArchive::new(file).unwrap();
        assert_eq!(archive.len(), 1);
        let mut contents = String::new();
        let status = archive.by_index(0).unwrap().read_to_string(&mut contents);
        assert!(status.is_ok());
        contents
    }

    fn validate_book(book_name: path::PathBuf) {
        for fen in read_compressed_book(book_name).lines() {
            let fen = fen.trim();
            match fen.split_ascii_whitespace().count() {
                6 => check_correct_fen(fen),
                // Patch EPD to get a full FEN out of it.
                4 => check_correct_fen(&Position::patch_epd(fen)),
                _ => unreachable!(),
            }
        }
    }

    // TODO: Maybe move this to tests/ and re-use for move generator and evaluator.
    // TODO: This is expensive (>60 sec). Make this test optional but make sure it's
    // enabled in the CI.
    #[test]
    fn stockfish_books() {
        let mut path = path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("data/books/");
        for file in fs::read_dir(path).unwrap() {
            let path = file.unwrap().path();
            dbg!(&path);
            if path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .ends_with(".epd.zip")
            {
                dbg!("Validating {}", &path);
                validate_book(path);
            }
        }
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
            " \n epd  rnbqkb1r/ppp1pp1p/5np1/3p4/3P1B2/5N2/PPP1PPPP/RN1QKB1R  w  KQkq   -  \n"
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
    }
}
