//! Mappings of occupied squares to the attacked squares for each piece. The
//! mappings are pre-calculated where possible to provide an efficient way of
//! generating moves.
//!
//! The implementation uses BMI2 (if available) for performance ([reference]),
//! specifically the PEXT instruction for [PEXT Bitboards].
//!
//! [reference]: https://www.chessprogramming.org/BMI2
//! [PEXT Bitboards]: https://www.chessprogramming.org/BMI2#PEXTBitboards

use super::generated;
use crate::chess::bitboard::{Bitboard, Pieces};
use crate::chess::core::{BOARD_SIZE, Square};
use crate::environment::Player;

pub(super) fn king_attacks(from: Square) -> Bitboard {
    generated::KING_ATTACKS[from as usize]
}

pub(super) fn queen_attacks(from: Square, occupancy: Bitboard) -> Bitboard {
    bishop_attacks(from, occupancy) | rook_attacks(from, occupancy)
}

pub(super) fn rook_attacks(from: Square, occupancy: Bitboard) -> Bitboard {
    generated::ROOK_ATTACKS[generated::ROOK_ATTACK_OFFSETS[from as usize]
        + pext(
            occupancy.bits(),
            generated::ROOK_RELEVANT_OCCUPANCIES[from as usize],
        ) as usize]
}

pub(super) fn bishop_attacks(from: Square, occupancy: Bitboard) -> Bitboard {
    generated::BISHOP_ATTACKS[generated::BISHOP_ATTACK_OFFSETS[from as usize]
        + pext(
            occupancy.bits(),
            generated::BISHOP_RELEVANT_OCCUPANCIES[from as usize],
        ) as usize]
}

pub(super) const fn knight_attacks(square: Square) -> Bitboard {
    generated::KNIGHT_ATTACKS[square as usize]
}

pub(super) const fn pawn_attacks(square: Square, player: Player) -> Bitboard {
    match player {
        Player::White => generated::WHITE_PAWN_ATTACKS[square as usize],
        Player::Black => generated::BLACK_PAWN_ATTACKS[square as usize],
    }
}

pub(super) const fn ray(from: Square, to: Square) -> Bitboard {
    generated::RAYS[(from as usize) * (BOARD_SIZE as usize) + to as usize]
}

pub(super) const fn bishop_ray(from: Square, to: Square) -> Bitboard {
    generated::BISHOP_RAYS[(from as usize) * (BOARD_SIZE as usize) + to as usize]
}

const fn rook_ray(from: Square, to: Square) -> Bitboard {
    generated::ROOK_RAYS[(from as usize) * (BOARD_SIZE as usize) + to as usize]
}

/// Parallel bit extract operation - extracts bits from `a` according to `mask`.
/// Uses BMI2 PEXT instruction when available, falls back to software
/// implementation.
#[inline]
fn pext(a: u64, mask: u64) -> u64 {
    #[cfg(target_arch = "x86_64")]
    {
        if cfg!(target_feature = "bmi2") {
            return unsafe { core::arch::x86_64::_pext_u64(a, mask) };
        }
    }

    // Software fallback implementation
    let mut result = 0u64;
    let mut mask = mask;
    let mut scanning_bit = 1u64;

    while mask != 0 {
        let ls1b = 1u64 << mask.trailing_zeros();
        if (a & ls1b) != 0 {
            result |= scanning_bit;
        }
        mask ^= ls1b;
        scanning_bit <<= 1;
    }
    result
}

#[derive(Debug)]
pub(super) struct AttackInfo {
    pub(super) attacks: Bitboard,
    pub(super) checkers: Bitboard,
    pub(super) pins: Bitboard,
    pub(super) safe_king_squares: Bitboard,
}

impl AttackInfo {
    pub(super) fn new(
        they: Player,
        their: &Pieces,
        king: Square,
        our_occupancy: Bitboard,
        occupancy: Bitboard,
    ) -> Self {
        let mut result = Self {
            attacks: Bitboard::empty(),
            checkers: Bitboard::empty(),
            pins: Bitboard::empty(),
            safe_king_squares: Bitboard::empty(),
        };
        result.safe_king_squares = !our_occupancy & king_attacks(king);
        let occupancy_without_king = occupancy - Bitboard::from(king);
        // King.
        let their_king = their.king.as_square();
        result.attacks |= king_attacks(their_king);
        // Knights.
        for knight in their.knights.iter() {
            let targets = knight_attacks(knight);
            result.attacks |= targets;
            if targets.contains(king) {
                result.checkers.extend(knight);
            }
        }
        // Pawns.
        for pawn in their.pawns.iter() {
            let targets = pawn_attacks(pawn, they);
            result.attacks |= targets;
            if targets.contains(king) {
                result.checkers.extend(pawn);
            }
        }
        // Process sliding pieces (queens, bishops, rooks)
        let sliding_ctx = SlidingPieceContext {
            occupancy,
            occupancy_without_king,
            king,
            our_occupancy,
        };

        // Queens can attack like both rooks and bishops
        for queen in their.queens.iter() {
            process_sliding_piece(&mut result, queen, &sliding_ctx, queen_attacks, ray);
        }
        for bishop in their.bishops.iter() {
            process_sliding_piece(
                &mut result,
                bishop,
                &sliding_ctx,
                bishop_attacks,
                bishop_ray,
            );
        }
        for rook in their.rooks.iter() {
            process_sliding_piece(&mut result, rook, &sliding_ctx, rook_attacks, rook_ray);
        }
        result.safe_king_squares -= result.attacks;
        result
    }
}

