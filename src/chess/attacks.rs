//! Mappings of occupied squares to the attacked squares for each piece. The
//! mappings are pre-calculated where possible to provide an efficient way of
//! generating moves.

use crate::chess::bitboard::Bitboard;
use crate::chess::core::{Square, BOARD_SIZE};

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

pub(super) fn get_bishop_attacks(square: Square, occupancy: Bitboard) -> Bitboard {
    BISHOP_ATTACKS[BISHOP_OFFSETS[square as usize]
        + pext(
            occupancy.bits(),
            BISHOP_RELEVANT_OCCUPANCIES[square as usize],
        ) as usize]
}

pub(super) fn get_rook_attacks(square: Square, occupancy: Bitboard) -> Bitboard {
    ROOK_ATTACKS[ROOK_OFFSETS[square as usize]
        + pext(occupancy.bits(), ROOK_RELEVANT_OCCUPANCIES[square as usize]) as usize]
}

// Pre-calculated attacks of a knight from each square.
pub(super) const KNIGHT_ATTACKS: [Bitboard; BOARD_SIZE as usize] = [
    Bitboard::from_bits(0x0000_0000_0002_0400),
    Bitboard::from_bits(0x0000_0000_0005_0800),
    Bitboard::from_bits(0x0000_0000_000A_1100),
    Bitboard::from_bits(0x0000_0000_0014_2200),
    Bitboard::from_bits(0x0000_0000_0028_4400),
    Bitboard::from_bits(0x0000_0000_0050_8800),
    Bitboard::from_bits(0x0000_0000_00A0_1000),
    Bitboard::from_bits(0x0000_0000_0040_2000),
    Bitboard::from_bits(0x0000_0000_0204_0004),
    Bitboard::from_bits(0x0000_0000_0508_0008),
    Bitboard::from_bits(0x0000_0000_0A11_0011),
    Bitboard::from_bits(0x0000_0000_1422_0022),
    Bitboard::from_bits(0x0000_0000_2844_0044),
    Bitboard::from_bits(0x0000_0000_5088_0088),
    Bitboard::from_bits(0x0000_0000_A010_0010),
    Bitboard::from_bits(0x0000_0000_4020_0020),
    Bitboard::from_bits(0x0000_0002_0400_0402),
    Bitboard::from_bits(0x0000_0005_0800_0805),
    Bitboard::from_bits(0x0000_000A_1100_110A),
    Bitboard::from_bits(0x0000_0014_2200_2214),
    Bitboard::from_bits(0x0000_0028_4400_4428),
    Bitboard::from_bits(0x0000_0050_8800_8850),
    Bitboard::from_bits(0x0000_00A0_1000_10A0),
    Bitboard::from_bits(0x0000_0040_2000_2040),
    Bitboard::from_bits(0x0000_0204_0004_0200),
    Bitboard::from_bits(0x0000_0508_0008_0500),
    Bitboard::from_bits(0x0000_0A11_0011_0A00),
    Bitboard::from_bits(0x0000_1422_0022_1400),
    Bitboard::from_bits(0x0000_2844_0044_2800),
    Bitboard::from_bits(0x0000_5088_0088_5000),
    Bitboard::from_bits(0x0000_A010_0010_A000),
    Bitboard::from_bits(0x0000_4020_0020_4000),
    Bitboard::from_bits(0x0002_0400_0402_0000),
    Bitboard::from_bits(0x0005_0800_0805_0000),
    Bitboard::from_bits(0x000A_1100_110A_0000),
    Bitboard::from_bits(0x0014_2200_2214_0000),
    Bitboard::from_bits(0x0028_4400_4428_0000),
    Bitboard::from_bits(0x0050_8800_8850_0000),
    Bitboard::from_bits(0x00A0_1000_10A0_0000),
    Bitboard::from_bits(0x0040_2000_2040_0000),
    Bitboard::from_bits(0x0204_0004_0200_0000),
    Bitboard::from_bits(0x0508_0008_0500_0000),
    Bitboard::from_bits(0x0A11_0011_0A00_0000),
    Bitboard::from_bits(0x1422_0022_1400_0000),
    Bitboard::from_bits(0x2844_0044_2800_0000),
    Bitboard::from_bits(0x5088_0088_5000_0000),
    Bitboard::from_bits(0xA010_0010_A000_0000),
    Bitboard::from_bits(0x4020_0020_4000_0000),
    Bitboard::from_bits(0x0400_0402_0000_0000),
    Bitboard::from_bits(0x0800_0805_0000_0000),
    Bitboard::from_bits(0x1100_110A_0000_0000),
    Bitboard::from_bits(0x2200_2214_0000_0000),
    Bitboard::from_bits(0x4400_4428_0000_0000),
    Bitboard::from_bits(0x8800_8850_0000_0000),
    Bitboard::from_bits(0x1000_10A0_0000_0000),
    Bitboard::from_bits(0x2000_2040_0000_0000),
    Bitboard::from_bits(0x0004_0200_0000_0000),
    Bitboard::from_bits(0x0008_0500_0000_0000),
    Bitboard::from_bits(0x0011_0A00_0000_0000),
    Bitboard::from_bits(0x0022_1400_0000_0000),
    Bitboard::from_bits(0x0044_2800_0000_0000),
    Bitboard::from_bits(0x0088_5000_0000_0000),
    Bitboard::from_bits(0x0010_A000_0000_0000),
    Bitboard::from_bits(0x0020_4000_0000_0000),
];

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use super::{Bitboard, Square};
    use crate::chess::attacks::{
        get_bishop_attacks,
        get_rook_attacks,
        BISHOP_RELEVANT_OCCUPANCIES,
        ROOK_RELEVANT_OCCUPANCIES,
    };

    #[test]
    fn bishop_attacks() {
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
        let attacks = get_bishop_attacks(Square::E4, occupancy);
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
        let attacks = get_rook_attacks(Square::E4, occupancy);
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
}
