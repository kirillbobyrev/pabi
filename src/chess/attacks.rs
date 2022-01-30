//! Mappings of occupied squares to the attacked squares for each piece. The
//! mappings are pre-calculated where possible to provide an efficient way of
//! generating moves.

use crate::chess::bitboard::Bitboard;
use crate::chess::core::{BOARD_SIZE, BOARD_WIDTH};

// TODO: Document PEXT bitboards.
const BISHOP_ATTACKS_COUNT: usize = 5248;
const BISHOP_ATTACKS: [Bitboard; BISHOP_ATTACKS_COUNT] =
    generate_table::<BISHOP_ATTACKS_COUNT>(&BISHOP_ATTACK_DIRECTIONS).0;
const BISHOP_RELEVANT_OCCUPANCIES: [Bitboard; BOARD_SIZE as usize] =
    generate_table::<BISHOP_ATTACKS_COUNT>(&BISHOP_ATTACK_DIRECTIONS).1;
const BISHOP_ATTACK_OFFSETS: [usize; BOARD_SIZE as usize] =
    generate_table::<BISHOP_ATTACKS_COUNT>(&BISHOP_ATTACK_DIRECTIONS).2;
const ROOK_ATTACKS_COUNT: usize = 102400;
const ROOK_ATTACKS: [Bitboard; ROOK_ATTACKS_COUNT] =
    generate_table::<ROOK_ATTACKS_COUNT>(&ROOK_ATTACK_DIRECTIONS).0;
const ROOK_RELEVANT_OCCUPANCIES: [Bitboard; BOARD_SIZE as usize] =
    generate_table::<ROOK_ATTACKS_COUNT>(&ROOK_ATTACK_DIRECTIONS).1;
const ROOK_ATTACK_OFFSETS: [usize; BOARD_SIZE as usize] =
    generate_table::<ROOK_ATTACKS_COUNT>(&ROOK_ATTACK_DIRECTIONS).2;

const BISHOP_ATTACK_DIRECTIONS: [(i32, i32); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
const ROOK_ATTACK_DIRECTIONS: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

const fn to_index(column: i32, row: i32) -> usize {
    (row * BOARD_WIDTH as i32 + column) as usize
}

const fn to_square(column: i32, row: i32) -> u64 {
    1 << to_index(column, row)
}

const fn is_within_board(column: i32, row: i32) -> bool {
    0 <= column && column < BOARD_WIDTH as i32 && 0 <= row && row < BOARD_WIDTH as i32
}

const fn generate_attacks(
    source_column: i32,
    source_row: i32,
    directions: &[(i32, i32); 4],
    occupancy_mask: u64,
) -> u64 {
    let mut result = 0u64;
    let mut delta_idx = 0;
    while delta_idx < directions.len() {
        let (d_column, d_row) = directions[delta_idx];
        let mut column = source_column + d_column;
        let mut row = source_row + d_row;
        while is_within_board(column + d_column, row + d_row) {
            let attacked_square = to_square(column, row);
            result |= attacked_square;
            if (occupancy_mask & attacked_square) != 0 {
                break;
            }
            column += d_column;
            row += d_row;
        }
        delta_idx += 1;
    }
    result
}

// TODO: Document why I'm doing this.
const fn pdep(index: u64, mask: u64) -> u64 {
    let mut result = 0u64;
    let mut mask = mask;
    let mut scanning_bit = 0u64;
    while scanning_bit < 64 {
        if mask == 0 {
            break;
        }
        let ls1b = 1u64 << mask.trailing_zeros();
        if (index & (1 << scanning_bit)) != 0 {
            result |= ls1b;
        }
        mask ^= ls1b;
        scanning_bit += 1;
    }
    result
}

// Generates PEXT magic table for rooks and bishops and returns the table size
// for correctness check. Returned table size should be:
//
// - 5248 for bishop
// - 102400 for rook
//
// The tables are used in src/chess/attacks.rs, see documentation there for more
// information.
// TODO: Looking at this, the only problem with moving generation to compile_time might be looping
const fn generate_table<const SIZE: usize>(
    directions: &[(i32, i32); 4],
) -> (
    [Bitboard; SIZE],
    [Bitboard; BOARD_SIZE as usize],
    [usize; BOARD_SIZE as usize],
) {
    let mut attacks: [Bitboard; SIZE] = [Bitboard::empty(); SIZE];
    let mut relevant_occupancies: [Bitboard; BOARD_SIZE as usize] =
        [Bitboard::empty(); BOARD_SIZE as usize];
    let mut attack_offsets: [usize; BOARD_SIZE as usize] = [0; BOARD_SIZE as usize];
    let mut offset = 0;
    let mut source_column = 0i32;
    while source_column < BOARD_WIDTH as i32 {
        let mut source_row = 0i32;
        while source_row < BOARD_WIDTH as i32 {
            let square_index = to_index(source_column, source_row);
            let mut relevant_occupancy_mask = 0u64;
            relevant_occupancies[square_index] = Bitboard::from_bits(relevant_occupancy_mask);
            attack_offsets[square_index] = offset;
            let mut delta_idx = 0;
            while delta_idx < directions.len() {
                let (d_column, d_row) = directions[delta_idx];
                let mut column = source_column + d_column;
                let mut row = source_row + d_row;
                while is_within_board(column + d_column, row + d_row) {
                    relevant_occupancy_mask |= to_square(column, row);
                    column += d_column;
                    row += d_row;
                }
                delta_idx += 1;
            }
            let max_index = 1 << relevant_occupancy_mask.count_ones();
            let mut index = 0;
            while index < max_index {
                let occupancies = pdep(index, relevant_occupancy_mask);
                attacks[square_index] = Bitboard::from_bits(generate_attacks(
                    source_column,
                    source_row,
                    directions,
                    occupancies,
                ));
                attacks[0] = Bitboard::from_bits(0);
                index += 1;
            }
            offset += max_index as usize;
            source_row += 1;
        }
        source_column += 1;
    }
    (attacks, relevant_occupancies, attack_offsets)
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
