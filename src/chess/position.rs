//! Provides fully-specified [Chess Position] implementation: stores
//! information about the board and tracks the state of castling, 50-move rule
//! draw, etc.
//!
//! The core of Move Generator and move making is also implemented here as a way
//! to produce ways of mutating [`Position`].
//!
//! [Chess Position]: https://www.chessprogramming.org/Chess_Position

use std::fmt::{self, Write};

use anyhow::{Context, bail};

use super::core::{Direction, PieceKind};
use crate::chess::bitboard::{Bitboard, Pieces};
use crate::chess::core::{
    BOARD_WIDTH, CastleRights, File, Move, MoveList, Piece, Promotion, Rank, Square,
};
use crate::chess::{attacks, generated, zobrist};
use crate::environment::Player;

/// Piece-centric implementation of the chess position, which includes all
/// pieces and their placement, information about the castling rights, side to
/// move, 50 move rule counters etc.
///
/// This is the "back-end" of the chess engine, an efficient board
/// representation is crucial for performance. An alternative implementation
/// would be Square-Piece table but both have different trade-offs and scenarios
/// where they are efficient.  It is likely that the best overall performance
/// can be achieved by keeping both to complement each other.
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
#[derive(Clone)]
pub struct Position {
    white_pieces: Pieces,
    black_pieces: Pieces,
    castling: CastleRights,
    side_to_move: Player,
    /// [Halfmove Clock][^ply] keeps track of the number of halfmoves since the
    /// last capture or pawn move and is used to enforce fifty[^fifty]-move draw
    /// rule.
    ///
    /// [Halfmove Clock]: https://www.chessprogramming.org/Halfmove_Clock
    /// [^ply]: Half-move or [ply](https://www.chessprogramming.org/Ply) means a move of only
    ///     one side.
    /// [^fifty]: 50 __full__ moves
    halfmove_clock: u8,
    fullmove_counter: u16,
    en_passant_square: Option<Square>,
    hash: zobrist::Key,
}

/// Context for move generation to reduce parameter passing and encapsulate
/// common logic
struct MoveGenContext<'a> {
    us: Player,
    them: Player,
    king: Square,
    pins: Bitboard,
    blocking_ray: Bitboard,
    their_or_empty: Bitboard,
    occupied_squares: Bitboard,
    moves: &'a mut MoveList,
}

impl<'a> MoveGenContext<'a> {
    fn new(
        position: &Position,
        moves: &'a mut MoveList,
        blocking_ray: Bitboard,
        pins: Bitboard,
    ) -> Self {
        let us = position.side_to_move;
        let them = !us;
        let king = position.pieces(us).king.as_square();
        let their_or_empty = !position.occupancy(us);
        let occupied_squares = position.occupied_squares();

        Self {
            us,
            them,
            king,
            pins,
            blocking_ray,
            their_or_empty,
            occupied_squares,
            moves,
        }
    }

    /// Check if a pinned piece's move is legal along the pin ray
    fn is_pin_legal(&self, from: Square, to: Square) -> bool {
        !self.pins.contains(from)
            || (attacks::ray(from, self.king) & attacks::ray(to, self.king)).has_any()
    }

    /// Add a move if it's legal with respect to pins and blocking rays
    unsafe fn try_add_move(&mut self, from: Square, to: Square, promotion: Option<Promotion>) {
        if self.blocking_ray.contains(to) && self.is_pin_legal(from, to) {
            unsafe {
                self.moves.push_unchecked(Move::new(from, to, promotion));
            }
        }
    }

