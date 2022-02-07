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

use crate::chess::attacks;
use crate::chess::bitboard::{Bitboard, Board, Pieces};
use crate::chess::core::{
    CastleRights,
    Move,
    Piece,
    PieceKind,
    Player,
    Promotion,
    Rank,
    Square,
    BOARD_WIDTH,
};

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
    board: Board,
    castling: CastleRights,
    side_to_move: Player,
    /// [Halfmove Clock][^ply] keeps track of the number of (half-)moves
    /// since the last capture or pawn move and is used to enforce
    /// fifty[^fifty]-move draw rule.
    ///
    ///
    /// [Halfmove Clock]: https://www.chessprogramming.org/Halfmove_Clock
    /// [^ply]: "Half-move" or ["ply"](https://www.chessprogramming.org/Ply) means a move of only
    ///     one side.
    /// [^fifty]: 50 __full__ moves
    halfmove_clock: u8,
    fullmove_counter: NonZeroU16,
    en_passant_square: Option<Square>,
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
    #[must_use]
    pub fn empty() -> Self {
        Self {
            board: Board::empty(),
            castling: CastleRights::NONE,
            side_to_move: Player::White,
            halfmove_clock: 0,
            fullmove_counter: NonZeroU16::new(1).unwrap(),
            en_passant_square: None,
        }
    }

    pub(super) fn us(&self) -> Player {
        self.side_to_move
    }

    pub(super) fn they(&self) -> Player {
        self.us().opponent()
    }

    pub(super) fn pieces(&self, player: Player) -> &Pieces {
        self.board.player_pieces(player)
    }

    fn occupancy(&self, player: Player) -> Bitboard {
        self.board.player_pieces(player).all()
    }

    fn occupied_squares(&self) -> Bitboard {
        self.occupancy(self.us()) | self.occupancy(self.they())
    }

    /// Calculates a list of legal moves (i.e. the moves that do not leave our
    /// king in check).
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
    // portability?
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
    // TODO:: Store the moves on the stack instead? It might be faster, see
    // https://github.com/niklasf/shakmaty/blob/e0020c0ab4b5f8601486c17c87b3313476a3cf12/src/movelist.rs
    // TODO: Use monomorphization to generate code for calculating attacks for both sides to reduce
    // branching? https://rustc-dev-guide.rust-lang.org/backend/monomorph.html
    #[must_use]
    pub fn generate_moves(&self) -> Vec<Move> {
        debug_assert!(self.is_legal());
        let attack_info = attacks::AttackInfo::new(self);
        // TODO: The average branching factor for chess is 35 but we probably
        // have to account for a healthy percentile instead of the average.
        // https://en.wikipedia.org/wiki/Branching_factor
        let mut moves = Vec::with_capacity(50);
        // Cache squares occupied by each player.
        // TODO: Try caching more e.g. all()s? Benchmark to confirm that this is an
        // improvement.
        let (our_pieces, opponent_pieces) = (self.pieces(self.us()), self.pieces(self.they()));
        let our_king: Square = our_pieces.king.as_square();
        let non_capture_mask = our_pieces.all();
        let occupied_squares = self.occupied_squares();
        // Moving the king to safety is always correct regardless of the checks.
        for safe_square in attack_info.safe_king_squares.iter() {
            moves.push(Move::new(our_king, safe_square, None));
        }
        // If there are checks, the moves are restricted to resolving them.
        let blocking_ray = match attack_info.checkers.count() {
            0 => Bitboard::empty(),
            // There are two ways of getting out of check:
            //
            // - Moving king to safety (calculated above)
            // - Blocking the checker or capturing it
            //
            // The former is calculated above, the latter is dealt with below.
            1 => {
                let checker: Square = attack_info.checkers.as_square();
                let ray = attacks::ray(checker, our_king);
                if ray.is_empty() {
                    // This means the checker is a knight: capture is the only
                    // way left to resolve this check.
                    attack_info.checkers
                } else {
                    // Checker is a sliding piece: both capturing and blocking
                    // resolves the check.
                    ray
                }
            },
            // Double checks can only be evaded by the king moves to safety: no
            // need to consider other moves.
            2 => return moves,
            _ => unreachable!("more than two pieces can not check the king"),
        };
        // TODO: Maybe iterating manually would be faster.
        for (kind, bitboard) in self.pieces(self.us()).iter() {
            for from in bitboard.iter() {
                let targets = match kind {
                    PieceKind::King => Bitboard::empty(),
                    PieceKind::Queen => attacks::queen_attacks(from, occupied_squares),
                    PieceKind::Rook => attacks::rook_attacks(from, occupied_squares),
                    PieceKind::Bishop => attacks::bishop_attacks(from, occupied_squares),
                    PieceKind::Knight => attacks::knight_attacks(from),
                    PieceKind::Pawn => {
                        attacks::pawn_attacks(from, self.us())
                            & (opponent_pieces.all()
                                | match self.en_passant_square {
                                    Some(square) => Bitboard::from(square),
                                    None => Bitboard::empty(),
                                })
                    },
                } - non_capture_mask;
                for to in targets.iter() {
                    // TODO: This block is repeated several times; abstract it out.
                    if !blocking_ray.is_empty() & !blocking_ray.contains(to) {
                        continue;
                    }
                    if attack_info.pins.contains(from)
                        && (attacks::ray(from, our_king) & attacks::ray(to, our_king)).is_empty()
                    {
                        continue;
                    }
                    dbg!(from, to);
                    match (kind, to.rank()) {
                        (PieceKind::Pawn, Rank::Eight | Rank::One) => {
                            moves.push(Move::new(from, to, Some(Promotion::Queen)));
                            moves.push(Move::new(from, to, Some(Promotion::Rook)));
                            moves.push(Move::new(from, to, Some(Promotion::Bishop)));
                            moves.push(Move::new(from, to, Some(Promotion::Knight)));
                        },
                        _ => moves.push(Move::new(from, to, None)),
                    }
                }
            }
        }
        // Check if capturing en passant resolves the check.
        if let Some(en_passant_square) = self.en_passant_square {
            let en_passant_pawn = en_passant_square
                .shift(self.they().push_direction())
                .unwrap();
            if attack_info.checkers.contains(en_passant_pawn) {
                let candidate_pawns =
                    attacks::pawn_attacks(en_passant_square, self.they()) & our_pieces.pawns;
                for our_pawn in candidate_pawns.iter() {
                    if attack_info.pins.contains(our_pawn) {
                        continue;
                    }
                    moves.push(Move::new(our_pawn, en_passant_square, None));
                }
            }
        }
        // Regular pawn pushes.
        let push_direction = self.us().push_direction();
        let pawn_pushes = our_pieces.pawns.shift(push_direction) - occupied_squares;
        let original_squares = pawn_pushes.shift(push_direction.opposite());
        let add_pawn_moves = |moves: &mut Vec<Move>, from, to: Square| {
            // TODO: This is probably better with self.side_to_move.opponent().backrank()
            // but might be slower.
            match to.rank() {
                Rank::Eight | Rank::One => {
                    moves.push(Move::new(from, to, Some(Promotion::Queen)));
                    moves.push(Move::new(from, to, Some(Promotion::Rook)));
                    moves.push(Move::new(from, to, Some(Promotion::Bishop)));
                    moves.push(Move::new(from, to, Some(Promotion::Knight)));
                },
                _ => moves.push(Move::new(from, to, None)),
            }
        };
        for (from, to) in itertools::zip(original_squares.iter(), pawn_pushes.iter()) {
            if !blocking_ray.is_empty() & !blocking_ray.contains(to) {
                continue;
            }
            if attack_info.pins.contains(from)
                && (attacks::ray(from, our_king) & attacks::ray(to, our_king)).is_empty()
            {
                continue;
            }
            add_pawn_moves(&mut moves, from, to);
        }
        // Double pawn pushes.
        // TODO: Come up with a better name for it.
        let third_rank = Rank::pawns_starting(self.us()).mask().shift(push_direction);
        let double_pushes = (pawn_pushes & third_rank).shift(push_direction) - occupied_squares;
        let original_squares = double_pushes
            .shift(push_direction.opposite())
            .shift(push_direction.opposite());
        // Double pawn pushes are never promoting.
        for (from, to) in itertools::zip(original_squares.iter(), double_pushes.iter()) {
            if !blocking_ray.is_empty() & !blocking_ray.contains(to) {
                continue;
            }
            if attack_info.pins.contains(from)
                && (attacks::ray(from, our_king) & attacks::ray(to, our_king)).is_empty()
            {
                continue;
            }
            moves.push(Move::new(from, to, None));
        }
        // TODO: Castling.
        // TODO: In FCR we should check if the rook is pinned or not.
        if attack_info.checkers.is_empty() {
            match self.us() {
                Player::White => {
                    if self.castling.contains(CastleRights::WHITE_SHORT)
                        && (attack_info.attacks & attacks::WHITE_SHORT_CASTLE_KING_WALK).is_empty()
                        && (occupied_squares
                            & (attacks::WHITE_SHORT_CASTLE_KING_WALK
                                | attacks::WHITE_SHORT_CASTLE_ROOK_WALK))
                            .is_empty()
                    {
                        moves.push(Move::new(Square::E1, Square::G1, None));
                    }
                    if self.castling.contains(CastleRights::WHITE_LONG)
                        && (attack_info.attacks & attacks::WHITE_LONG_CASTLE_KING_WALK).is_empty()
                        && (occupied_squares
                            & (attacks::WHITE_LONG_CASTLE_KING_WALK
                                | attacks::WHITE_LONG_CASTLE_ROOK_WALK))
                            .is_empty()
                    {
                        moves.push(Move::new(Square::E1, Square::C1, None));
                    }
                },
                Player::Black => {
                    if self.castling.contains(CastleRights::BLACK_SHORT)
                        && (attack_info.attacks & attacks::BLACK_SHORT_CASTLE_KING_WALK).is_empty()
                        && (occupied_squares
                            & (attacks::BLACK_SHORT_CASTLE_KING_WALK
                                | attacks::BLACK_SHORT_CASTLE_ROOK_WALK))
                            .is_empty()
                    {
                        moves.push(Move::new(Square::E8, Square::G8, None));
                    }
                    if self.castling.contains(CastleRights::BLACK_LONG)
                        && (attack_info.attacks & attacks::BLACK_LONG_CASTLE_KING_WALK).is_empty()
                        && (occupied_squares
                            & (attacks::BLACK_LONG_CASTLE_KING_WALK
                                | attacks::BLACK_LONG_CASTLE_ROOK_WALK))
                            .is_empty()
                    {
                        moves.push(Move::new(Square::E8, Square::C8, None));
                    }
                },
            }
        }
        moves
    }

    /// Applies the move in a position
    // TODO: Make an checked version of it? With the move coming from the UCI
    // it's best to check if it's valid or not.
    pub fn make_move(&mut self, next_move: Move) {
        let (our_pieces, _opponent_pieces) = match self.us() {
            Player::White => (&mut self.board.white_pieces, &mut self.board.black_pieces),
            Player::Black => (&mut self.board.black_pieces, &mut self.board.white_pieces),
        };
        if our_pieces.king.contains(next_move.to) {
            // Check if the move is castling.
            let backrank = Rank::backrank(self.us()).mask();
            if backrank.contains(next_move.from) && backrank.contains(next_move.to) {}
            //
        }
    }

    // TODO: If there is a check, there should only be one delivered by the
    // non-moving side. Theoretically, this can still not be "correct" position
    // of the classical chess. However, this is probably sufficient for Pabi's
    // needs. This should probably be a debug assertion.
    // Idea for inspiration: https://github.com/sfleischman105/Pleco/blob/b825cecc258ad25cba65919208727994f38a06fb/pleco/src/board/fen.rs#L105-L189
    #[must_use]
    pub fn is_legal(&self) -> bool {
        // TODO: The following patterns look repetitive; maybe refactor the
        // common structure even though it's quite short?
        if self.board.white_pieces.king.count() != 1 {
            return false;
        }
        if self.board.black_pieces.king.count() != 1 {
            return false;
        }
        if self.board.white_pieces.pawns.count() > 8 {
            return false;
        }
        if self.board.black_pieces.pawns.count() > 8 {
            return false;
        }
        if ((self.board.white_pieces.pawns | self.board.black_pieces.pawns)
            & (Rank::One.mask() | Rank::Eight.mask()))
        .count()
            != 0
        {
            return false;
        }
        let attacks = attacks::AttackInfo::new(self);
        // Can't have more than two checks.
        if attacks.checkers.count() > 2 {
            return false;
        }
        if let Some(en_passant_square) = self.en_passant_square {
            let expected_rank = match self.side_to_move {
                Player::White => Rank::Six,
                Player::Black => Rank::Three,
            };
            if en_passant_square.rank() != expected_rank {
                return false;
            }
            // A pawn that was just pushed by our opponent should be in front of
            // en_passant_square.
            let pushed_pawn = en_passant_square
                .shift(self.they().push_direction())
                .expect("we already checked for correct rank");
            if !self.pieces(self.they()).pawns.contains(pushed_pawn) {
                return false;
            }
            // If en-passant was played and there's a check, doubly pushed pawn
            // should be the only checker or it should be a discovery.
            let king = self.pieces(self.us()).king.as_square();
            if attacks.checkers.has_any() {
                if attacks.checkers.count() > 1 {
                    return false;
                }
                // The check wasn't delivered by pushed pawn.
                if attacks.checkers != Bitboard::from(pushed_pawn) {
                    let checker = attacks.checkers.as_square();
                    let original_square =
                        en_passant_square.shift(self.us().push_direction()).unwrap();
                    if !(attacks::ray(checker, king).contains(original_square)) {
                        return false;
                    }
                }
            }
            // Doubly pushed pawn can not block a diagonal check.
            for attacker in
                (self.pieces(self.they()).queens | self.pieces(self.they()).bishops).iter()
            {
                let xray = attacks::bishop_ray(attacker, king);
                if (xray & (self.occupied_squares())).count() == 2
                    && xray.contains(attacker)
                    && xray.contains(pushed_pawn)
                {
                    return false;
                }
            }
        }
        // TODO: The rest of the checks.
        true
    }

    #[must_use]
    pub fn has_insufficient_material(&self) -> bool {
        todo!()
    }

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
    /// Technically, that is not a full FEN position, but it is supported
    /// because EPD-style position strings are common in public position books
    /// and datasets where halfmove clock and fullmove counters do not matter.
    /// Supporting these datasets is important but distinguishing between full
    /// and trimmed FEN strings is not.
    ///
    /// NOTE: This expects properly-formatted inputs: no extra symbols or
    /// additional whitespace. Use [`Position::try_from`] for cleaning up the
    /// input if it is coming from untrusted source and is likely to contain
    /// extra symbols.
    pub fn from_fen(input: &str) -> anyhow::Result<Self> {
        let mut parts = input.split(' ');
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
                if file > BOARD_WIDTH {
                    bail!("file exceeded {BOARD_WIDTH}");
                }
                match symbol {
                    '0' => bail!("increment can not be 0"),
                    '1'..='9' => {
                        file += symbol as u8 - b'0';
                        continue;
                    },
                    _ => (),
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
            Some(value) => {
                // TODO: Here and below: parse manually just by getting through
                // ASCII digits since we're already checking them.
                if !value.bytes().all(|c| c.is_ascii_digit()) {
                    bail!("halfmove clock can not contain anything other than digits");
                }
                match value.parse::<u8>() {
                    Ok(num) => num,
                    Err(e) => {
                        return Err(e).with_context(|| {
                            format!("incorrect FEN: halfmove clock can not be parsed {value}")
                        });
                    },
                }
            },
            // This is a correct EPD: exit early.
            None => return Ok(result),
        };
        result.fullmove_counter = match parts.next() {
            Some(value) => {
                if !value.bytes().all(|c| c.is_ascii_digit()) {
                    bail!("fullmove counter clock can not contain anything other than digits");
                }
                match value.parse::<NonZeroU16>() {
                    Ok(num) => num,
                    Err(e) => {
                        return Err(e).with_context(|| {
                            format!("incorrect FEN: fullmove counter can not be parsed {value}")
                        });
                    },
                }
            },
            None => bail!("incorrect FEN: missing halfmove clock"),
        };
        match parts.next() {
            None => Ok(result),
            Some(_) => bail!("trailing symbols are not allowed in FEN"),
        }
    }
}

