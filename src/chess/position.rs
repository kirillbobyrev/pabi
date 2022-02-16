//! Provides fully-specified [Chess Position] implementation: stores
//! information about the board and tracks the state of castling, 50-move rule
//! draw, etc.
//!
//! The core of Move Generator and move making is also implemented here as a way
//! to produce ways of mutating [`Position`].
//!
//! [Chess Position]: https://www.chessprogramming.org/Chess_Position

use std::fmt;

use anyhow::{bail, Context};

use crate::chess::attacks;
use crate::chess::bitboard::{Bitboard, Board, Pieces};
use crate::chess::core::{
    CastleRights,
    File,
    Move,
    MoveList,
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
// TODO: Make the fields private, expose appropriate assessors.
// TODO: Store Zobrist hash, possibly other info.
#[derive(Clone)]
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
    fullmove_counter: u16,
    en_passant_square: Option<Square>,
}

impl Position {
    /// Creates the starting position of the standard chess.
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

    fn empty() -> Self {
        Self {
            board: Board::empty(),
            castling: CastleRights::NONE,
            side_to_move: Player::White,
            halfmove_clock: 0,
            fullmove_counter: 1,
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

    /// Parses board from Forsyth-Edwards Notation and checks its correctness.
    /// The parser will accept trimmed full FEN and trimmed FEN (4 first parts).
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
    /// Correctness check employs a small set of simple heuristics to check if
    /// the position can be analyzed by the engine and will reject the most
    /// obvious incorrect positions (e.g. missing kings, pawns on the wrong
    /// ranks, problems with en passant square). The only public way of creating
    /// a [`Position`] is by parsing it from string, so this acts as a filter
    /// for positions that won't cause undefined behavior or crashes. It's
    /// important that positions that are known to be dubious are filtered out.
    ///
    /// NOTE: This expects properly-formatted inputs: no extra symbols or
    /// additional whitespace. Use [`Position::try_from`] for cleaning up the
    /// input if it is coming from untrusted source and is likely to contain
    /// extra symbols.
    // was not legal.
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
            None => {
                return match validate(&result) {
                    Ok(_) => Ok(result),
                    Err(e) => Err(e.context("illegal position")),
                }
            },
        };
        result.fullmove_counter = match parts.next() {
            Some(value) => {
                if !value.bytes().all(|c| c.is_ascii_digit()) {
                    bail!("fullmove counter clock can not contain anything other than digits");
                }
                match value.parse::<u16>() {
                    Ok(0) => {
                        bail!("fullmove counter can not be 0")
                    },
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
            None => match validate(&result) {
                Ok(_) => Ok(result),
                Err(e) => Err(e.context("illegal position")),
            },
            Some(_) => bail!("trailing symbols are not allowed in FEN"),
        }
    }

    /// Returns a string representation of the position in FEN format.
    #[must_use]
    pub fn fen(&self) -> String {
        self.to_string()
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
    // TODO: Another source for comparison:
    // https://github.com/sfleischman105/Pleco/blob/b825cecc258ad25cba65919208727994f38a06fb/pleco/src/board/movegen.rs#L68-L85
    // TODO: Maybe use python-chess testset of perft moves:
    // https://github.com/niklasf/python-chess/blob/master/examples/perft/random.perft
    // TODO: Compare with other engines and perft generators
    // (https://github.com/jniemann66/juddperft).
    // TODO: Check movegen comparison (https://github.com/Gigantua/Chess_Movegen).
    // TODO: Use monomorphization to generate code for calculating attacks for both sides to reduce
    // branching? https://rustc-dev-guide.rust-lang.org/backend/monomorph.html
    // TODO: Split into subroutines so that it's easier to tune performance.
    #[must_use]
    pub fn generate_moves(&self) -> MoveList {
        // debug_assert!(validate(&self).is_ok(), "{}", self.fen());
        let attack_info = attacks::AttackInfo::new(self);
        // TODO: The average branching factor for chess is 35 but we probably
        // have to account for a healthy percentile instead of the average.
        // https://en.wikipedia.org/wiki/Branching_factor
        let mut moves = MoveList::new();
        // Cache squares occupied by each player.
        // TODO: Try caching more e.g. all()s? Benchmark to confirm that this is an
        // improvement.
        let (our_pieces, their_pieces) = (self.pieces(self.us()), self.pieces(self.they()));
        let our_king: Square = our_pieces.king.as_square();
        let non_capture_mask = our_pieces.all();
        let occupied_squares = self.occupied_squares();
        // Moving the king to safety is always correct regardless of the checks.
        for safe_square in attack_info.safe_king_squares.iter() {
            unsafe {
                moves.push_unchecked(Move::new(our_king, safe_square, None));
            }
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
                    PieceKind::Pawn => attacks::pawn_attacks(from, self.us()) & their_pieces.all(),
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
                    match (kind, to.rank()) {
                        (PieceKind::Pawn, Rank::Eight | Rank::One) => unsafe {
                            moves.push_unchecked(Move::new(from, to, Some(Promotion::Queen)));
                            moves.push_unchecked(Move::new(from, to, Some(Promotion::Rook)));
                            moves.push_unchecked(Move::new(from, to, Some(Promotion::Bishop)));
                            moves.push_unchecked(Move::new(from, to, Some(Promotion::Knight)));
                        },
                        _ => unsafe { moves.push_unchecked(Move::new(from, to, None)) },
                    }
                }
            }
        }
        // Generate en passant moves.
        if let Some(en_passant_square) = self.en_passant_square {
            let en_passant_pawn = en_passant_square
                .shift(self.they().push_direction())
                .unwrap();
            // Check if capturing en passant resolves the check.
            let candidate_pawns =
                attacks::pawn_attacks(en_passant_square, self.they()) & our_pieces.pawns;
            if attack_info.checkers.contains(en_passant_pawn) {
                for our_pawn in candidate_pawns.iter() {
                    if attack_info.pins.contains(our_pawn) {
                        continue;
                    }
                    unsafe {
                        moves.push_unchecked(Move::new(our_pawn, en_passant_square, None));
                    }
                }
            } else {
                // Check if capturing en passant does not create a discovered check.
                for our_pawn in candidate_pawns.iter() {
                    let mut occupancy_after_capture = occupied_squares;
                    occupancy_after_capture.clear(our_pawn);
                    occupancy_after_capture.clear(en_passant_pawn);
                    occupancy_after_capture.extend(en_passant_square);
                    if (attacks::queen_attacks(our_king, occupancy_after_capture)
                        & their_pieces.queens)
                        .is_empty()
                        && (attacks::rook_attacks(our_king, occupancy_after_capture)
                            & their_pieces.rooks)
                            .is_empty()
                        && (attacks::bishop_attacks(our_king, occupancy_after_capture)
                            & their_pieces.bishops)
                            .is_empty()
                    {
                        unsafe {
                            moves.push_unchecked(Move::new(our_pawn, en_passant_square, None));
                        }
                    }
                }
            }
        }
        // Regular pawn pushes.
        let push_direction = self.us().push_direction();
        let pawn_pushes = our_pieces.pawns.shift(push_direction) - occupied_squares;
        let original_squares = pawn_pushes.shift(push_direction.opposite());
        let add_pawn_moves = |moves: &mut MoveList, from, to: Square| {
            // TODO: This is probably better with self.side_to_move.opponent().backrank()
            // but might be slower.
            match to.rank() {
                Rank::Eight | Rank::One => unsafe {
                    moves.push_unchecked(Move::new(from, to, Some(Promotion::Queen)));
                    moves.push_unchecked(Move::new(from, to, Some(Promotion::Rook)));
                    moves.push_unchecked(Move::new(from, to, Some(Promotion::Bishop)));
                    moves.push_unchecked(Move::new(from, to, Some(Promotion::Knight)));
                },
                _ => unsafe { moves.push_unchecked(Move::new(from, to, None)) },
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
            unsafe {
                moves.push_unchecked(Move::new(from, to, None));
            }
        }
        // TODO: Generalize castling to FCR.
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
                        unsafe {
                            moves.push_unchecked(Move::new(Square::E1, Square::G1, None));
                        }
                    }
                    if self.castling.contains(CastleRights::WHITE_LONG)
                        && (attack_info.attacks & attacks::WHITE_LONG_CASTLE_KING_WALK).is_empty()
                        && (occupied_squares
                            & (attacks::WHITE_LONG_CASTLE_KING_WALK
                                | attacks::WHITE_LONG_CASTLE_ROOK_WALK))
                            .is_empty()
                    {
                        unsafe {
                            moves.push_unchecked(Move::new(Square::E1, Square::C1, None));
                        }
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
                        unsafe {
                            moves.push_unchecked(Move::new(Square::E8, Square::G8, None));
                        }
                    }
                    if self.castling.contains(CastleRights::BLACK_LONG)
                        && (attack_info.attacks & attacks::BLACK_LONG_CASTLE_KING_WALK).is_empty()
                        && (occupied_squares
                            & (attacks::BLACK_LONG_CASTLE_KING_WALK
                                | attacks::BLACK_LONG_CASTLE_ROOK_WALK))
                            .is_empty()
                    {
                        unsafe {
                            moves.push_unchecked(Move::new(Square::E8, Square::C8, None));
                        }
                    }
                },
            }
        }
        moves
    }

    // TODO: Docs: this is the only way to mutate a position....
    // TODO: Make an checked version of it? With the move coming from the UCI
    // it's best to check if it's valid or not.
    // TODO: Is it better to clone and return a new Position? It seems that the
    // most usecases (e.g. for search) would clone the position and then mutate
    // it anyway. This would prevent (im)mutability reference problems.
    pub fn make_move(&mut self, next_move: &Move) {
        // debug_assert!(self.is_legal());
        let (us, they) = (self.us(), self.they());
        let our_backrank = Rank::backrank(us);
        let (our_pieces, their_pieces) = match self.us() {
            Player::White => (&mut self.board.white_pieces, &mut self.board.black_pieces),
            Player::Black => (&mut self.board.black_pieces, &mut self.board.white_pieces),
        };
        let previous_en_passant = self.en_passant_square;
        self.en_passant_square = None;
        if us == Player::Black {
            self.fullmove_counter += 1;
        }
        self.halfmove_clock += 1;
        // NOTE: We reset side_to_move early! To access the moving side, use cached
        // `us`.
        self.side_to_move = us.opponent();
        // Handle captures.
        if our_pieces.rooks.contains(next_move.from) {
            match (us, next_move.from) {
                (Player::White, Square::A1) => self.castling.remove(CastleRights::WHITE_LONG),
                (Player::White, Square::H1) => self.castling.remove(CastleRights::WHITE_SHORT),
                (Player::Black, Square::A8) => self.castling.remove(CastleRights::BLACK_LONG),
                (Player::Black, Square::H8) => self.castling.remove(CastleRights::BLACK_SHORT),
                _ => (),
            }
        }
        if their_pieces.all().contains(next_move.to) {
            // Capturing a piece resets the clock.
            self.halfmove_clock = 0;
            // TODO: Clear castling rights if we're capturing opponent's rook.
            match (they, next_move.to) {
                (Player::White, Square::H1) => self.castling.remove(CastleRights::WHITE_SHORT),
                (Player::White, Square::A1) => self.castling.remove(CastleRights::WHITE_LONG),
                (Player::Black, Square::H8) => self.castling.remove(CastleRights::BLACK_SHORT),
                (Player::Black, Square::A8) => self.castling.remove(CastleRights::BLACK_LONG),
                _ => (),
            };
            their_pieces.clear(next_move.to);
        }
        if our_pieces.pawns.contains(next_move.from) {
            // Pawn move resets the clock.
            self.halfmove_clock = 0;
            // Check en passant.
            if let Some(en_passant_square) = previous_en_passant {
                if next_move.to == en_passant_square {
                    let captured_pawn = Square::new(next_move.to.file(), next_move.from.rank());
                    their_pieces.pawns.clear(captured_pawn);
                }
            }
            our_pieces.pawns.clear(next_move.from);
            // Check promotions.
            // TODO: Debug assertions to make sure the promotion is valid.
            if let Some(promotion) = next_move.promotion {
                match promotion {
                    Promotion::Queen => our_pieces.queens.extend(next_move.to),
                    Promotion::Rook => our_pieces.rooks.extend(next_move.to),
                    Promotion::Bishop => our_pieces.bishops.extend(next_move.to),
                    Promotion::Knight => our_pieces.knights.extend(next_move.to),
                };
                return;
            }
            our_pieces.pawns.extend(next_move.to);
            let single_push_square = next_move.from.shift(us.push_direction()).unwrap();
            if next_move.from.rank() == Rank::pawns_starting(us)
                && next_move.from.file() == next_move.to.file()
                && single_push_square != next_move.to
                // Technically, this is not correct: https://github.com/jhlywa/chess.js/issues/294
                && (their_pieces.pawns & attacks::pawn_attacks(single_push_square, us)).has_any()
            {
                self.en_passant_square = Some(single_push_square);
            }
            return;
        }
        if our_pieces.king.contains(next_move.from) {
            // Check if the move is castling.
            if next_move.from.rank() == our_backrank
                && next_move.to.rank() == our_backrank
                && next_move.from.file() == File::E
            {
                if next_move.to.file() == File::G {
                    // TODO: debug_assert!(self.can_castle_short())
                    our_pieces.rooks.clear(Square::new(File::H, our_backrank));
                    our_pieces.rooks.extend(Square::new(File::F, our_backrank));
                } else if next_move.to.file() == File::C {
                    // TODO: debug_assert!(self.can_castle_long())
                    our_pieces.rooks.clear(Square::new(File::A, our_backrank));
                    our_pieces.rooks.extend(Square::new(File::D, our_backrank));
                }
            }
            our_pieces.king.clear(next_move.from);
            our_pieces.king.extend(next_move.to);
            // The king has moved: reset castling.
            match us {
                Player::White => self.castling.remove(CastleRights::WHITE_BOTH),
                Player::Black => self.castling.remove(CastleRights::BLACK_BOTH),
            };
            return;
        }
        // Regular moves: put the piece from the source to destination. We
        // already cleared the opponent piece if there was a capture.
        for piece in [
            &mut our_pieces.queens,
            &mut our_pieces.rooks,
            &mut our_pieces.bishops,
            &mut our_pieces.knights,
        ] {
            if piece.contains(next_move.from) {
                piece.clear(next_move.from);
                piece.extend(next_move.to);
                return;
            }
        }
    }

    #[must_use]
    pub fn has_insufficient_material(&self) -> bool {
        todo!()
    }

    #[must_use]
    pub fn is_legal(&self) -> bool {
        validate(self).is_ok()
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

/// [Perft] (**per**formance **t**esting) is a technique for checking
/// correctness of move generation by traversing the tree of possible positions
/// (nodes) and calculating all the leaf nodes at certain depth.
///
/// [Perft]: https://www.chessprogramming.org/Perft
#[must_use]
pub fn perft(position: &Position, depth: u8) -> u64 {
    debug_assert!(position.is_legal());
    if depth == 0 {
        return 1;
    }
    let mut nodes = 0;
    for next_move in position.generate_moves().iter() {
        let mut next_position = position.clone();
        next_position.make_move(next_move);
        nodes += perft(&next_position, depth - 1);
    }
    nodes
}

// Checks if the position is "legal", i.e. if it can be reasoned about by
// the engine. Checking whether the position is truly reachable from the
// starting position (either in standard chess or Fischer Random Chess)
// requires retrograde analysis and potentially unreasonable amount of time.
// This check employs a limited number of heuristics that filter out the
// most obvious incorrect positions and prevents them from being analyzed.
// This helps set up barrier (constructing positions from FEN) between the
// untrusted environment (UCI front-end, user input) and the engine.
#[must_use]
fn validate(position: &Position) -> anyhow::Result<()> {
    if position.fullmove_counter == 0 {
        bail!("fullmove counter cannot be zero")
    }
    // TODO: Probe opposite checks.
    // TODO: The following patterns look repetitive; maybe refactor the
    // common structure even though it's quite short?
    if position.board.white_pieces.king.count() != 1 {
        bail!(
            "expected 1 white king, got {}",
            position.board.white_pieces.king.count()
        )
    }
    if position.board.black_pieces.king.count() != 1 {
        bail!(
            "expected 1 black king, got {}",
            position.board.black_pieces.king.count()
        )
    }
    if position.board.white_pieces.pawns.count() > 8 {
        bail!(
            "expected <= 8 white pawns, got {}",
            position.board.white_pieces.pawns.count()
        )
    }
    if position.board.black_pieces.pawns.count() > 8 {
        bail!(
            "expected <= 8 black pawns, got {}",
            position.board.black_pieces.pawns.count()
        )
    }
    if ((position.board.white_pieces.pawns | position.board.black_pieces.pawns)
        & (Rank::One.mask() | Rank::Eight.mask()))
    .has_any()
    {
        bail!("pawns can not be placed on backranks")
    }
    let attacks = attacks::AttackInfo::new(position);
    // Can't have more than two checks.
    if attacks.checkers.count() > 2 {
        bail!("expected <= 2 checks, got {}", attacks.checkers.count())
    }
    if let Some(en_passant_square) = position.en_passant_square {
        let expected_rank = match position.side_to_move {
            Player::White => Rank::Six,
            Player::Black => Rank::Three,
        };
        if en_passant_square.rank() != expected_rank {
            bail!(
                "expected en passant square to be on rank {}, got {}",
                expected_rank,
                en_passant_square.rank()
            )
        }
        // A pawn that was just pushed by our opponent should be in front of
        // en_passant_square.
        let pushed_pawn = en_passant_square
            .shift(position.they().push_direction())
            .expect("we already checked for correct rank");
        if !position.pieces(position.they()).pawns.contains(pushed_pawn) {
            bail!("en passant square is not beyond pushed pawn")
        }
        // If en-passant was played and there's a check, doubly pushed pawn
        // should be the only checker or it should be a discovery.
        let king = position.pieces(position.us()).king.as_square();
        if attacks.checkers.has_any() {
            if attacks.checkers.count() > 1 {
                bail!("more than 1 check after double pawn push is impossible")
            }
            // The check wasn't delivered by pushed pawn.
            if attacks.checkers != Bitboard::from(pushed_pawn) {
                let checker = attacks.checkers.as_square();
                let original_square = en_passant_square
                    .shift(position.us().push_direction())
                    .unwrap();
                if !(attacks::ray(checker, king).contains(original_square)) {
                    bail!(
                        "the only possible checks after double pawn push are either discovery \
                            targeting the original pawn square or the pushed pawn itself"
                    )
                }
            }
        }
        // Doubly pushed pawn can not block a diagonal check.
        for attacker in (position.pieces(position.they()).queens
            | position.pieces(position.they()).bishops)
            .iter()
        {
            let xray = attacks::bishop_ray(attacker, king);
            if (xray & (position.occupied_squares())).count() == 2
                && xray.contains(attacker)
                && xray.contains(pushed_pawn)
            {
                bail!("doubly pushed pawn can not be the only blocker on a diagonal")
            }
        }
    }
    Ok(())
}