    /// Add promotion moves if the target square is a promotion square
    unsafe fn try_add_move_with_promotions(&mut self, from: Square, to: Square) {
        match to.rank() {
            Rank::Rank1 | Rank::Rank8 => unsafe {
                self.try_add_move(from, to, Some(Promotion::Queen));
                self.try_add_move(from, to, Some(Promotion::Rook));
                self.try_add_move(from, to, Some(Promotion::Bishop));
                self.try_add_move(from, to, Some(Promotion::Knight));
            },
            _ => unsafe { self.try_add_move(from, to, None) },
        }
    }
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
        let mut result = Self {
            white_pieces: Pieces::starting(Player::White),
            black_pieces: Pieces::starting(Player::Black),
            castling: CastleRights::ALL,
            side_to_move: Player::White,
            halfmove_clock: 0,
            fullmove_counter: 1,
            en_passant_square: None,
            hash: zobrist::Key::default(),
        };
        result.hash = result.compute_hash();
        result
    }

    pub(crate) const fn us(&self) -> Player {
        self.side_to_move
    }

    pub(crate) fn them(&self) -> Player {
        !self.us()
    }

    pub(crate) fn pieces(&self, player: Player) -> &Pieces {
        match player {
            Player::White => &self.white_pieces,
            Player::Black => &self.black_pieces,
        }
    }

    fn pieces_mut(&mut self, player: Player) -> &mut Pieces {
        match player {
            Player::White => &mut self.white_pieces,
            Player::Black => &mut self.black_pieces,
        }
    }

    /// Returns Zobrist hash of the position.
    #[must_use]
    pub fn hash(&self) -> zobrist::Key {
        self.hash
    }

    pub(crate) const fn castling(&self) -> CastleRights {
        self.castling
    }

    pub(crate) const fn en_passant(&self) -> Option<Square> {
        self.en_passant_square
    }

    pub(crate) const fn halfmove_clock(&self) -> u8 {
        self.halfmove_clock
    }

    /// Calls `f` for every piece on the board.
    pub(crate) fn for_each_piece(&self, mut f: impl FnMut(Square, Piece)) {
        for player in [Player::White, Player::Black] {
            let pieces = self.pieces(player);
            for (bitboard, kind) in [
                (pieces.pawns, PieceKind::Pawn),
                (pieces.knights, PieceKind::Knight),
                (pieces.bishops, PieceKind::Bishop),
                (pieces.rooks, PieceKind::Rook),
                (pieces.queens, PieceKind::Queen),
                (pieces.king, PieceKind::King),
            ] {
                for square in bitboard.iter() {
                    f(square, Piece { player, kind });
                }
            }
        }
    }

    fn occupancy(&self, player: Player) -> Bitboard {
        self.pieces(player).all()
    }

    fn occupied_squares(&self) -> Bitboard {
        self.occupancy(self.us()) | self.occupancy(self.them())
    }

    pub fn num_pieces(&self) -> usize {
        self.occupied_squares().count() as usize
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
    // TODO: Add support for Shredder FEN and Chess960.
    pub fn from_fen(input: &str) -> anyhow::Result<Self> {
        let mut white_pieces = Pieces::empty();
        let mut black_pieces = Pieces::empty();

        let mut parts = input.split(' ');
        // Parse Piece Placement.
        let pieces_placement = match parts.next() {
            Some(placement) => placement,
            None => bail!("missing pieces placement"),
        };
        let ranks = pieces_placement.split('/');
        let mut rank_id = 8;
        for rank_fen in ranks {
            if rank_id == 0 {
                bail!("expected 8 ranks, got {pieces_placement}");
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
                    }
                    _ => (),
                }
                let piece = Piece::try_from(symbol)?;
                let pieces = match piece.player {
                    Player::White => &mut white_pieces,
                    Player::Black => &mut black_pieces,
                };
                let square = Square::new(file.try_into()?, rank);
                *pieces.bitboard_for_mut(piece.kind) |= Bitboard::from(square);
                file += 1;
            }
            if file != BOARD_WIDTH {
                bail!(
                    "rank size should be exactly {BOARD_WIDTH},
                     got {rank_fen} of length {file}"
                );
            }
        }
        if rank_id != 0 {
            bail!("there should be 8 ranks, got {pieces_placement}");
        }
        let side_to_move = match parts.next() {
            Some(value) => value.try_into()?,
            None => bail!("missing side to move"),
        };
        let castling = match parts.next() {
            Some(value) => value.try_into()?,
            None => bail!("missing castling rights"),
        };
        let en_passant_square = match parts.next() {
            Some("-") => None,
            Some(value) => Some(value.try_into()?),
            None => bail!("missing en passant square"),
        };
        let halfmove_clock = match parts.next() {
            Some(value) => match value.parse::<u8>() {
                Ok(num) => Some(num),
                Err(e) => {
                    return Err(e)
                        .with_context(|| format!("halfmove clock can not be parsed {value}"));
                }
            },
            None => None,
        };
        let fullmove_counter = match parts.next() {
            Some(value) => match value.parse::<u16>() {
                Ok(0) => {
                    bail!("fullmove counter can not be 0")
                }
                Ok(num) => Some(num),
                Err(e) => {
                    return Err(e)
                        .with_context(|| format!("fullmove counter can not be parsed {value}"));
                }
            },
            None => match halfmove_clock {
                Some(_) => bail!("if halfmove clock is present, fullmove counter must be present"),
                // This is a correct EPD position.
                None => None,
            },
        };

        if parts.next().is_some() {
            bail!("trailing symbols");
        }

        let halfmove_clock = halfmove_clock.unwrap_or(0);
        let fullmove_counter = fullmove_counter.unwrap_or(1);

        let mut result = Self {
            white_pieces,
            black_pieces,
            castling,
            side_to_move,
            halfmove_clock,
            fullmove_counter,
            en_passant_square,
            hash: zobrist::Key::default(),
        };
        result.hash = result.compute_hash();

        match validate(&result) {
            Ok(()) => Ok(result),
            Err(e) => Err(e.context("illegal position")),
        }
    }

    /// Checks whether a position is pseudo-legal. This is a simple check to
    /// ensure that the state is not corrupted and is safe to work with. It
    /// doesn't handle all corner cases and is simply used to as a sanity check.
    #[must_use]
    pub(crate) fn is_legal(&self) -> bool {
        validate(self).is_ok()
    }

    pub(super) fn attack_info(&self) -> attacks::AttackInfo {
        let (us, them) = (self.us(), self.them());
        let (our_pieces, their_pieces) = (self.pieces(us), self.pieces(them));
        let king: Square = our_pieces.king.as_square();
        let (our_occupancy, their_occupancy) = (our_pieces.all(), their_pieces.all());
        let occupancy = our_occupancy | their_occupancy;
        attacks::AttackInfo::new(them, their_pieces, king, our_occupancy, occupancy)
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
    // TODO: Look at and compare speed with https://github.com/jordanbray/chess
    // TODO: Maybe use python-chess testset of perft moves:
    // https://github.com/niklasf/python-chess/blob/master/examples/perft/random.perft
    // TODO: Compare with other engines and perft generators
    // (https://github.com/jniemann66/juddperft).
    // TODO: Check movegen comparison (https://github.com/Gigantua/Chess_Movegen).
    #[must_use]
    pub fn generate_moves(&self) -> MoveList {
        let mut moves = MoveList::new();
        debug_assert!(self.is_legal());

        let (us, them) = (self.us(), self.them());
        let (our_pieces, their_pieces) = (self.pieces(us), self.pieces(them));
        let king: Square = our_pieces.king.as_square();
        let (our_occupancy, their_occupancy) = (our_pieces.all(), their_pieces.all());
        let occupied_squares = our_occupancy | their_occupancy;
        let attack_info =
            attacks::AttackInfo::new(them, their_pieces, king, our_occupancy, occupied_squares);

        // Moving the king to safety is always a valid move
        generate_king_moves(king, attack_info.safe_king_squares, &mut moves);

        // If there are multiple checks, only king moves are legal
        if attack_info.checkers.count() > 1 {
            return moves;
        }

        // Calculate blocking ray for single check or all squares for no check
        let blocking_ray = if attack_info.checkers.count() == 1 {
            let checker: Square = attack_info.checkers.as_square();
            let ray = attacks::ray(checker, king);
            if ray.is_empty() {
                // Checker is a knight: capture is the only way to resolve check
                attack_info.checkers
            } else {
                // Checker is a sliding piece: both capturing and blocking resolves check
                ray
            }
        } else {
            Bitboard::full()
        };

        let mut ctx = MoveGenContext::new(self, &mut moves, blocking_ray, attack_info.pins);

        // Generate non-king moves
        generate_knight_moves(our_pieces.knights, &mut ctx);

        // Generate rook-like moves (rooks and queens)
        generate_sliding_moves(
            our_pieces.rooks | our_pieces.queens,
            &mut ctx,
            attacks::rook_attacks,
        );
        // Generate bishop-like moves (bishops and queens)
        generate_sliding_moves(
            our_pieces.bishops | our_pieces.queens,
            &mut ctx,
            attacks::bishop_attacks,
        );

        generate_pawn_moves(
            our_pieces.pawns,
            their_pieces,
            their_occupancy,
            attack_info.checkers,
            self.en_passant_square,
            &mut ctx,
        );

        generate_castle_moves(
            us,
            attack_info.checkers,
            self.castling,
            attack_info.attacks,
            occupied_squares,
            &mut moves,
        );

        moves
    }

    /// Transitions to the next position by applying the move.
    ///
    /// This is the only way to mutate the position and it will ensure that the
    /// cached information (e.g. hash) is updated correctly.
    pub fn make_move(&mut self, next_move: &Move) {
        debug_assert!(self.is_legal());

        let us = self.side_to_move;
        let (from, to) = (next_move.from(), next_move.to());
        let moved = self
            .pieces(us)
            .at(from)
            .expect("a legal move originates from an occupied square");

        // Increment the halfmove clock early: captures and pawn moves reset it
        // below. Saturate to avoid overflow in (unadjudicated) games that run
        // long past the 50-move threshold.
        self.halfmove_clock = self.halfmove_clock.saturating_add(1);

        // En passant is transient (lasts one move): clear it from the state and
        // the hash, remembering the square to detect an en passant capture.
        let previous_en_passant = self.en_passant_square.take();
        if let Some(ep_square) = previous_en_passant {
            self.hash ^= generated::EN_PASSANT_FILES[ep_square.file() as usize];
        }

        self.update_castling_rights(next_move);
        self.remove_captured_piece(to);

        match moved {
            PieceKind::Pawn => {
                self.make_pawn_move(from, to, next_move.promotion(), previous_en_passant)
            }
            PieceKind::King => self.make_king_move(from, to),
            kind => self.move_piece(Piece { player: us, kind }, from, to),
        }

        if us == Player::Black {
            self.fullmove_counter += 1;
        }
        self.side_to_move = !us;
        self.hash ^= generated::BLACK_TO_MOVE;
    }

    /// Adds `piece` to `square` if it is absent or removes it if present,
    /// keeping the Zobrist hash in sync. Because keys are XOR-ed in, the same
    /// call also undoes itself.
    fn toggle_piece(&mut self, piece: Piece, square: Square) {
        self.pieces_mut(piece.player)
            .bitboard_for_mut(piece.kind)
            .toggle(square);
        self.hash ^= generated::get_piece_key(piece, square);
    }

    /// Moves `piece` between two squares, keeping the hash in sync.
    fn move_piece(&mut self, piece: Piece, from: Square, to: Square) {
        self.toggle_piece(piece, from);
        self.toggle_piece(piece, to);
    }

    fn update_castling_rights(&mut self, next_move: &Move) {
        self.update_side_castling_rights(Player::White, next_move);
        self.update_side_castling_rights(Player::Black, next_move);
    }

    fn update_side_castling_rights(&mut self, player: Player, next_move: &Move) {
        let (king, short_rook, long_rook, short, long, short_key, long_key) = match player {
            Player::White => (
                Square::E1,
                Square::H1,
                Square::A1,
                CastleRights::WHITE_SHORT,
                CastleRights::WHITE_LONG,
                generated::WHITE_CAN_CASTLE_SHORT,
                generated::WHITE_CAN_CASTLE_LONG,
            ),
            Player::Black => (
                Square::E8,
                Square::H8,
                Square::A8,
                CastleRights::BLACK_SHORT,
                CastleRights::BLACK_LONG,
                generated::BLACK_CAN_CASTLE_SHORT,
                generated::BLACK_CAN_CASTLE_LONG,
            ),
        };

        // A castling right is lost when the king or the corresponding rook
        // leaves its square, or when that rook is captured on its square.
        for (right, rook, key) in [(short, short_rook, short_key), (long, long_rook, long_key)] {
            if self.castling.contains(right)
                && (next_move.from() == king || next_move.from() == rook || next_move.to() == rook)
            {
                self.castling.remove(right);
                self.hash ^= key;
            }
        }
    }

    /// Removes the opponent's piece captured on `square` (if any) and resets
    /// the halfmove clock. En passant captures do not land on the captured
    /// pawn's square and are handled in [`Self::make_pawn_move`].
    fn remove_captured_piece(&mut self, square: Square) {
        let them = !self.side_to_move;
        if let Some(kind) = self.pieces(them).at(square) {
            self.halfmove_clock = 0;
            self.toggle_piece(Piece { player: them, kind }, square);
        }
    }

    fn make_pawn_move(
        &mut self,
        from: Square,
        to: Square,
        promotion: Option<Promotion>,
        previous_en_passant: Option<Square>,
    ) {
        let us = self.side_to_move;
        // Any pawn move resets the halfmove clock.
        self.halfmove_clock = 0;

        // A pawn landing on the en passant square captures the pawn that
        // double-pushed on the previous move (which sits beside the target).
        if previous_en_passant == Some(to) {
            let captured = Square::new(to.file(), from.rank());
            self.toggle_piece(
                Piece {
                    player: !us,
                    kind: PieceKind::Pawn,
                },
                captured,
            );
        }

        self.toggle_piece(
            Piece {
                player: us,
                kind: PieceKind::Pawn,
            },
            from,
        );
        if let Some(promotion) = promotion {
            self.toggle_piece(
                Piece {
                    player: us,
                    kind: promotion.into(),
                },
                to,
            );
            return;
        }
        self.toggle_piece(
            Piece {
                player: us,
                kind: PieceKind::Pawn,
            },
            to,
        );

        // A double push that an enemy pawn could answer sets the en passant
        // square.
        let single_push = from.shift(pawn_push_direction(us)).unwrap();
        if from.rank() == Rank::pawns_starting(us)
            && from.file() == to.file()
            && single_push != to
            // Technically not fully correct: https://github.com/jhlywa/chess.js/issues/294
            && (self.pieces(!us).pawns & attacks::pawn_attacks(single_push, us)).has_any()
        {
            self.en_passant_square = Some(single_push);
            self.hash ^= generated::EN_PASSANT_FILES[single_push.file() as usize];
        }
    }

    fn make_king_move(&mut self, from: Square, to: Square) {
        let us = self.side_to_move;
        // A king stepping two files (E to G or C) is castling: move the rook to
        // the square the king skipped over.
        if from.file() == File::E && matches!(to.file(), File::G | File::C) {
            let backrank = Rank::backrank(us);
            let (rook_from, rook_to) = if to.file() == File::G {
                (
                    Square::new(File::H, backrank),
                    Square::new(File::F, backrank),
                )
            } else {
                (
                    Square::new(File::A, backrank),
                    Square::new(File::D, backrank),
                )
            };
            self.move_piece(
                Piece {
                    player: us,
                    kind: PieceKind::Rook,
                },
                rook_from,
                rook_to,
            );
        }
        self.move_piece(
            Piece {
                player: us,
                kind: PieceKind::King,
            },
            from,
            to,
        );
    }

    #[must_use]
    pub fn in_check(&self) -> bool {
        let king = self.pieces(self.us()).king.as_square();
        self.is_attacked_by(king, self.them())
    }

    /// Returns true if the given square is attacked by any piece of the given
    /// player.
    fn is_attacked_by(&self, target: Square, attacker: Player) -> bool {
        let their = self.pieces(attacker);
        let occupancy = self.occupied_squares();

        // Pawn attacks: compute candidate squares via bitboard shifts instead
        // of using the pawn attack table, which has empty entries for back-rank
        // squares (rank 1 and 8) where kings can reside.
        const NOT_A_FILE: Bitboard = Bitboard::from_bits(0xFEFE_FEFE_FEFE_FEFE);
        const NOT_H_FILE: Bitboard = Bitboard::from_bits(0x7F7F_7F7F_7F7F_7F7F);
        let target_bb = Bitboard::from(target);
        let pawn_attacker_squares = match attacker {
            // White pawns attack from one rank below the target.
            Player::White => ((target_bb >> 9) & NOT_H_FILE) | ((target_bb >> 7) & NOT_A_FILE),
            // Black pawns attack from one rank above the target.
            Player::Black => ((target_bb << 7) & NOT_H_FILE) | ((target_bb << 9) & NOT_A_FILE),
        };

        (attacks::knight_attacks(target) & their.knights).has_any()
            || (pawn_attacker_squares & their.pawns).has_any()
            || (attacks::king_attacks(target) & their.king).has_any()
            || (attacks::rook_attacks(target, occupancy) & (their.rooks | their.queens)).has_any()
            || (attacks::bishop_attacks(target, occupancy) & (their.bishops | their.queens))
                .has_any()
    }

    /// Returns true if 50-move rule draw is in effect.
    #[must_use]
    pub fn halfmove_clock_expired(&self) -> bool {
        self.halfmove_clock >= 100
    }

    /// Returns true if neither side has sufficient material to checkmate.
    ///
    /// Detected cases:
    /// - K vs K
    /// - K+N vs K
    /// - K+B vs K
    /// - K+B vs K+B (same-colored bishops)
    #[must_use]
    pub fn is_insufficient_material(&self) -> bool {
        let (white, black) = (&self.white_pieces, &self.black_pieces);

        // Any pawns, rooks, or queens means sufficient material.
        if (white.pawns | black.pawns | white.rooks | black.rooks | white.queens | black.queens)
            .has_any()
        {
            return false;
        }

        let white_minors = white.knights.count() + white.bishops.count();
        let black_minors = black.knights.count() + black.bishops.count();

        // K vs K
        if white_minors == 0 && black_minors == 0 {
            return true;
        }

        // K+minor vs K
        if (white_minors == 1 && black_minors == 0) || (white_minors == 0 && black_minors == 1) {
            return true;
        }

        // K+B vs K+B with same-colored bishops
        if white.knights.count() == 0
            && black.knights.count() == 0
            && white.bishops.count() == 1
            && black.bishops.count() == 1
        {
            const DARK_SQUARES: Bitboard = Bitboard::from_bits(0xAA55_AA55_AA55_AA55);
            let all_bishops = white.bishops | black.bishops;
            let bishops_on_dark = all_bishops & DARK_SQUARES;
            // All bishops on same color: either all dark or all light.
            return bishops_on_dark.is_empty() || bishops_on_dark == all_bishops;
        }

        false
    }

    #[must_use]
    pub(crate) fn at(&self, square: Square) -> Option<Piece> {
        if let Some(kind) = self.white_pieces.at(square) {
            return Some(Piece {
                player: Player::White,
                kind,
            });
        }
        if let Some(kind) = self.black_pieces.at(square) {
            return Some(Piece {
                player: Player::Black,
                kind,
            });
        }
        None
    }

    /// Computes standard Zobrist hash of the position using pseudo-random
    /// numbers generated during the build stage.
    ///
    /// This is not efficient and is only used when a position is created. The
    /// hash is then cached and updated incrementally after each move.
    fn compute_hash(&self) -> zobrist::Key {
        let mut key = 0;

        if self.side_to_move == Player::Black {
            key ^= generated::BLACK_TO_MOVE;
        }

        if self.castling.contains(CastleRights::WHITE_SHORT) {
            key ^= generated::WHITE_CAN_CASTLE_SHORT;
        }
        if self.castling.contains(CastleRights::WHITE_LONG) {
            key ^= generated::WHITE_CAN_CASTLE_LONG;
        }
        if self.castling.contains(CastleRights::BLACK_SHORT) {
            key ^= generated::BLACK_CAN_CASTLE_SHORT;
        }
        if self.castling.contains(CastleRights::BLACK_LONG) {
            key ^= generated::BLACK_CAN_CASTLE_LONG;
        }

        if let Some(ep_square) = self.en_passant_square {
            key ^= generated::EN_PASSANT_FILES[ep_square.file() as usize];
        }

        for square in self.occupied_squares().iter() {
            let piece = self.at(square).expect("occupied square");
            key ^= generated::get_piece_key(piece, square);
        }

        key
    }
}