// TODO: Take plain bytes through from_ascii: that's more principled and may be
// faster.
impl TryFrom<&str> for Position {
    type Error = anyhow::Error;

    // TODO: Docs.
    fn try_from(input: &str) -> anyhow::Result<Self> {
        let input = input.trim();
        for prefix in ["fen ", "epd "] {
            if let Some(stripped) = input.strip_prefix(prefix) {
                return Self::from_fen(stripped);
            }
        }
        Self::from_fen(input)
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
        // bitflags' default fmt::Debug implementation is not very convenient:
        // dump FEN instead.
        writeln!(f, "Castling rights: {}", &self.castling)?;
        writeln!(f, "FEN: {}", &self.to_string())?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use itertools::Itertools;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::chess::core::Move;

    fn setup(fen: &str) -> Position {
        let position = Position::try_from(fen);
        assert!(position.is_ok(), "input: {fen}");
        let position = position.unwrap();
        assert_eq!(position.to_string(), fen);
        assert!(position.is_legal(), "{}", position.to_string());
        position
    }

    // TODO: Validate the precise contents of the bitboard directly.
    // TODO: Add incorrect ones and validate parsing errors.
    #[test]
    #[allow(unused_results)]
    fn correct_fen() {
        setup("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        setup("2r3r1/p3k3/1p3pp1/1B5p/5P2/2P1p1P1/PP4KP/3R4 w - - 0 34");
        setup("rnbqk1nr/p3bppp/1p2p3/2ppP3/3P4/P7/1PP1NPPP/R1BQKBNR w KQkq c6 0 7");
        setup("r2qkb1r/1pp1pp1p/p1np1np1/1B6/3PP1b1/2N1BN2/PPP2PPP/R2QK2R w KQkq - 0 7");
        setup("r3k3/5p2/2p5/p7/P3r3/2N2n2/1PP2P2/2K2B2 w q - 0 24");
        setup("r1b1qrk1/ppp2pbp/n2p1np1/4p1B1/2PPP3/2NB1N1P/PP3PP1/R2QK2R w KQ e6 0 9");
        setup("8/8/8/8/2P5/3k4/8/KB6 b - c3 0 1");
        setup("rnbq1rk1/pp4pp/1b1ppn2/2p2p2/2PP4/1P2PN2/PB2BPPP/RN1Q1RK1 w - c6 0 9");
    }

    #[test]
    fn correct_epd() {
        let epd = "rnbqkb1r/pp2pppp/3p1n2/8/3NP3/2N5/PPP2PPP/R1BQKB1R b KQkq -";
        assert!(Position::try_from(epd).is_ok());
    }

    #[test]
    fn no_crash() {
        assert!(Position::try_from("3k2p1N/82/8/8/7B/6K1/3R4/8 b - - 0 1").is_err());
        assert!(
            Position::try_from("3kn3/R2p1N2/8/8/70000000000000000B/6K1/3R4/8 b - - 0 1").is_err()
        );
        assert!(Position::try_from("3kn3/R4N2/8/8/7B/6K1/3R4/8 b - - 0 48 b - - 0 4/8 b").is_err());
        assert!(Position::try_from("\tfen3kn3/R2p1N2/8/8/7B/6K1/3R4/8 b - - 0 23").is_err());
        assert!(Position::try_from("fen3kn3/R2p1N2/8/8/7B/6K1/3R4/8 b - - 0 23").is_err());
        assert!(Position::try_from("3kn3/R4N2/8/8/7B/6K1/3r4/8 b - - +8 1").is_err());
    }

    #[test]
    fn clean_board_str() {
        // Prefix with "fen".
        assert!(Position::try_from(
            "fen rn1qkb1r/pp3ppp/2p1pn2/3p1b2/2PP4/5NP1/PP2PPBP/RNBQK2R w KQkq - 0 1"
        )
        .is_ok());
        // Prefix with "epd".
        assert!(Position::try_from(
            "epd rnbqkb1r/ppp1pp1p/5np1/3p4/3P1B2/5N2/PPP1PPPP/RN1QKB1R w KQkq -"
        )
        .is_ok());
        // No prefix: infer EPD.
        assert!(
            Position::try_from("rnbqkbnr/pp2pppp/8/3p4/3P4/3B4/PPP2PPP/RNBQK1NR b KQkq -").is_ok()
        );
        // No prefix: infer FEN.
        assert!(
            Position::try_from("rnbqkbnr/pp2pppp/8/3p4/3P4/3B4/PPP2PPP/RNBQK1NR b KQkq - 0 1")
                .is_ok()
        );
        // Don't crash on unicode symbols.
        assert!(Position::try_from("8/8/8/8/8/8/8/8 b 88 🔠 🔠 ").is_err());
        // Whitespaces at the start/end of the input are not accepted in from_fen but
        // will be cleaned up by try_from.
        assert!(Position::try_from(
            "rnbqkb1r/ppp1pp1p/5np1/3p4/3P1B2/5N2/PPP1PPPP/RN1QKB1R w KQkq -\n"
        )
        .is_ok());
        assert!(Position::try_from(
            "\n epd rnbqkb1r/ppp1pp1p/5np1/3p4/3P1B2/5N2/PPP1PPPP/RN1QKB1R w KQkq -"
        )
        .is_ok());
        assert!(Position::from_fen(
            "\n epd rnbqkb1r/ppp1pp1p/5np1/3p4/3P1B2/5N2/PPP1PPPP/RN1QKB1R w KQkq -\n"
        )
        .is_err());
    }

    fn get_moves(position: &Position) -> Vec<String> {
        position
            .generate_moves()
            .iter()
            .map(Move::to_string)
            .sorted()
            .collect::<Vec<_>>()
    }

    fn sorted_moves(moves: &[&str]) -> Vec<String> {
        moves
            .iter()
            .map(|m| (*m).to_string())
            .sorted()
            .collect::<Vec<_>>()
    }

    #[test]
    fn starting_moves() {
        assert_eq!(
            get_moves(&Position::starting()),
            sorted_moves(&[
                "a2a3", "a2a4", "b1a3", "b1c3", "b2b3", "b2b4", "c2c3", "c2c4", "d2d3", "d2d4",
                "e2e3", "e2e4", "f2f3", "f2f4", "g1f3", "g1h3", "g2g3", "g2g4", "h2h3", "h2h4"
            ])
        );
    }

    #[test]
    fn basic_moves() {
        assert_eq!(
            get_moves(&setup("2n4k/1PP5/6K1/3Pp1Q1/3N4/3P4/P3R3/8 w - e6 0 1")),
            sorted_moves(&[
                "a2a3", "a2a4", "d5d6", "d5e6", "b7b8q", "b7b8r", "b7b8b", "b7b8n", "b7c8q",
                "b7c8r", "b7c8b", "b7c8n", "e2e1", "e2e3", "e2e4", "e2e5", "e2b2", "e2c2", "e2d2",
                "e2f2", "e2g2", "e2h2", "d4b3", "d4c2", "d4f3", "d4b5", "d4c6", "d4e6", "d4f5",
                "g5c1", "g5d2", "g5e3", "g5f4", "g5g4", "g5g3", "g5g2", "g5g1", "g5h4", "g5e5",
                "g5f5", "g5h5", "g5h6", "g5f6", "g5e7", "g5d8", "g6f5", "g6h5", "g6f6", "g6h6",
                "g6f7",
            ])
        );
    }

    #[test]
    fn double_check_evasions() {
        assert_eq!(
            get_moves(&setup("3kn3/R2p1N2/8/8/7B/6K1/3R4/8 b - - 0 1")),
            sorted_moves(&["d8c8"])
        );
        assert_eq!(
            get_moves(&setup("8/5Nk1/7p/4Bp2/3q4/8/8/5KR1 b - - 0 1")),
            sorted_moves(&["g7f8", "g7f7", "g7h7"])
        );
        assert_eq!(
            get_moves(&setup("8/5Pk1/7p/4Bp2/3q4/8/8/5KR1 b - - 0 1")),
            sorted_moves(&["g7f8", "g7f7", "g7h7"])
        );
    }

    #[test]
    fn check_evasions() {
        assert_eq!(
            get_moves(&setup("3kn3/R2p4/8/6B1/8/6K1/3R4/8 b - - 0 1")),
            sorted_moves(&["e8f6", "d8c8"])
        );
        assert_eq!(
            get_moves(&setup("2R5/8/6k1/8/8/8/PPn5/KR6 w - - 0 1")),
            sorted_moves(&["c8c2"])
        );
    }

    #[test]
    fn pins() {
        // The pawn is pinned but can capture en passant.
        assert_eq!(
            get_moves(&setup("6qk/8/8/3Pp3/8/8/K7/8 w - e6 0 1")),
            sorted_moves(&["a2a1", "a2a3", "a2b1", "a2b2", "a2b3", "d5e6"])
        );
        // The pawn is pinned but there is no en passant: it can't move.
        assert_eq!(
            get_moves(&setup("6qk/8/8/3Pp3/8/8/K7/8 w - - 0 1")),
            sorted_moves(&["a2a1", "a2a3", "a2b1", "a2b2", "a2b3"])
        );
        // The pawn is pinned and can't move.
        assert_eq!(
            get_moves(&setup("k7/1p6/8/8/8/8/8/4K2B b - - 0 1")),
            sorted_moves(&["a8a7", "a8b8"])
        );
    }

    // Artifacts from the fuzzer.
    #[test]
    fn moves_in_other_positions() {
        assert_eq!(
            get_moves(&setup(
                "2r3r1/3p3k/1p3pp1/1B5P/5P2/2P1pqP1/PP4KP/3R4 w - - 0 34"
            )),
            sorted_moves(&["g2g1", "g2f3", "g2h3"])
        );
        assert_eq!(
            get_moves(&setup(
                "2r3r1/3p3k/1p3pp1/1B5P/5p2/2P1p1P1/PP4KP/3R4 w - - 0 34"
            )),
            sorted_moves(&[
                "a2a3", "a2a4", "b2b3", "b2b4", "c3c4", "b5a4", "b5a6", "b5c6", "b5d7", "b5c4",
                "b5d3", "b5e2", "b5f1", "g3g4", "h2h3", "h2h4", "h5h6", "h5g6", "g2f3", "g2f1",
                "g2g1", "g2h3", "g2h1", "d1a1", "d1b1", "d1c1", "d1e1", "d1f1", "d1g1", "d1h1",
                "d1d2", "d1d3", "d1d4", "d1d5", "d1d6", "d1d7", "g3f4",
            ])
        );
        assert_eq!(
            get_moves(&setup(
                "2r3r1/3p3k/1p3pp1/1B5p/5P2/2P2pP1/PP4KP/3R4 w - - 0 34"
            )),
            sorted_moves(&["g2f1", "g2f2", "g2f3", "g2g1", "g2h1", "g2h3"])
        );
        assert_eq!(
            get_moves(&setup(
                "2r3r1/P3k3/pp3p2/1B5p/5P2/2P3pP/PP4KP/3R4 w - - 0 1"
            )),
            sorted_moves(&[
                "a2a3", "a2a4", "a7a8b", "a7a8n", "a7a8q", "a7a8r", "b2b3", "b2b4", "b5a4", "b5a6",
                "b5c4", "b5c6", "b5d3", "b5d7", "b5e2", "b5e8", "b5f1", "c3c4", "d1a1", "d1b1",
                "d1c1", "d1d2", "d1d3", "d1d4", "d1d5", "d1d6", "d1d7", "d1d8", "d1e1", "d1f1",
                "d1g1", "d1h1", "f4f5", "g2f1", "g2f3", "g2g1", "g2h1", "h2g3", "h3h4",
            ])
        );
        assert_eq!(
            get_moves(&setup(
                "2r3r1/p3k3/pp3p2/1B5p/5P2/2pqp1P1/PPK4P/3R4 w - - 0 34"
            )),
            sorted_moves(&["b5d3", "c2b3", "c2c1", "c2d3", "d1d3"])
        );
        assert_eq!(
            get_moves(&setup(
                "2r3r1/p3k3/pp3p2/1B5p/5P2/2P1p1P1/PP4Kr/3R4 w - - 0 1"
            )),
            sorted_moves(&["g2f1", "g2f3", "g2g1", "g2h2"])
        );
        assert_eq!(
            get_moves(&setup("r3k3/r7/8/5pP1/5QKN/8/8/6RR w - f6 0 1")),
            sorted_moves(&["f4f5", "h4f5", "g4f5", "g4f3", "g4g3", "g4h3", "g5f6", "g4h5"])
        );
        assert_eq!(
            get_moves(&setup("4k1r1/8/8/4PpP1/6K1/8/8/8 w - f6 0 1")),
            sorted_moves(&["g4f4", "g4f3", "g4f5", "g4g3", "g4h3", "g4h4", "g4h5", "e5f6"])
        );
    }

    #[test]
    fn castle() {
        // Can castle both sides.
        assert_eq!(
            get_moves(&setup("r3k2r/8/8/8/8/8/6N1/4K3 b kq - 0 1")),
            sorted_moves(&[
                "a8a7", "a8a6", "a8a5", "a8a4", "a8a3", "a8a2", "a8a1", "a8b8", "a8c8", "a8d8",
                "h8f8", "h8g8", "h8h7", "h8h6", "h8h5", "h8h4", "h8h3", "h8h2", "h8h1", "e8e7",
                "e8d8", "e8d7", "e8f8", "e8f7", "e8c8", "e8g8"
            ])
        );
        // Castling short blocked by a check.
        assert_eq!(
            get_moves(&setup("r3k2r/8/8/8/8/8/6R1/4K3 b kq - 0 1")),
            sorted_moves(&[
                "a8a7", "a8a6", "a8a5", "a8a4", "a8a3", "a8a2", "a8a1", "a8b8", "a8c8", "a8d8",
                "h8f8", "h8g8", "h8h7", "h8h6", "h8h5", "h8h4", "h8h3", "h8h2", "h8h1", "e8e7",
                "e8d8", "e8d7", "e8f8", "e8f7", "e8c8"
            ])
        );
        // Castling short blocked by our piece, castling long is not available.
        assert_eq!(
            get_moves(&setup("r3k2r/8/8/8/8/8/6R1/4K3 b k - 0 1")),
            sorted_moves(&[
                "a8a7", "a8a6", "a8a5", "a8a4", "a8a3", "a8a2", "a8a1", "a8b8", "a8c8", "a8d8",
                "h8f8", "h8g8", "h8h7", "h8h6", "h8h5", "h8h4", "h8h3", "h8h2", "h8h1", "e8e7",
                "e8d8", "e8d7", "e8f8", "e8f7"
            ])
        );
        // Castling long is not blocked: the attacked square is not the one king will
        // walk through.
        assert_eq!(
            get_moves(&setup("r3k2r/8/8/8/8/8/1R6/4K3 b q - 0 1")),
            sorted_moves(&[
                "a8a7", "a8a6", "a8a5", "a8a4", "a8a3", "a8a2", "a8a1", "a8b8", "a8c8", "a8d8",
                "h8f8", "h8g8", "h8h7", "h8h6", "h8h5", "h8h4", "h8h3", "h8h2", "h8h1", "e8e7",
                "e8d8", "e8d7", "e8f8", "e8f7", "e8c8"
            ])
        );
        // Castling long is blocked by an attack and the king is cut off.
        assert_eq!(
            get_moves(&setup("r3k2r/8/8/8/8/8/3R4/4K3 b kq - 0 1")),
            sorted_moves(&[
                "a8a7", "a8a6", "a8a5", "a8a4", "a8a3", "a8a2", "a8a1", "a8b8", "a8c8", "a8d8",
                "h8f8", "h8g8", "h8h7", "h8h6", "h8h5", "h8h4", "h8h3", "h8h2", "h8h1", "e8e7",
                "e8f8", "e8f7", "e8g8"
            ])
        );
    }

    #[test]
    fn chess_programming_wiki_perft_positions() {
        // Positions from https://www.chessprogramming.org/Perft_Results with
        // depth=1.
        // Position 1 is the starting position: handled in detail before.
        // Position 2.
        assert_eq!(
            get_moves(&setup(
                "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"
            ))
            .len(),
            48
        );
        // Position 3.
        assert_eq!(
            get_moves(&setup("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1")).len(),
            14,
        );
        // Position 4.
        assert_eq!(
            get_moves(&setup(
                "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1"
            ))
            .len(),
            6
        );
        // Mirrored.
        assert_eq!(
            get_moves(&setup(
                "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1"
            ))
            .len(),
            6
        );
        // Position 5.
        assert_eq!(
            get_moves(&setup(
                "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8"
            ))
            .len(),
            44
        );
        // Position 6
        assert_eq!(
            get_moves(&setup(
                "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10"
            ))
            .len(),
            46
        );
        // "kiwipete"
        assert_eq!(
            get_moves(&setup(
                "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"
            ))
            .len(),
            48
        );
    }

    fn illegal_position(input: &str) {
        let position = Position::try_from(input).unwrap();
        assert!(!position.is_legal());
    }

    // TODO: Check precise error messages.
    #[test]
    fn illegal_positions() {
        // TODO: Should we check for legality in the parser and reject incorrect
        // positions?
        // The check was not delivered by the doubly pushed pawn.
        illegal_position("rnbqk1nr/bb3p1p/1q2r3/2pPp3/3P4/7P/1PP1NpPP/R1BQKBNR w KQkq c6 0 1");
        // Three checks.
        illegal_position("2r3r1/P3k3/prp5/1B5p/5P2/2Q1n2p/PP4KP/3R4 w - - 0 34");
        // Doubly pushed pawn can not block a diagonal check.
        illegal_position("q6k/8/8/3pP3/8/8/8/7K w - d6 0 1");
        // No white kings.
        illegal_position("3k4/8/8/8/8/8/8/8 w - - 0 1");
        // No white kings.
        illegal_position("3k4/8/8/8/8/8/8/8 w - - 0 1");
        // No black kings.
        illegal_position("8/8/8/8/8/8/8/3K4 w - - 0 1");
        // Too many kings.
        illegal_position("1kkk4/8/8/8/8/8/8/1KKK4 w - - 0 1");
        // Too many white pawns.
        illegal_position("rnbqkbnr/pppppppp/8/8/8/P7/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        // Too many black pawns.
        illegal_position("rnbqkbnr/pppppppp/p7/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        // Pawns on backranks.
        illegal_position("3kr3/8/8/8/8/5Q2/8/1KP5 w - - 0 1");
        // En passant square not behind a pushed pawn.
        illegal_position("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq d3 0 1");
        // Wrong en passant rank.
        illegal_position("rnbqkbnr/pppppppp/8/4P3/8/8/PPPP1PPP/RNBQKBNR b KQkq e4 0 1");
        // En passant can't result in double check.
        illegal_position("r2qkbnr/ppp3Np/8/4Q3/4P3/8/PP4PP/RNB1KB1R w KQkq e3 0 1");
    }
}
