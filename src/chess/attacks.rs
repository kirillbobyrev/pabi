//! Mappings of occupied squares to the attacked squares for each piece. The
//! mappings are pre-calculated where possible to provide an efficient way of
//! generating moves.
//!
//! The implementation uses BMI2 (if available) for performance ([reference]),
//! specifically the PEXT instruction for [PEXT Bitboards].
//!
//! [reference]: https://www.chessprogramming.org/BMI2
//! [PEXT Bitboards]: https://www.chessprogramming.org/BMI2#PEXTBitboards

// TODO: This code is probably by far the least appealing in the project.
// Refactor it and make it nicer.

use crate::chess::bitboard::{Bitboard, Pieces};
use crate::chess::core::{Player, Square, BOARD_SIZE};

use super::generated;

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

// TODO: Document.
fn pext(a: u64, mask: u64) -> u64 {
    #[cfg(target_arch = "x86_64")]
    {
        if cfg!(target_feature = "bmi2") {
            return unsafe { core::arch::x86_64::_pext_u64(a, mask) };
        }
    }
    // Fallback.
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
    // TODO: Get rid of the XRays.
    pub(super) xrays: Bitboard,
    pub(super) safe_king_squares: Bitboard,
}

impl AttackInfo {
    // TODO: Handle each piece separately.
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
            xrays: Bitboard::empty(),
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
        // Queens.
        // TODO: Sliders repeat each other. Pull this into a function.
        for queen in their.queens.iter() {
            let targets = queen_attacks(queen, occupancy);
            result.attacks |= targets;
            if targets.contains(king) {
                result.checkers.extend(queen);
                result.safe_king_squares -= queen_attacks(queen, occupancy_without_king);
                // An attack can be either a check or a (potential) pin, not
                // both.
                continue;
            }
            let attack_ray = ray(queen, king);
            let blocker = (attack_ray & occupancy) - Bitboard::from(queen);
            if blocker.count() == 1 {
                if (blocker & our_occupancy).has_any() {
                    result.pins |= blocker;
                } else {
                    result.xrays |= blocker;
                }
            }
        }
        for bishop in their.bishops.iter() {
            let targets = bishop_attacks(bishop, occupancy);
            result.attacks |= targets;
            if targets.contains(king) {
                result.checkers.extend(bishop);
                result.safe_king_squares -= bishop_attacks(bishop, occupancy_without_king);
                // An attack can be either a check or a (potential) pin, not
                // both.
                continue;
            }
            let attack_ray = bishop_ray(bishop, king);
            let blocker = (attack_ray & occupancy) - Bitboard::from(bishop);
            if blocker.count() == 1 {
                if (blocker & our_occupancy).has_any() {
                    result.pins |= blocker;
                } else {
                    result.xrays |= blocker;
                }
            }
        }
        for rook in their.rooks.iter() {
            let targets = rook_attacks(rook, occupancy);
            result.attacks |= targets;
            if targets.contains(king) {
                result.checkers.extend(rook);
                result.safe_king_squares -= rook_attacks(rook, occupancy_without_king);
                // An attack can be either a check or a (potential) pin, not
                // both.
                continue;
            }
            let attack_ray = rook_ray(rook, king);
            let blocker = (attack_ray & occupancy) - Bitboard::from(rook);
            if blocker.count() == 1 {
                if (blocker & our_occupancy).has_any() {
                    result.pins |= blocker;
                } else {
                    result.xrays |= blocker;
                }
            }
        }
        result.safe_king_squares -= result.attacks;
        result
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
mod test {
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
        for square in Rank::One.mask().iter().chain(Rank::Eight.mask().iter()) {
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
        assert!(attacks.xrays.is_empty());
    }

    #[test]
    fn xrays() {
        let position = Position::try_from("b6k/8/8/3p4/8/8/8/7K w - - 0 1").unwrap();
        let attacks = position.attack_info();
        assert_eq!(
            format!("{:?}", attacks.attacks),
            ". . . . . . 1 .\n\
            . 1 . . . . 1 1\n\
            . . 1 . . . . .\n\
            . . . 1 . . . .\n\
            . . 1 . 1 . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . ."
        );
        assert!(attacks.checkers.is_empty());
        assert!(attacks.pins.is_empty());
        assert_eq!(
            format!("{:?}", attacks.xrays),
            ". . . . . . . .\n\
            . . . . . . . .\n\
            . . . . . . . .\n\
            . . . 1 . . . .\n\
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
        assert!(attacks.xrays.is_empty());
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
        assert!(attacks.xrays.is_empty());
    }
}
