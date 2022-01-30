//! Provides fully-specified [Chess Position] implementation: stores information
//! about the board and tracks the state of castling, 50-move rule draw, etc.
//!
//! The core of Move Generator and move making is also implemented here as a way
//! to produce ways of mutating [`Position`].
//!
//! [Chess Position]: https://www.chessprogramming.org/Chess_Position

use std::fmt;
use std::num::NonZeroU16;

use anyhow::{bail, Context};
use itertools::Itertools;

use crate::chess::attacks::KNIGHT_ATTACKS;
use crate::chess::bitboard::{Bitboard, BitboardSet, Board};
use crate::chess::core::{CastleRights, Move, Piece, PieceKind, Player, Rank, Square, BOARD_WIDTH};

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
/// [Forsyth-Edwards Notation]: https://www.chessprogramming.org/Forsyth-Edwards_Notation
/// [Extended Position Description]: https://www.chessprogramming.org/Extended_Position_Description
/// [Operations]: https://www.chessprogramming.org/Extended_Position_Description#Operations
// TODO: This only stores information about pieces in BitboardSets. Stockfish
// and many other engines maintain both piece- and square-centric
// representations at once to speed up querying the piece on a specific square.
// Implement and benchmark this.
// TODO: Make the fields private, expose appropriate assessors.
pub struct Position {
    pub(super) board: Board,
    pub(super) castling: CastleRights,
    pub(super) side_to_move: Player,
    /// [Halfmove Clock][^ply] keeps track of the number of (half-)moves
    /// since the last capture or pawn move and is used to enforce
    /// fifty[^fifty]-move draw rule.
    ///
    ///
    /// [Halfmove Clock]: https://www.chessprogramming.org/Halfmove_Clock
    /// [^ply]: "Half-move" or ["ply"](https://www.chessprogramming.org/Ply) means a move of only
    ///     one side.
    /// [^fifty]: 50 __full__ moves
    pub(super) halfmove_clock: u8,
    pub(super) fullmove_counter: NonZeroU16,
    pub(super) en_passant_square: Option<Square>,
}

impl Position {
    /// Creates the starting position of the standard chess variant.
    ///
    /// ```
    /// use pabi::chess::position::Position;
    ///
    /// let starting_position = Position::starting();
    /// assert_eq!(
    ///     &starting_position.to_string(),
    ///     "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
    /// );
    /// ```
    #[must_use]
    pub fn starting() -> Self {
        Self {
            board: Board::starting(),
            castling: CastleRights::ALL,
            ..Self::empty()
        }
    }

    // Creates an empty board to be filled by parser.
    fn empty() -> Self {
        Self {
            board: Board::empty(),
            castling: CastleRights::NONE,
            side_to_move: Player::White,
            halfmove_clock: 0,
            fullmove_counter: NonZeroU16::new(1).unwrap(),
            en_passant_square: None,
        }
    }

    fn our_pieces(&self) -> &BitboardSet {
        match self.side_to_move {
            Player::White => &self.board.white_pieces,
            Player::Black => &self.board.black_pieces,
        }
    }

    /// Produces a list of legal moves (i.e. the moves that do not leave the
    /// King in check).
    ///
    /// This is a performance and correctness-critical path: every modification
    /// should be benchmarked and carefully tested.
    ///
    /// NOTE: [BMI Instruction Set] (and specifically efficient [PEXT]) is not
    /// widely available on all processors (e.g. the AMD only started providing
    /// an *efficient* PEXT since Ryzen 3). The current implementation will
    /// rely on PEXT for performance because it is the most efficient move
    /// generator technique available.
    ///
    /// [generation]: https://www.chessprogramming.org/Table-driven_Move_Generation
    /// [BMI2 Pext Bitboards]: https://www.chessprogramming.org/BMI2#PEXTBitboards
    /// [BMI Instruction Set]: https://en.wikipedia.org/wiki/X86_Bit_manipulation_instruction_set
    /// [PEXT]: https://en.wikipedia.org/wiki/X86_Bit_manipulation_instruction_set#Parallel_bit_deposit_and_extract
    // TODO: Fall back to Fancy Magic Bitboards if BMI2 is not available for
    // portability? Maybe for now just implement
    // https://www.chessprogramming.org/BMI2#Serial_Implementation#Serial_Implementation2
    // TODO: Look at and compare speed with https://github.com/jordanbray/chess
    // TODO: Also implement divide and use <https://github.com/jniemann66/juddperft> to validate the
    // results.
    // TODO: Another source for comparison:
    // https://github.com/sfleischman105/Pleco/blob/b825cecc258ad25cba65919208727994f38a06fb/pleco/src/board/movegen.rs#L68-L85
    // TODO: Maybe use python-chess testset of perft moves:
    // https://github.com/niklasf/python-chess/blob/master/examples/perft/random.perft
    // TODO: Compare with other engines and perft generators, e.g. Berserk,
    // shakmaty (https://github.com/jordanbray/chess_perft).
    // TODO: Check movegen comparison (https://github.com/Gigantua/Chess_Movegen).
    // TODO: Should this be `Position::generate_moves` instead?
    #[must_use]
    pub fn generate_moves(&self) -> Vec<Move> {
        // TODO: let mut vec = Vec::with_capacity(35); and tweak the specific
        // number. The average branching factor for chess is 35 but we probably
        // have to account for a healthy percentile instead of the average.
        // https://en.wikipedia.org/wiki/Branching_factor
        let result = vec![];
        // TODO: Completely delegate to Position since it already has side_to_move?
        // Cache squares occupied by each player.
        let our_pieces = self.our_pieces();
        for from in our_pieces.all().iter() {
            // TODO: Filter out pins?
            // Only generate moves when the square is non-empty and has the piece of a
            // correct color on it.
            let piece = match self.board.at(from) {
                None => unreachable!(),
                Some(piece) => piece,
            };
            let targets = match piece.kind {
                PieceKind::King => todo!(),
                // Sliding pieces.
                PieceKind::Queen | PieceKind::Rook | PieceKind::Bishop => todo!(),
                PieceKind::Knight => KNIGHT_ATTACKS[from as usize],
                PieceKind::Pawn => todo!(),
            } - our_pieces.all();
            // Loop over the target squares and produce moves for those which are
            // not occupied by our pieces. The empty squares or opponent pieces
            // (captures) are valid.
            for _to in targets.iter() {
                // TODO: Create a move if it's possible.
                todo!();
            }
        }
        // TODO: Check afterstate? Our king should not be checked.
        // TODO: Castling.
        // TODO: Pawn moves + en passant.
        result
    }

