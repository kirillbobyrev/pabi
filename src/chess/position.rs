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

use crate::chess::attacks::{get_bishop_attacks, get_rook_attacks, KNIGHT_ATTACKS};
use crate::chess::bitboard::{Bitboard, BitboardSet, Board};
use crate::chess::core::{CastleRights, Move, Piece, Player, Rank, Square, BOARD_WIDTH};

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

    fn opponent_pieces(&self) -> &BitboardSet {
        match self.side_to_move.opponent() {
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
        let mut moves = vec![];
        // TODO: Completely delegate to Position since it already has side_to_move?
        // Cache squares occupied by each player.
        let our_pieces = self.our_pieces();
        let opponent_pieces = self.opponent_pieces();
        let non_capture_mask = our_pieces.all() | opponent_pieces.king;
        for from in our_pieces.knights.iter() {
            let targets = KNIGHT_ATTACKS[from as usize] - non_capture_mask;
            for to in targets.iter() {
                moves.push(Move::new(from, to, None));
            }
        }
        for from in our_pieces.bishops.iter() {
            let targets = get_bishop_attacks(from, our_pieces.all() | opponent_pieces.all())
                - non_capture_mask;
            for to in targets.iter() {
                moves.push(Move::new(from, to, None));
            }
        }
        for from in our_pieces.rooks.iter() {
            let targets =
                get_rook_attacks(from, our_pieces.all() | opponent_pieces.all()) - non_capture_mask;
            for to in targets.iter() {
                moves.push(Move::new(from, to, None));
            }
        }
        // TODO: Check afterstate? Our king should not be checked.
        // TODO: Castling.
        // TODO: Pawn moves + en passant.
        moves
    }

    /// Applies the move in a position
    // TODO: Make an unchecked version of it? With the move coming from the move
    // gen, it's probably much faster to make moves without checking whether
    // they're OK or not. But that would require some benchmarks.
    pub fn make_move(&mut self) -> anyhow::Result<()> {
        todo!();
    }

    // TODO: Add checks for board validity? Not sure if it'd be useful, but here are
    // the heuristics:
    // - If there is a check, there should only be one.
    // Theoretically, this can still not be "correct" position of the classical
    // chess. However, this is probably sufficient for Pabi's needs. This should
    // probably be a debug assertion.
    // Idea for inspiration: https://github.com/sfleischman105/Pleco/blob/b825cecc258ad25cba65919208727994f38a06fb/pleco/src/board/fen.rs#L105-L189
    pub fn check_validity(&self) -> anyhow::Result<()> {
        // TODO: The following patterns look repetitive; maybe refactor the
        // common structure even though it's quite short?
        if self.board.white_pieces.king.count_ones() != 1 {
            bail!(
                "incorrect number of white kings: expected 1, got {}",
                self.board.white_pieces.king.count_ones()
            );
        }
        if self.board.black_pieces.king.count_ones() != 1 {
            bail!(
                "incorrect number of black kings: expected 1, got {}",
                self.board.black_pieces.king.count_ones()
            );
        }
        if self.board.white_pieces.pawns.count_ones() > 8 {
            bail!(
                "incorrect number of white pawns: expected <= 8, got {}",
                self.board.white_pieces.pawns.count_ones()
            );
        }
        if self.board.black_pieces.pawns.count_ones() > 8 {
            bail!(
                "incorrect number of black pawns: expected <= 8, got {}",
                self.board.black_pieces.pawns.count_ones()
            );
        }
        if ((self.board.white_pieces.pawns & self.board.black_pieces.pawns)
            & (Bitboard::rank_mask(Rank::One) | Bitboard::rank_mask(Rank::Eight)))
        .count_ones()
            != 0
        {
            bail!("pawns can not be on the first and last rank");
        }
        if let Some(en_passant_square) = self.en_passant_square {
            let expected_rank = match self.side_to_move {
                Player::White => Rank::Six,
                Player::Black => Rank::Three,
            };
            if en_passant_square.rank() != expected_rank {
                bail!(
                    "incorrect en passant rank: expected {expected_rank}, got {}",
                    en_passant_square.rank()
                );
            }
            // TODO: Moreover, en passant square should be behind a doubly
            // pushed pawn.
        }
        // TODO: The rest of the checks.
        Ok(())
    }
}

impl TryFrom<&str> for Position {
    type Error = anyhow::Error;

    /// Parses board from Forsyth-Edwards Notation. It will also accept trimmed
    /// FEN (EPD with 4 parts).
    ///
    /// FEN ::=
    ///       Piece Placement
    ///   ' ' Side to move
    ///   ' ' Castling ability
    ///   ' ' En passant target square
    ///   ' ' Halfmove clock
    ///   ' ' Fullmove counter
    ///
    /// The last two parts (together) are optional and will default to "0 1".
    // TODO: Test specific errors.
    fn try_from(input: &str) -> anyhow::Result<Self> {
        let mut input = input;
        for prefix in ["fen", "epd"] {
            if let Some(stripped) = input.strip_prefix(prefix) {
                input = stripped;
                break;
            }
        }

        let mut parts = input.trim().split_ascii_whitespace();
        // Parse Piece Placement.
        let mut result = Self::empty();
        let pieces_placement = match parts.next() {
            Some(placement) => placement,
            None => bail!("incorrect FEN: missing pieces placement"),
        };
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
        result.side_to_move = match parts.next() {
            Some(value) => value.try_into()?,
            None => bail!("incorrect FEN: missing side to move"),
        };
        result.castling = match parts.next() {
            Some(value) => value.try_into()?,
            None => bail!("incorrect FEN: missing castling rights"),
        };
        result.en_passant_square = match parts.next() {
            Some("-") => None,
            Some(value) => Some(value.try_into()?),
            None => bail!("incorrect FEN: missing en passant square"),
        };
        result.halfmove_clock = match parts.next() {
            Some(value) => match value.parse::<u8>() {
                Ok(num) => num,
                Err(e) => {
                    return Err(e).with_context(|| {
                        format!("incorrect FEN: halfmove clock can not be parsed {value}")
                    });
                },
            },
            // This is a correct EPD: exit early.
            None => return Ok(result),
        };
        result.fullmove_counter = match parts.next() {
            Some(value) => match value.parse::<NonZeroU16>() {
                Ok(num) => num,
                Err(e) => {
                    return Err(e).with_context(|| {
                        format!("incorrect FEN: fullmove counter can not be parsed {value}")
                    });
                },
            },
            None => bail!("incorrect FEN: missing halfmove clock"),
        };
        Ok(result)
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
        writeln!(f, "{:?}", &self.board)?;
        writeln!(f, "Player to move: {:?}", &self.side_to_move)?;
        writeln!(f, "Fullmove counter: {:?}", &self.fullmove_counter)?;
        writeln!(f, "En Passant: {:?}", &self.en_passant_square)?;
        // bitflags default fmt::Debug implementation is not very convenient.
        writeln!(f, "Castling rights: {}", &self.castling)?;
        writeln!(f, "FEN: {}", &self.to_string())?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use super::Position;

    fn check_correct_fen(fen: &str) {
        let position = Position::try_from(fen);
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