impl TryFrom<&str> for Position {
    type Error = anyhow::Error;

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
    /// Returns position representation in Forsyth-Edwards Notation (FEN).
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for rank_idx in (0..BOARD_WIDTH).rev() {
            let rank: Rank = unsafe { std::mem::transmute(rank_idx) };
            let mut empty_squares = 0i32;
            for file_idx in 0..BOARD_WIDTH {
                let file: File = unsafe { std::mem::transmute(file_idx) };
                let square = Square::new(file, rank);
                if let Some(piece) = self.at(square) {
                    if empty_squares != 0 {
                        write!(f, "{empty_squares}")?;
                        empty_squares = 0;
                    }
                    write!(f, "{piece}")?;
                } else {
                    empty_squares += 1;
                }
            }
            if empty_squares != 0 {
                write!(f, "{empty_squares}")?;
            }
            if rank != Rank::Rank1 {
                const RANK_SEPARATOR: char = '/';
                write!(f, "{RANK_SEPARATOR}")?;
            }
        }
        write!(f, " {} ", self.side_to_move)?;
        write!(f, "{} ", self.castling)?;
        match self.en_passant_square {
            Some(square) => write!(f, "{square} "),
            None => write!(f, "- "),
        }?;
        write!(f, "{} ", self.halfmove_clock)?;
        write!(f, "{}", self.fullmove_counter)?;
        Ok(())
    }
}