/// Context for processing sliding pieces
struct SlidingPieceContext {
    occupancy: Bitboard,
    occupancy_without_king: Bitboard,
    king: Square,
    our_occupancy: Bitboard,
}

/// Helper function to process sliding pieces (queens, bishops, rooks) uniformly
#[inline]
fn process_sliding_piece(
    result: &mut AttackInfo,
    piece_square: Square,
    ctx: &SlidingPieceContext,
    attack_fn: impl Fn(Square, Bitboard) -> Bitboard,
    ray_fn: impl Fn(Square, Square) -> Bitboard,
) {
    let targets = attack_fn(piece_square, ctx.occupancy);
    result.attacks |= targets;

    if targets.contains(ctx.king) {
        result.checkers.extend(piece_square);
        result.safe_king_squares -= attack_fn(piece_square, ctx.occupancy_without_king);
        return; // An attack can be either a check or a (potential) pin, not both
    }

    let attack_ray = ray_fn(piece_square, ctx.king);
    let blocker = (attack_ray & ctx.occupancy) - Bitboard::from(piece_square);

    if blocker.count() == 1 && (blocker & ctx.our_occupancy).has_any() {
        result.pins |= blocker;
    }
}

pub(super) const WHITE_SHORT_CASTLE_KING_WALK: Bitboard =
    Bitboard::from_bits(0x0000_0000_0000_0060);
pub(super) const WHITE_SHORT_CASTLE_ROOK_WALK: Bitboard =
    Bitboard::from_bits(0x0000_0000_0000_0060);
pub(super) const WHITE_LONG_CASTLE_KING_WALK: Bitboard = Bitboard::from_bits(0x0000_0000_0000_000C);
pub(super) const WHITE_LONG_CASTLE_ROOK_WALK: Bitboard = Bitboard::from_bits(0x0000_0000_0000_000E);
pub(super) const BLACK_SHORT_CASTLE_KING_WALK: Bitboard =
    Bitboard::from_bits(0x6000_0000_0000_0000);
pub(super) const BLACK_SHORT_CASTLE_ROOK_WALK: Bitboard =
    Bitboard::from_bits(0x6000_0000_0000_0000);
