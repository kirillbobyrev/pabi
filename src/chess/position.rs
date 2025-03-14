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

    /// Returns Zobrist hash of the position.
    #[must_use]
    pub fn hash(&self) -> zobrist::Key {
        self.hash
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
                match Piece::try_from(symbol) {
                    Ok(piece) => {
                        let pieces = match piece.player {
                            Player::White => &mut white_pieces,
                            Player::Black => &mut black_pieces,
                        };
                        let square = Square::new(file.try_into()?, rank);
                        *pieces.bitboard_for_mut(piece.kind) |= Bitboard::from(square);
                    }
                    Err(e) => return Err(e),
                }
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
        // TODO: Try caching more e.g. all()s? Benchmark to confirm that this is an
        // improvement.
        let (us, them) = (self.us(), self.them());
        let (our_pieces, their_pieces) = (self.pieces(us), self.pieces(them));
        let king: Square = our_pieces.king.as_square();
        let (our_occupancy, their_occupancy) = (our_pieces.all(), their_pieces.all());
        let occupied_squares = our_occupancy | their_occupancy;
        let their_or_empty = !our_occupancy;
        let attack_info =
            attacks::AttackInfo::new(them, their_pieces, king, our_occupancy, occupied_squares);
        // Moving the king to safety is always a valid move.
        generate_king_moves(king, attack_info.safe_king_squares, &mut moves);
        // If there are checks, the moves are restricted to resolving them.
        let blocking_ray = match attack_info.checkers.count() {
            0 => Bitboard::full(),
            // There are two ways of getting out of check:
            //
            // - Moving king to safety (calculated above)
            // - Blocking the checker or capturing it
            //
            // The former is calculated above, the latter is dealt with below.
            1 => {
                let checker: Square = attack_info.checkers.as_square();
                let ray = attacks::ray(checker, king);
                if ray.is_empty() {
                    // This means the checker is a knight: capture is the only
                    // way left to resolve this check.
                    attack_info.checkers
                } else {
                    // Checker is a sliding piece: both capturing and blocking
                    // resolves the check.
                    ray
                }
            }
            // Double checks can only be evaded by the king moves to safety: no
            // need to consider other moves.
            2 => return moves,
            _ => unreachable!("checks can't be given by more than two pieces at once"),
        };
        generate_knight_moves(
            our_pieces.knights,
            their_or_empty,
            attack_info.pins,
            blocking_ray,
            &mut moves,
        );
        generate_rook_moves(
            our_pieces.rooks | our_pieces.queens,
            occupied_squares,
            their_or_empty,
            blocking_ray,
            attack_info.pins,
            king,
            &mut moves,
        );
        generate_bishop_moves(
            our_pieces.bishops | our_pieces.queens,
            occupied_squares,
            their_or_empty,
            blocking_ray,
            attack_info.pins,
            king,
            &mut moves,
        );
        generate_pawn_moves(
            our_pieces.pawns,
            us,
            them,
            their_pieces,
            their_occupancy,
            their_or_empty,
            blocking_ray,
            attack_info.pins,
            attack_info.checkers,
            king,
            self.en_passant_square,
            occupied_squares,
            &mut moves,
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

        // Increment halfmove clock early: it will be reset on capture or pawn
        // push.
        self.halfmove_clock += 1;

        self.update_castling_rights(next_move);

        self.handle_capture(next_move);
        self.make_pawn_move(next_move);
        self.make_king_move(next_move);
        self.make_regular_move(next_move);

        if self.side_to_move == Player::Black {
            self.fullmove_counter += 1;
        }

        self.side_to_move = !self.side_to_move;
    }

    fn update_castling_rights(&mut self, next_move: &Move) {
        if self.castling.contains(CastleRights::WHITE_SHORT)
            && (next_move.from() == Square::E1
                || next_move.from() == Square::H1
                || next_move.to() == Square::H1)
        {
            self.castling.remove(CastleRights::WHITE_SHORT);
            self.hash ^= generated::WHITE_CAN_CASTLE_SHORT;
        }
        if self.castling.contains(CastleRights::WHITE_LONG)
            && (next_move.from() == Square::E1
                || next_move.from() == Square::A1
                || next_move.to() == Square::A1)
        {
            self.castling.remove(CastleRights::WHITE_LONG);
            self.hash ^= generated::WHITE_CAN_CASTLE_LONG;
        }
        if self.castling.contains(CastleRights::BLACK_SHORT)
            && (next_move.from() == Square::E8
                || next_move.from() == Square::H8
                || next_move.to() == Square::H8)
        {
            self.castling.remove(CastleRights::BLACK_SHORT);
            self.hash ^= generated::BLACK_CAN_CASTLE_SHORT;
        }
        if self.castling.contains(CastleRights::BLACK_LONG)
            && (next_move.from() == Square::E8
                || next_move.from() == Square::A8
                || next_move.to() == Square::A8)
        {
            self.castling.remove(CastleRights::BLACK_LONG);
            self.hash ^= generated::BLACK_CAN_CASTLE_LONG;
        }
    }

    fn handle_capture(&mut self, next_move: &Move) {
        let their_pieces = match self.side_to_move {
            Player::White => &mut self.black_pieces,
            Player::Black => &mut self.white_pieces,
        };

        if their_pieces.all().contains(next_move.to()) {
            // Capturing a piece resets the clock.
            self.halfmove_clock = 0;

            let square = next_move.to();

            for (piece, kind) in [
                (&mut their_pieces.queens, PieceKind::Queen),
                (&mut their_pieces.rooks, PieceKind::Rook),
                (&mut their_pieces.bishops, PieceKind::Bishop),
                (&mut their_pieces.knights, PieceKind::Knight),
                (&mut their_pieces.pawns, PieceKind::Pawn),
            ] {
                if piece.contains(square) {
                    piece.clear(square);
                    self.hash ^= generated::get_piece_key(
                        Piece {
                            player: !self.side_to_move,
                            kind,
                        },
                        square,
                    );
                    break;
                }
            }
        }
    }

    fn make_pawn_move(&mut self, next_move: &Move) -> bool {
        let (our_pieces, their_pieces) = match self.side_to_move {
            Player::White => (&mut self.white_pieces, &mut self.black_pieces),
            Player::Black => (&mut self.black_pieces, &mut self.white_pieces),
        };

        let previous_en_passant = self.en_passant_square;
        self.en_passant_square = None;

        if !our_pieces.pawns.contains(next_move.from()) {
            return false;
        }

        // Pawn move resets the 50 halfmove rule clock.
        self.halfmove_clock = 0;

        // Check en passant.
        if let Some(en_passant_square) = previous_en_passant {
            if next_move.to() == en_passant_square {
                let captured_pawn = Square::new(next_move.to().file(), next_move.from().rank());
                their_pieces.pawns.clear(captured_pawn);
                self.hash ^= generated::get_piece_key(
                    Piece {
                        player: !self.side_to_move,
                        kind: PieceKind::Pawn,
                    },
                    captured_pawn,
                );
            }
        }

        our_pieces.pawns.clear(next_move.from());
        self.hash ^= generated::get_piece_key(
            Piece {
                player: self.side_to_move,
                kind: PieceKind::Pawn,
            },
            next_move.from(),
        );

        // Check promotions.
        // TODO: Debug assertions to make sure the promotion is valid.
        if let Some(promotion) = next_move.promotion() {
            match promotion {
                Promotion::Queen => {
                    our_pieces.queens.extend(next_move.to());
                    self.hash ^= generated::get_piece_key(
                        Piece {
                            player: self.side_to_move,
                            kind: PieceKind::Queen,
                        },
                        next_move.to(),
                    );
                }
                Promotion::Rook => {
                    our_pieces.rooks.extend(next_move.to());
                    self.hash ^= generated::get_piece_key(
                        Piece {
                            player: self.side_to_move,
                            kind: PieceKind::Rook,
                        },
                        next_move.to(),
                    );
                }
                Promotion::Bishop => {
                    our_pieces.bishops.extend(next_move.to());
                    self.hash ^= generated::get_piece_key(
                        Piece {
                            player: self.side_to_move,
                            kind: PieceKind::Bishop,
                        },
                        next_move.to(),
                    );
                }
                Promotion::Knight => {
                    our_pieces.knights.extend(next_move.to());
                    self.hash ^= generated::get_piece_key(
                        Piece {
                            player: self.side_to_move,
                            kind: PieceKind::Knight,
                        },
                        next_move.to(),
                    );
                }
            };
            return true;
        }

        our_pieces.pawns.extend(next_move.to());
        self.hash ^= generated::get_piece_key(
            Piece {
                player: self.side_to_move,
                kind: PieceKind::Pawn,
            },
            next_move.to(),
        );

        let single_push_square = next_move
            .from()
            .shift(pawn_push_direction(self.side_to_move))
            .unwrap();

        // Double push creates en passant square.
        if next_move.from().rank() == Rank::pawns_starting(self.side_to_move)
                && next_move.from().file() == next_move.to().file()
                && single_push_square != next_move.to()
                // Technically, this is not correct: https://github.com/jhlywa/chess.js/issues/294
                && (their_pieces.pawns & attacks::pawn_attacks(single_push_square, self.side_to_move)).has_any()
        {
            self.en_passant_square = Some(single_push_square);
            self.hash ^= generated::EN_PASSANT_FILES[single_push_square.file() as usize];
        }

        true
    }

    /// Castle or regular king move.
    // TODO: Merge with the other castling rights handler.
    fn make_king_move(&mut self, next_move: &Move) -> bool {
        let our_pieces = match self.side_to_move {
            Player::White => &mut self.white_pieces,
            Player::Black => &mut self.black_pieces,
        };

        if !our_pieces.king.contains(next_move.from()) {
            return false;
        }

        let backrank = Rank::backrank(self.side_to_move);

        // Check if the move is castling.
        if next_move.from().rank() == backrank
            && next_move.to().rank() == backrank
            && next_move.from().file() == File::E
        {
            if next_move.to().file() == File::G {
                let from = Square::new(File::H, backrank);
                our_pieces.rooks.clear(from);
                self.hash ^= generated::get_piece_key(
                    Piece {
                        player: self.side_to_move,
                        kind: PieceKind::Rook,
                    },
                    from,
                );
                let to = Square::new(File::F, backrank);
                our_pieces.rooks.extend(to);
                self.hash ^= generated::get_piece_key(
                    Piece {
                        player: self.side_to_move,
                        kind: PieceKind::Rook,
                    },
                    to,
                );
            } else if next_move.to().file() == File::C {
                let from = Square::new(File::A, backrank);
                our_pieces.rooks.clear(from);
                self.hash ^= generated::get_piece_key(
                    Piece {
                        player: self.side_to_move,
                        kind: PieceKind::Rook,
                    },
                    from,
                );
                let to = Square::new(File::D, backrank);
                our_pieces.rooks.extend(to);
                self.hash ^= generated::get_piece_key(
                    Piece {
                        player: self.side_to_move,
                        kind: PieceKind::Rook,
                    },
                    to,
                );
            }
        }

        our_pieces.king.clear(next_move.from());
        self.hash ^= generated::get_piece_key(
            Piece {
                player: self.side_to_move,
                kind: PieceKind::King,
            },
            next_move.from(),
        );
        our_pieces.king.extend(next_move.to());
        self.hash ^= generated::get_piece_key(
            Piece {
                player: self.side_to_move,
                kind: PieceKind::King,
            },
            next_move.to(),
        );

        true
    }

    fn make_regular_move(&mut self, next_move: &Move) {
        let our_pieces = match self.side_to_move {
            Player::White => &mut self.white_pieces,
            Player::Black => &mut self.black_pieces,
        };

        for (bitboard, kind) in [
            (&mut our_pieces.queens, PieceKind::Queen),
            (&mut our_pieces.rooks, PieceKind::Rook),
            (&mut our_pieces.bishops, PieceKind::Bishop),
            (&mut our_pieces.knights, PieceKind::Knight),
        ] {
            if bitboard.contains(next_move.from()) {
                bitboard.clear(next_move.from());
                self.hash ^= generated::get_piece_key(
                    Piece {
                        player: self.side_to_move,
                        kind,
                    },
                    next_move.from(),
                );
                bitboard.extend(next_move.to());
                self.hash ^= generated::get_piece_key(
                    Piece {
                        player: self.side_to_move,
                        kind,
                    },
                    next_move.to(),
                );
                return;
            }
        }
    }

    #[must_use]
    pub fn in_check(&self) -> bool {
        // TODO: Computing this is expensive. Cache/check for attacks on king
        // separately?
        let attack_info = attacks::AttackInfo::new(
            self.them(),
            self.pieces(self.them()),
            self.pieces(self.us()).king.as_square(),
            self.occupancy(self.us()),
            self.occupied_squares(),
        );
        attack_info.checkers.has_any()
    }

    /// Returns true if 50-move rule draw is in effect.
    #[must_use]
    pub fn halfmove_clock_expired(&self) -> bool {
        self.halfmove_clock >= 100
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
        write!(f, " {} ", &self.side_to_move)?;
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

        writeln!(f, "Player to move: {:?}", &self.side_to_move)?;
        writeln!(f, "Fullmove counter: {:?}", &self.fullmove_counter)?;
        writeln!(f, "En Passant: {:?}", &self.en_passant_square)?;
        // bitflags' default fmt::Debug implementation is not very convenient:
        // dump FEN instead.
        writeln!(f, "Castling rights: {}", &self.castling)?;
        writeln!(f, "FEN: {}", &self)?;

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
    if position.white_pieces.king.count() != 1 {
        bail!(
            "expected 1 white king, got {}",
            position.white_pieces.king.count()
        )
    }
    if position.black_pieces.king.count() != 1 {
        bail!(
            "expected 1 black king, got {}",
            position.black_pieces.king.count()
        )
    }
    if position.white_pieces.pawns.count() > 8 {
        bail!(
            "expected <= 8 white pawns, got {}",
            position.white_pieces.pawns.count()
        )
    }
    if position.black_pieces.pawns.count() > 8 {
        bail!(
            "expected <= 8 black pawns, got {}",
            position.black_pieces.pawns.count()
        )
    }
    if ((position.white_pieces.pawns | position.black_pieces.pawns)
        & (Rank::Rank1.mask() | Rank::Rank8.mask()))
    .has_any()
    {
        bail!("pawns can not be placed on backranks")
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

fn generate_knight_moves(
    knights: Bitboard,
    their_or_empty: Bitboard,
    pins: Bitboard,
    blocking_ray: Bitboard,
    moves: &mut MoveList,
) {
    // When a knight is pinned, it can not move at all because it can't stay on
    // the same horizontal, vertical or diagonal.
    for from in (knights - pins).iter() {
        let targets = attacks::knight_attacks(from) & their_or_empty & blocking_ray;
        for to in targets.iter() {
            unsafe {
                moves.push_unchecked(Move::new(from, to, None));
            }
        }
    }
}

fn generate_rook_moves(
    rooks: Bitboard,
    occupied_squares: Bitboard,
    their_or_empty: Bitboard,
    blocking_ray: Bitboard,
    pins: Bitboard,
    king: Square,
    moves: &mut MoveList,
) {
    for from in rooks.iter() {
        let targets = attacks::rook_attacks(from, occupied_squares) & their_or_empty & blocking_ray;
        for to in targets.iter() {
            // TODO: This block is repeated several times; abstract it out.
            if pins.contains(from) && (attacks::ray(from, king) & attacks::ray(to, king)).is_empty()
            {
                continue;
            }
            unsafe { moves.push_unchecked(Move::new(from, to, None)) }
        }
    }
}

fn generate_bishop_moves(
    bishops: Bitboard,
    occupied_squares: Bitboard,
    their_or_empty: Bitboard,
    blocking_ray: Bitboard,
    pins: Bitboard,
    king: Square,
    moves: &mut MoveList,
) {
    for from in bishops.iter() {
        let targets =
            attacks::bishop_attacks(from, occupied_squares) & their_or_empty & blocking_ray;
        for to in targets.iter() {
            // TODO: This block is repeated several times; abstract it out.
            if pins.contains(from) && (attacks::ray(from, king) & attacks::ray(to, king)).is_empty()
            {
                continue;
            }
            unsafe { moves.push_unchecked(Move::new(from, to, None)) }
        }
    }
}

fn generate_pawn_moves(
    pawns: Bitboard,
    us: Player,
    them: Player,
    their_pieces: &Pieces,
    their_occupancy: Bitboard,
    their_or_empty: Bitboard,
    blocking_ray: Bitboard,
    pins: Bitboard,
    checkers: Bitboard,
    king: Square,
    en_passant_square: Option<Square>,
    occupied_squares: Bitboard,
    moves: &mut MoveList,
) {
    // TODO: Get rid of the branch: AND pawns getting to the promotion rank and the
    // rest.
    for from in pawns.iter() {
        let targets =
            (attacks::pawn_attacks(from, us) & their_occupancy) & their_or_empty & blocking_ray;
        for to in targets.iter() {
            // TODO: This block is repeated several times; abstract it out.
            if pins.contains(from) && (attacks::ray(from, king) & attacks::ray(to, king)).is_empty()
            {
                continue;
            }
            match to.rank() {
                Rank::Rank1 | Rank::Rank8 => unsafe {
                    moves.push_unchecked(Move::new(from, to, Some(Promotion::Queen)));
                    moves.push_unchecked(Move::new(from, to, Some(Promotion::Rook)));
                    moves.push_unchecked(Move::new(from, to, Some(Promotion::Bishop)));
                    moves.push_unchecked(Move::new(from, to, Some(Promotion::Knight)));
                },
                _ => unsafe { moves.push_unchecked(Move::new(from, to, None)) },
            }
        }
    }
    // Generate en passant moves.
    if let Some(en_passant_square) = en_passant_square {
        let en_passant_pawn = en_passant_square.shift(pawn_push_direction(them)).unwrap();
        // Check if capturing en passant resolves the check.
        let candidate_pawns = attacks::pawn_attacks(en_passant_square, them) & pawns;
        if checkers.contains(en_passant_pawn) {
            for our_pawn in candidate_pawns.iter() {
                if pins.contains(our_pawn) {
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
                if (attacks::queen_attacks(king, occupancy_after_capture) & their_pieces.queens)
                    .is_empty()
                    && (attacks::rook_attacks(king, occupancy_after_capture) & their_pieces.rooks)
                        .is_empty()
                    && (attacks::bishop_attacks(king, occupancy_after_capture)
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
    let push_direction = pawn_push_direction(us);
    let pawn_pushes = pawns.shift(push_direction) - occupied_squares;
    let original_squares = pawn_pushes.shift(push_direction.opposite());
    let add_pawn_moves = |moves: &mut MoveList, from, to: Square| {
        // TODO: This is probably better with self.side_to_move.opponent().backrank()
        // but might be slower.
        match to.rank() {
            Rank::Rank8 | Rank::Rank1 => unsafe {
                moves.push_unchecked(Move::new(from, to, Some(Promotion::Queen)));
                moves.push_unchecked(Move::new(from, to, Some(Promotion::Rook)));
                moves.push_unchecked(Move::new(from, to, Some(Promotion::Bishop)));
                moves.push_unchecked(Move::new(from, to, Some(Promotion::Knight)));
            },
            _ => unsafe { moves.push_unchecked(Move::new(from, to, None)) },
        }
    };
    for (from, to) in std::iter::zip(original_squares.iter(), pawn_pushes.iter()) {
        if !blocking_ray.contains(to) {
            continue;
        }
        if pins.contains(from) && (attacks::ray(from, king) & attacks::ray(to, king)).is_empty() {
            continue;
        }
        add_pawn_moves(moves, from, to);
    }
    // Double pawn pushes.
    // TODO: Come up with a better name for it.
    let third_rank = Rank::pawns_starting(us).mask().shift(push_direction);
    let double_pushes = (pawn_pushes & third_rank).shift(push_direction) - occupied_squares;
    let original_squares = double_pushes
        .shift(push_direction.opposite())
        .shift(push_direction.opposite());
    // Double pawn pushes are never promoting.
    for (from, to) in std::iter::zip(original_squares.iter(), double_pushes.iter()) {
        if !blocking_ray.contains(to) {
            continue;
        }
        if pins.contains(from) && (attacks::ray(from, king) & attacks::ray(to, king)).is_empty() {
            continue;
        }
        unsafe {
            moves.push_unchecked(Move::new(from, to, None));
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
    // TODO: Generalize castling to FCR.
    // TODO: In FCR we should check if the rook is pinned or not.
    if checkers.is_empty() {
        match us {
            Player::White => {
                if castling.contains(CastleRights::WHITE_SHORT)
                    && (attacks & attacks::WHITE_SHORT_CASTLE_KING_WALK).is_empty()
                    && (occupied_squares
                        & (attacks::WHITE_SHORT_CASTLE_KING_WALK
                            | attacks::WHITE_SHORT_CASTLE_ROOK_WALK))
                        .is_empty()
                {
                    unsafe {
                        moves.push_unchecked(Move::new(Square::E1, Square::G1, None));
                    }
                }
                if castling.contains(CastleRights::WHITE_LONG)
                    && (attacks & attacks::WHITE_LONG_CASTLE_KING_WALK).is_empty()
                    && (occupied_squares
                        & (attacks::WHITE_LONG_CASTLE_KING_WALK
                            | attacks::WHITE_LONG_CASTLE_ROOK_WALK))
                        .is_empty()
                {
                    unsafe {
                        moves.push_unchecked(Move::new(Square::E1, Square::C1, None));
                    }
                }
            }
            Player::Black => {
                if castling.contains(CastleRights::BLACK_SHORT)
                    && (attacks & attacks::BLACK_SHORT_CASTLE_KING_WALK).is_empty()
                    && (occupied_squares
                        & (attacks::BLACK_SHORT_CASTLE_KING_WALK
                            | attacks::BLACK_SHORT_CASTLE_ROOK_WALK))
                        .is_empty()
                {
                    unsafe {
                        moves.push_unchecked(Move::new(Square::E8, Square::G8, None));
                    }
                }
                if castling.contains(CastleRights::BLACK_LONG)
                    && (attacks & attacks::BLACK_LONG_CASTLE_KING_WALK).is_empty()
                    && (occupied_squares
                        & (attacks::BLACK_LONG_CASTLE_KING_WALK
                            | attacks::BLACK_LONG_CASTLE_ROOK_WALK))
                        .is_empty()
                {
                    unsafe {
                        moves.push_unchecked(Move::new(Square::E8, Square::C8, None));
                    }
                }
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