impl fmt::Debug for Position {
    /// Dumps the board in a human readable format ('.' for empty square, FEN
    /// algebraic symbol for piece).
    ///
    /// Useful for debugging purposes.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Board:")?;

        const LINE_SEPARATOR: &str = "\n";
        const SQUARE_SEPARATOR: &str = " ";

        for rank_idx in (0..BOARD_WIDTH).rev() {
            let rank: Rank = unsafe { std::mem::transmute(rank_idx) };
            for file_idx in 0..BOARD_WIDTH {
                let file: File = unsafe { std::mem::transmute(file_idx) };
                match self.at(Square::new(file, rank)) {
                    Some(piece) => write!(f, "{piece}"),
                    None => f.write_char('.'),
                }?;
                if file != File::H {
                    write!(f, "{SQUARE_SEPARATOR}")?;
                }
            }
            write!(f, "{LINE_SEPARATOR}")?;
        }
        write!(f, "{LINE_SEPARATOR}")?;

        writeln!(f, "Player to move: {:?}", self.side_to_move)?;
        writeln!(f, "Fullmove counter: {:?}", self.fullmove_counter)?;
        writeln!(f, "En Passant: {:?}", self.en_passant_square)?;
        // bitflags' default fmt::Debug implementation is not very convenient:
        // dump FEN instead.
        writeln!(f, "Castling rights: {}", self.castling)?;
        writeln!(f, "FEN: {}", self)?;