    /// Applies the move in a position
    // TODO: Make an unchecked version of it? With the move coming from the move
    // gen, it's probably much faster to make moves without checking whether
    // they're OK or not. But that would require some benchmarks.
    pub fn make_move(&mut self) -> anyhow::Result<()> {
        todo!();
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
    // TODO: Test specific errors.
    fn from_fen(fen: &str) -> anyhow::Result<Self> {
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
            None => bail!(
                "incorrect FEN: expected 6 parts, got: {}",
                fen.split_ascii_whitespace().count()
            ),
        };
        // Parse Piece Placement.
        let mut result = Self::empty();
        let ranks = pieces_placement.split('/');
        let mut rank_id = 8;
        for rank_fen in ranks {
            if rank_id == 0 {
                bail!("incorrect FEN: expected 8 ranks, got {pieces_placement}");
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
                    Err(e) => return Err(e),
                }
                file += 1;
            }
            if file != BOARD_WIDTH {
                bail!("incorrect FEN: rank size should be exactly {BOARD_WIDTH}, got {rank_fen} of length {file}");
            }
        }
        if rank_id != 0 {
            bail!("incorrect FEN: there should be 8 ranks, got {pieces_placement}");
        }
        result.side_to_move = side_to_move.try_into()?;
        result.castling = castling_ability.try_into()?;
        if en_passant_square != "-" {
            result.en_passant_square = Some(en_passant_square.try_into()?);
        }
        result.halfmove_clock = match halfmove_clock.parse::<u8>() {
            Ok(num) => num,
            Err(e) => {
                return Err(e).with_context(|| {
                    format!("incorrect FEN: halfmove clock can not be parsed {halfmove_clock}")
                });
            },
        };
        result.fullmove_counter = match fullmove_counter.parse::<NonZeroU16>() {
            Ok(num) => num,
            Err(e) => {
                return Err(e).with_context(|| {
                    format!("incorrect FEN: fullmove counter can not be parsed {fullmove_counter}")
                });
            },
        };
        Ok(result)
    }

    fn patch_epd(epd: &str) -> String {
        epd.trim().to_string() + " 0 1"
    }

    // TODO: Add checks for board validity? Not sure if it'd be useful, but here are
    // the heuristics:
    // - Pawns can't be on ranks 1 and 8.
    // - There can't be more than 8 pawns per side.
    // - There should be exactly two kings.
    // - If there is a check, there should only be one.
    // - En passant squares can not be in squares with ranks other than 3 and 6.
    // Theoretically, this can still not be "correct" position of the classical
    // chess. However, this is probably sufficient for Pabi's needs. This should
    // probably be a debug assertion.
    // Idea for inspiration: https://github.com/sfleischman105/Pleco/blob/b825cecc258ad25cba65919208727994f38a06fb/pleco/src/board/fen.rs#L105-L189
    fn is_valid() -> anyhow::Result<()> {
        todo!()
    }
}

// TODO: There are many &str <-> String conversions. Memory allocations are
// expensive, it would be better to consume strings and avoid allocations.
impl TryFrom<&str> for Position {
    type Error = anyhow::Error;

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
            4 => Self::from_fen(&Self::patch_epd(input)),
            parts => bail!(
                "incorrect board representation: expected either FEN (6 parts) or EPD body \
                (4 parts), got: {parts}"
            ),
        }
    }
}

impl fmt::Display for Position {
    /// Prints board in Forsyth-Edwards Notation.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ", &self.board)?;
        write!(f, "{} ", &self.side_to_move)?;
        write!(f, "{} ", &self.castling)?;
        match self.en_passant_square {
            Some(square) => write!(f, "{square} "),
            None => write!(f, "- "),
        }?;
        write!(f, "{} ", &self.halfmove_clock)?;
        write!(f, "{}", &self.fullmove_counter)?;
        Ok(())
    }
}

impl fmt::Debug for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}\n", &self.board)?;
        write!(f, "Player to move: {:?}\n", &self.side_to_move)?;
        write!(f, "Fullmove counter: {:?}\n", &self.fullmove_counter)?;
        write!(f, "En Passant: {:?}\n", &self.en_passant_square)?;
        // bitflags default fmt::Debug implementation is not very convenient.
        write!(f, "Castling rights: {}\n", &self.castling)?;
        write!(f, "FEN: {}\n", &self.to_string())?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use super::Position;

    fn check_correct_fen(fen: &str) {
        let position = Position::from_fen(fen);
        assert!(position.is_ok(), "input: {fen}");
        let position = position.unwrap();
        assert_eq!(position.to_string(), fen, "input: {fen}");
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
