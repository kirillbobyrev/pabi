//! Mappings of occupied squares to the attacked squares for each piece. The
//! mappings are pre-calculated where possible to provide an efficient way of
//! generating moves.
// TODO: This code is probably by far the less appealing in the project.
// Refactor it and make it nicer.

use crate::chess::bitboard::Bitboard;
use crate::chess::core::{Player, Square, BOARD_SIZE};

// TODO: Here and elsewhere: get_unchecked instead.
pub(super) fn king_attacks(from: Square) -> Bitboard {
    KING_ATTACKS[from as usize]
}

pub(super) fn queen_attacks(from: Square, occupancy: Bitboard) -> Bitboard {
    bishop_attacks(from, occupancy) | rook_attacks(from, occupancy)
}

pub(super) fn rook_attacks(from: Square, occupancy: Bitboard) -> Bitboard {
    ROOK_ATTACKS[ROOK_OFFSETS[from as usize]
        + pext(occupancy.bits(), ROOK_RELEVANT_OCCUPANCIES[from as usize]) as usize]
}

pub(super) fn bishop_attacks(from: Square, occupancy: Bitboard) -> Bitboard {
    BISHOP_ATTACKS[BISHOP_OFFSETS[from as usize]
        + pext(occupancy.bits(), BISHOP_RELEVANT_OCCUPANCIES[from as usize]) as usize]
}

pub(super) fn knight_attacks(square: Square) -> Bitboard {
    KNIGHT_ATTACKS[square as usize]
}

pub(super) fn pawn_attacks(square: Square, player: Player) -> Bitboard {
    match player {
        Player::White => WHITE_PAWN_ATTACKS[square as usize],
        Player::Black => BLACK_PAWN_ATTACKS[square as usize],
    }
}

pub(super) fn ray(from: Square, to: Square) -> Bitboard {
    RAYS[(from as usize) * (BOARD_SIZE as usize) + to as usize]
}

fn pext(a: u64, mask: u64) -> u64 {
    if cfg!(target_feature = "bmi2") {
        unsafe { core::arch::x86_64::_pext_u64(a, mask) }
    } else {
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
}

// Generated in build.rs.
// TODO: Document PEXT bitboards.
const BISHOP_ATTACKS_COUNT: usize = 5248;
const BISHOP_ATTACKS: [Bitboard; BISHOP_ATTACKS_COUNT] =
    include!(concat!(env!("OUT_DIR"), "/bishop_attacks"));
const ROOK_ATTACKS_COUNT: usize = 102_400;
const ROOK_ATTACKS: [Bitboard; ROOK_ATTACKS_COUNT] =
    include!(concat!(env!("OUT_DIR"), "/rook_attacks"));
const BISHOP_RELEVANT_OCCUPANCIES: [u64; BOARD_SIZE as usize] =
    include!(concat!(env!("OUT_DIR"), "/bishop_occupancies"));
const ROOK_RELEVANT_OCCUPANCIES: [u64; BOARD_SIZE as usize] =
    include!(concat!(env!("OUT_DIR"), "/rook_occupancies"));
const BISHOP_OFFSETS: [usize; BOARD_SIZE as usize] =
    include!(concat!(env!("OUT_DIR"), "/bishop_offsets"));
const ROOK_OFFSETS: [usize; BOARD_SIZE as usize] =
    include!(concat!(env!("OUT_DIR"), "/rook_offsets"));

include!("generated/knight_attacks.rs");
include!("generated/king_attacks.rs");
include!("generated/white_pawn_attacks.rs");
include!("generated/black_pawn_attacks.rs");

include!("generated/rays.rs");

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use strum::IntoEnumIterator;

    use super::*;
    use crate::chess::core::Rank;

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
                Bitboard::from_bits(BISHOP_RELEVANT_OCCUPANCIES[Square::E4 as usize])
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
                Bitboard::from_bits(ROOK_RELEVANT_OCCUPANCIES[Square::E4 as usize])
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
        for square in Square::iter() {
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
}