        Ok(())
    }
}

/// [Perft] (**per**formance **t**esting) is a technique for checking
/// correctness of move generation by traversing the tree of possible positions
/// (nodes) and calculating all the leaf nodes at certain depth.
///
/// Here is a useful perft exploration web tool: <https://analog-hors.github.io/webperft/>
///
/// [Perft]: https://www.chessprogramming.org/Perft
#[must_use]
pub fn perft(position: &Position, depth: u8) -> u64 {
    debug_assert!(position.is_legal());
    if depth == 0 {
        return 1;
    }
    if depth == 1 {
        return position.generate_moves().len() as u64;
    }
    let mut nodes = 0;
    for next_move in position.generate_moves() {
        let mut next_position = position.clone();
        next_position.make_move(&next_move);
        nodes += perft(&next_position, depth - 1);
    }
    nodes
}

/// Checks if the position is "legal", i.e. if it can be reasoned about by the
/// engine. Checking whether the position is truly reachable from the starting
/// position (either in standard chess or Chess960) requires retrograde analysis
/// and potentially unreasonable amount of time.  This check employs a limited
/// number of heuristics that filter out the most obvious incorrect positions
/// and prevents them from being analyzed.  This helps set up barrier
/// (constructing positions from FEN) between the untrusted environment (UCI
/// front-end, user input) and the engine.
fn validate(position: &Position) -> anyhow::Result<()> {
    if position.fullmove_counter == 0 {
        bail!("fullmove counter cannot be zero")
    }
    // TODO: Probe opposite checks.
    // TODO: The following patterns look repetitive; maybe refactor the
    // common structure even though it's quite short?
    // Validate king counts (exactly one king per side)
    let white_king_count = position.white_pieces.king.count();
    let black_king_count = position.black_pieces.king.count();
    if white_king_count != 1 {
        bail!("expected 1 white king, got {white_king_count}")
    }
    if black_king_count != 1 {
        bail!("expected 1 black king, got {black_king_count}")
    }

    // Validate pawn counts (max 8 pawns per side)
    let white_pawn_count = position.white_pieces.pawns.count();
    let black_pawn_count = position.black_pieces.pawns.count();
    if white_pawn_count > 8 {
        bail!("expected <= 8 white pawns, got {white_pawn_count}")
    }
    if black_pawn_count > 8 {
        bail!("expected <= 8 black pawns, got {black_pawn_count}")
    }
    if ((position.white_pieces.pawns | position.black_pieces.pawns)
        & (Rank::Rank1.mask() | Rank::Rank8.mask()))
    .has_any()
    {
        bail!("pawns can not be placed on backranks")
    }
    // The player that just moved can not be left in check: it would mean that
    // their king can be captured. This also rejects positions with adjacent
    // kings.
    let their_king = position.pieces(position.them()).king.as_square();
    if position.is_attacked_by(their_king, position.us()) {
        bail!("side not to move can not be in check")
    }
    let attack_info = position.attack_info();
    // Can't have more than two checks.
    if attack_info.checkers.count() > 2 {
        bail!("expected <= 2 checks, got {}", attack_info.checkers.count())
    }
    if let Some(en_passant_square) = position.en_passant_square {
        let expected_rank = match position.side_to_move {
            Player::White => Rank::Rank6,
            Player::Black => Rank::Rank3,
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
            .shift(pawn_push_direction(position.them()))
            .unwrap();
        if !position.pieces(position.them()).pawns.contains(pushed_pawn) {
            bail!("en passant square is not beyond pushed pawn")
        }
        // If en-passant was played and there's a check, doubly pushed pawn
        // should be the only checker or it should be a discovery.
        let king = position.pieces(position.us()).king.as_square();
        if attack_info.checkers.has_any() {
            if attack_info.checkers.count() > 1 {
                bail!("more than 1 check after double pawn push is impossible")
            }
            // The check wasn't delivered by pushed pawn.
            if attack_info.checkers != Bitboard::from(pushed_pawn) {
                let checker = attack_info.checkers.as_square();
                let original_square = en_passant_square
                    .shift(pawn_push_direction(position.us()))
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
        for attacker in (position.pieces(position.them()).queens
            | position.pieces(position.them()).bishops)
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

fn generate_king_moves(king: Square, safe_squares: Bitboard, moves: &mut MoveList) {
    for safe_square in safe_squares.iter() {
        unsafe {
            moves.push_unchecked(Move::new(king, safe_square, None));
        }
    }
}

fn generate_knight_moves(knights: Bitboard, ctx: &mut MoveGenContext) {
    // When a knight is pinned, it can't move at all because it can't stay on
    // the same horizontal, vertical or diagonal.
    for from in (knights - ctx.pins).iter() {
        let targets = attacks::knight_attacks(from) & ctx.their_or_empty & ctx.blocking_ray;
        for to in targets.iter() {
            unsafe {
                ctx.try_add_move(from, to, None);
            }
        }
    }
}

fn generate_sliding_moves(
    pieces: Bitboard,
    ctx: &mut MoveGenContext,
    attack_fn: impl Fn(Square, Bitboard) -> Bitboard,
) {
    for from in pieces.iter() {
        let targets = attack_fn(from, ctx.occupied_squares) & ctx.their_or_empty & ctx.blocking_ray;
        for to in targets.iter() {
            unsafe {
                ctx.try_add_move(from, to, None);
            }
        }
    }
}

fn generate_pawn_moves(
    pawns: Bitboard,
    their_pieces: &Pieces,
    their_occupancy: Bitboard,
    checkers: Bitboard,
    en_passant_square: Option<Square>,
    ctx: &mut MoveGenContext,
) {
    // Generate pawn captures
    for from in pawns.iter() {
        let targets = attacks::pawn_attacks(from, ctx.us) & their_occupancy & ctx.blocking_ray;
        for to in targets.iter() {
            unsafe {
                ctx.try_add_move_with_promotions(from, to);
            }
        }
    }

    // Generate en passant captures
    if let Some(en_passant_square) = en_passant_square {
        generate_en_passant_moves(pawns, their_pieces, en_passant_square, checkers, ctx);
    }

    // Generate pawn pushes
    let push_direction = pawn_push_direction(ctx.us);
    let single_pushes = pawns.shift(push_direction) - ctx.occupied_squares;

    // Generate single pushes with potential promotions
    for to in (single_pushes & ctx.blocking_ray).iter() {
        let from = to.shift(push_direction.opposite()).unwrap();
        if ctx.is_pin_legal(from, to) {
            unsafe {
                ctx.try_add_move_with_promotions(from, to);
            }
        }
    }

    // Generate double pushes (only from starting rank)
    let starting_rank = Rank::pawns_starting(ctx.us).mask();
    let double_push_candidates = single_pushes & starting_rank.shift(push_direction);
    let double_pushes = double_push_candidates.shift(push_direction) - ctx.occupied_squares;

    for to in (double_pushes & ctx.blocking_ray).iter() {
        let from = to
            .shift(push_direction.opposite())
            .unwrap()
            .shift(push_direction.opposite())
            .unwrap();
        if ctx.is_pin_legal(from, to) {
            unsafe {
                ctx.try_add_move(from, to, None);
            }
        }
    }
}

fn generate_en_passant_moves(
    pawns: Bitboard,
    their_pieces: &Pieces,
    en_passant_square: Square,
    checkers: Bitboard,
    ctx: &mut MoveGenContext,
) {
    let en_passant_pawn = en_passant_square
        .shift(pawn_push_direction(ctx.them))
        .unwrap();
    let candidate_pawns = attacks::pawn_attacks(en_passant_square, ctx.them) & pawns;

    if checkers.contains(en_passant_pawn) {
        // If the en passant capture resolves a check, it's always legal
        for our_pawn in candidate_pawns.iter() {
            if !ctx.pins.contains(our_pawn) {
                unsafe {
                    ctx.moves
                        .push_unchecked(Move::new(our_pawn, en_passant_square, None));
                }
            }
        }
    } else {
        // Check if capturing en passant does not create a discovered check
        for our_pawn in candidate_pawns.iter() {
            let mut occupancy_after_capture = ctx.occupied_squares;
            occupancy_after_capture.clear(our_pawn);
            occupancy_after_capture.clear(en_passant_pawn);
            occupancy_after_capture.extend(en_passant_square);

            if !is_discovered_check_after_en_passant(
                ctx.king,
                their_pieces,
                occupancy_after_capture,
            ) {
                unsafe {
                    ctx.moves
                        .push_unchecked(Move::new(our_pawn, en_passant_square, None));
                }
            }
        }
    }
}

fn is_discovered_check_after_en_passant(
    king: Square,
    their_pieces: &Pieces,
    occupancy: Bitboard,
) -> bool {
    (attacks::queen_attacks(king, occupancy) & their_pieces.queens).has_any()
        || (attacks::rook_attacks(king, occupancy) & their_pieces.rooks).has_any()
        || (attacks::bishop_attacks(king, occupancy) & their_pieces.bishops).has_any()
}

/// Static geometry of a single castling move.
struct Castle {
    right: CastleRights,
    /// Squares the king passes over, including its destination; none of them
    /// may be attacked.
    king_walk: Bitboard,
    /// Squares between the king and the rook; all of them must be empty.
    rook_walk: Bitboard,
    king_from: Square,
    king_to: Square,
}

impl Castle {
    /// The two castling moves available to each player, kingside then
    /// queenside.
    const fn for_player(player: Player) -> &'static [Self; 2] {
        match player {
            Player::White => &[
                Self {
                    right: CastleRights::WHITE_SHORT,
                    king_walk: attacks::WHITE_SHORT_CASTLE_KING_WALK,
                    rook_walk: attacks::WHITE_SHORT_CASTLE_ROOK_WALK,
                    king_from: Square::E1,
                    king_to: Square::G1,
                },
                Self {
                    right: CastleRights::WHITE_LONG,
                    king_walk: attacks::WHITE_LONG_CASTLE_KING_WALK,
                    rook_walk: attacks::WHITE_LONG_CASTLE_ROOK_WALK,
                    king_from: Square::E1,
                    king_to: Square::C1,
                },
            ],
            Player::Black => &[
                Self {
                    right: CastleRights::BLACK_SHORT,
                    king_walk: attacks::BLACK_SHORT_CASTLE_KING_WALK,
                    rook_walk: attacks::BLACK_SHORT_CASTLE_ROOK_WALK,
                    king_from: Square::E8,
                    king_to: Square::G8,
                },
                Self {
                    right: CastleRights::BLACK_LONG,
                    king_walk: attacks::BLACK_LONG_CASTLE_KING_WALK,
                    rook_walk: attacks::BLACK_LONG_CASTLE_ROOK_WALK,
                    king_from: Square::E8,
                    king_to: Square::C8,
                },
            ],
        }
    }
}

fn generate_castle_moves(
    us: Player,
    checkers: Bitboard,
    castling: CastleRights,
    attacks: Bitboard,
    occupied_squares: Bitboard,
    moves: &mut MoveList,
) {
    if checkers.has_any() {
        return;
    }
    for castle in Castle::for_player(us) {
        if castling.contains(castle.right)
            && (attacks & castle.king_walk).is_empty()
            && (occupied_squares & (castle.king_walk | castle.rook_walk)).is_empty()
        {
            unsafe {
                moves.push_unchecked(Move::new(castle.king_from, castle.king_to, None));
            }
        }
    }
}

const fn pawn_push_direction(player: Player) -> Direction {
    match player {
        Player::White => Direction::Up,
        Player::Black => Direction::Down,
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    /// Plays random games and verifies that the incrementally updated state
    /// (most importantly the Zobrist hash, which is maintained through many
    /// hand-written paths in `make_move`: castling, en passant, promotions and
    /// captures) always matches the state recomputed from scratch.
    #[test]
    fn incremental_hash_matches_recomputed_in_random_games() {
        use rand::rngs::SmallRng;
        use rand::{RngExt, SeedableRng};

        const GAMES: u64 = 100;
        const MAX_PLIES: usize = 200;

        for seed in 0..GAMES {
            let mut rng = SmallRng::seed_from_u64(seed);
            let mut position = Position::starting();
            for _ in 0..MAX_PLIES {
                let moves = position.generate_moves();
                if moves.is_empty() || position.halfmove_clock_expired() {
                    break;
                }
                let next_move = moves[rng.random_range(0..moves.len())];
                position.make_move(&next_move);
                assert!(
                    position.is_legal(),
                    "illegal position after {next_move} (seed {seed}): {position}"
                );
                assert_eq!(
                    position.hash(),
                    position.compute_hash(),
                    "incremental hash diverged after {next_move} (seed {seed}): {position}"
                );
            }
        }
    }

    #[test]
    fn starting() {
        let position = Position::starting();
        assert_eq!(
            format!("{:?}", position),
            "Board:\n\
             r n b q k b n r\n\
             p p p p p p p p\n\
             . . . . . . . .\n\
             . . . . . . . .\n\
             . . . . . . . .\n\
             . . . . . . . .\n\
             P P P P P P P P\n\
             R N B Q K B N R\n\
             \n\
             Player to move: White\n\
             Fullmove counter: 1\n\
             En Passant: None\n\
             Castling rights: KQkq\n\
             FEN: rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1\n"
        );
        assert_eq!(
            position.white_pieces.all() | position.black_pieces.all(),
            Rank::Rank1.mask() | Rank::Rank2.mask() | Rank::Rank7.mask() | Rank::Rank8.mask()
        );
        assert_eq!(
            !(position.white_pieces.all() | position.black_pieces.all()),
            Rank::Rank3.mask() | Rank::Rank4.mask() | Rank::Rank5.mask() | Rank::Rank6.mask()
        );
    }
}