pub(super) const BLACK_LONG_CASTLE_KING_WALK: Bitboard = Bitboard::from_bits(0x0C00_0000_0000_0000);
pub(super) const BLACK_LONG_CASTLE_ROOK_WALK: Bitboard = Bitboard::from_bits(0x0E00_0000_0000_0000);

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::chess::core::Rank;
    use crate::chess::position::Position;

    #[test]
    fn sliders() {
        let occupancy = Bitboard::from_squares(&[
            Square::F4,
            Square::C4,
            Square::A4,
            Square::B1,
            Square::D5,
            Square::G5,
            Square::G6,
            Square::E8,
            Square::E2,
        ]);
        assert_eq!(
            format!("{:?}", occupancy),
            ". . . . 1 . . .\n\
            . . . . . . . .\n\
            . . . . . . 1 .\n\
            . . . 1 . . 1 .\n\
            1 . 1 . . 1 . .\n\
            . . . . . . . .\n\
            . . . . 1 . . .\n\
            . 1 . . . . . ."
        );
        assert_eq!(
            format!(
                "{:?}",
                Bitboard::from_bits(generated::BISHOP_RELEVANT_OCCUPANCIES[Square::E4 as usize])
            ),
            ". . . . . . . .\n\
            . 1 . . . . . .\n\
            . . 1 . . . 1 .\n\
            . . . 1 . 1 . .\n\
            . . . . . . . .\n\
            . . . 1 . 1 . .\n\
            . . 1 . . . 1 .\n\
            . . . . . . . ."
        );
        let attacks = bishop_attacks(Square::E4, occupancy);
        println!("{:064b}", attacks.bits());
        assert_eq!(
            format!("{:?}", attacks),
            ". . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . 1 .\n\
            . . . 1 . 1 . .\n\
            . . . . . . . .\n\
            . . . 1 . 1 . .\n\
            . . 1 . . . 1 .\n\
            . 1 . . . . . 1"
        );
        assert_eq!(
            format!(
                "{:?}",
                Bitboard::from_bits(generated::ROOK_RELEVANT_OCCUPANCIES[Square::E4 as usize])
            ),
            ". . . . . . . .\n\
            . . . . 1 . . .\n\
            . . . . 1 . . .\n\
            . . . . 1 . . .\n\
            . 1 1 1 . 1 1 .\n\
            . . . . 1 . . .\n\
            . . . . 1 . . .\n\
            . . . . . . . ."
        );
        let attacks = rook_attacks(Square::E4, occupancy);
        println!("{:064b}", attacks.bits());
        assert_eq!(
            format!("{:?}", attacks),
            ". . . . 1 . . .\n\
            . . . . 1 . . .\n\
            . . . . 1 . . .\n\
            . . . . 1 . . .\n\
            . . 1 1 . 1 . .\n\
            . . . . 1 . . .\n\
            . . . . 1 . . .\n\
            . . . . . . . ."
        );
    }

    #[test]
    fn king() {
        assert_eq!(
            format!("{:?}", king_attacks(Square::A1)),
            ". . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            1 1 . . . . . .\n\
            . 1 . . . . . ."
        );
        assert_eq!(
            format!("{:?}", king_attacks(Square::H3)),
            ". . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . 1 1\n\
            . . . . . . 1 .\n\
            . . . . . . 1 1\n\
            . . . . . . . ."
        );
        assert_eq!(
            format!("{:?}", king_attacks(Square::D4)),
            ". . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . 1 1 1 . . .\n\
            . . 1 . 1 . . .\n\
            . . 1 1 1 . . .\n\
            . . . . . . . .\n\
            . . . . . . . ."
        );
        assert_eq!(
            format!("{:?}", king_attacks(Square::F8)),
            ". . . . 1 . 1 .\n\
            . . . . 1 1 1 .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . ."
        );
    }

    #[test]
    fn knight() {
        assert_eq!(
            format!("{:?}", knight_attacks(Square::A1)),
            ". . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . 1 . . . . . .\n\
            . . 1 . . . . .\n\
            . . . . . . . ."
        );
        assert_eq!(
            format!("{:?}", knight_attacks(Square::B1)),
            ". . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            1 . 1 . . . . .\n\
            . . . 1 . . . .\n\
            . . . . . . . ."
        );
        assert_eq!(
            format!("{:?}", knight_attacks(Square::H3)),
            ". . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . 1 .\n\
            . . . . . 1 . .\n\
            . . . . . . . .\n\
            . . . . . 1 . .\n\
            . . . . . . 1 ."
        );
        assert_eq!(
            format!("{:?}", knight_attacks(Square::D4)),
            ". . . . . . . .\n\
            . . . . . . . .\n\
            . . 1 . 1 . . .\n\
            . 1 . . . 1 . .\n\
            . . . . . . . .\n\
            . 1 . . . 1 . .\n\
            . . 1 . 1 . . .\n\
            . . . . . . . ."
        );
        assert_eq!(
            format!("{:?}", knight_attacks(Square::F8)),
            ". . . . . . . .\n\
            . . . 1 . . . 1\n\
            . . . . 1 . 1 .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . ."
        );
    }

    #[test]
    fn pawn() {
        // Pawns can not be on the back ranks, hence the attack maps are empty.
        for square in Rank::Rank1.mask().iter().chain(Rank::Rank8.mask().iter()) {
            assert!(pawn_attacks(square, Player::White).is_empty());
            assert!(pawn_attacks(square, Player::Black).is_empty());
        }
        assert_eq!(
            format!("{:?}", pawn_attacks(Square::A2, Player::White)),
            ". . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . 1 . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . ."
        );
        assert_eq!(
            format!("{:?}", pawn_attacks(Square::A2, Player::Black)),
            ". . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . 1 . . . . . ."
        );
        assert_eq!(
            format!("{:?}", pawn_attacks(Square::D4, Player::White)),
            ". . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . 1 . 1 . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . ."
        );
        assert_eq!(
            format!("{:?}", pawn_attacks(Square::D4, Player::Black)),
            ". . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . 1 . 1 . . .\n\
            . . . . . . . .\n\
            . . . . . . . ."
        );
        assert_eq!(
            format!("{:?}", pawn_attacks(Square::H5, Player::White)),
            ". . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . 1 .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . ."
        );
        assert_eq!(
            format!("{:?}", pawn_attacks(Square::H5, Player::Black)),
            ". . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . 1 .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . ."
        );
    }

    #[test]
    fn rays() {
        // Rays with source == destination don't exist.
        for square_idx in 0..BOARD_SIZE {
            let square = Square::try_from(square_idx).unwrap();
            assert!(ray(square, square).is_empty());
        }
        // Rays don't exist for squares not on the same diagonal or vertical.
        assert!(ray(Square::A1, Square::B3).is_empty());
        assert!(ray(Square::A1, Square::H7).is_empty());
        assert!(ray(Square::B2, Square::H5).is_empty());
        assert!(ray(Square::F2, Square::H8).is_empty());
        assert_eq!(
            format!("{:?}", ray(Square::B3, Square::F7)),
            ". . . . . . . .\n\
            . . . . . . . .\n\
            . . . . 1 . . .\n\
            . . . 1 . . . .\n\
            . . 1 . . . . .\n\
            . 1 . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . ."
        );
        assert_eq!(
            format!("{:?}", ray(Square::F7, Square::B3)),
            ". . . . . . . .\n\
            . . . . . 1 . .\n\
            . . . . 1 . . .\n\
            . . . 1 . . . .\n\
            . . 1 . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . ."
        );
        assert_eq!(
            format!("{:?}", ray(Square::C8, Square::H8)),
            ". . 1 1 1 1 1 .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . ."
        );
        assert_eq!(
            format!("{:?}", ray(Square::H1, Square::H8)),
            ". . . . . . . .\n\
            . . . . . . . 1\n\
            . . . . . . . 1\n\
            . . . . . . . 1\n\
            . . . . . . . 1\n\
            . . . . . . . 1\n\
            . . . . . . . 1\n\
            . . . . . . . 1"
        );
        assert_eq!(
            format!("{:?}", ray(Square::E4, Square::B4)),
            ". . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . 1 1 1 . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . ."
        );
    }

    #[test]
    fn basic_attack_info() {
        let position = Position::try_from("3kn3/3p4/8/6B1/8/6K1/3R4/8 b - - 0 1").unwrap();
        let attacks = position.attack_info();
        assert_eq!(
            format!("{:?}", attacks.attacks),
            ". . . 1 . . . .\n\
            . . . 1 1 . . .\n\
            . . . 1 . 1 . 1\n\
            . . . 1 . . . .\n\
            . . . 1 . 1 1 1\n\
            . . . 1 1 1 . 1\n\
            1 1 1 1 1 1 1 1\n\
            . . . 1 . . . ."
        );
        assert_eq!(
            format!("{:?}", attacks.checkers),
            "\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . 1 .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . ."
        );
        assert_eq!(
            format!("{:?}", attacks.pins),
            ". . . . . . . .\n\
            . . . 1 . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . ."
        );
    }

    #[test]
    fn rich_attack_info() {
        let position =
            Position::try_from("1k3q2/8/8/4PP2/q4K2/3nRBR1/3b1Nr1/5r2 w - - 0 1").unwrap();
        let attacks = position.attack_info();
        assert_eq!(
            format!("{:?}", attacks.attacks),
            "1 1 1 1 1 . 1 1\n\
            1 1 1 1 1 1 1 .\n\
            1 . 1 1 . 1 . 1\n\
            1 1 1 . 1 1 . .\n\
            . 1 1 1 1 1 . .\n\
            1 1 1 . 1 . 1 .\n\
            1 1 1 . . 1 . 1\n\
            1 1 1 1 1 . 1 1"
        );
        assert_eq!(
            format!("{:?}", attacks.checkers),
            ". . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            1 . . . . . . .\n\
            . . . 1 . . . .\n\
            . . . . . . . .\n\
            . . . . . . . ."
        );
        assert_eq!(
            format!("{:?}", attacks.pins),
            ". . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . 1 . .\n\
            . . . . . . . .\n\
            . . . . 1 . . .\n\
            . . . . . . . .\n\
            . . . . . . . ."
        );
    }

    #[test]
    fn complicated_attack_info() {
        let position =
            Position::try_from("2r3r1/3p3k/1p3pp1/1B5P/5P2/2P1pqP1/PP4KP/3R4 w - - 0 34").unwrap();
        let attacks = position.attack_info();
        assert_eq!(
            format!("{:?}", attacks.checkers),
            ". . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . 1 . .\n\
            . . . . . . . .\n\
            . . . . . . . ."
        );
        assert_eq!(
            format!("{:?}", attacks.safe_king_squares),
            ". . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . 1 . 1\n\
            . . . . . . . .\n\
            . . . . . . 1 ."
        );
    }
}
