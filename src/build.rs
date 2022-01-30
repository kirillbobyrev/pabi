// TODO: Describe why this is needed and what the details are.

use core::arch::x86_64::_pdep_u64;
use std::error::Error;
use std::fmt::Write;
use std::path::Path;
use std::{env, fs, process};

const BOARD_WIDTH: i32 = 8;

const BISHOP_ATTACK_DIRECTIONS: [(i32, i32); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
const ROOK_ATTACK_DIRECTIONS: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

fn to_square(column: i32, row: i32) -> u64 {
    1 << (row * BOARD_WIDTH + column)
}

fn is_within_board(column: i32, row: i32) -> bool {
    0 <= column && column < BOARD_WIDTH && 0 <= row && row < BOARD_WIDTH
}

fn generate_file(filename: &str, contents: &str) {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join(filename);
    fs::write(&dest_path, contents).unwrap();
}

fn serialize_bitboard_array(array: &[u64]) -> Result<String, Box<dyn Error>> {
    let mut result = String::new();
    result.push('[');
    for element in array {
        write!(result, "Bitboard::from_bits({element}), \n")?;
    }
    result.push(']');
    Ok(result)
}

fn serialize_array(array: &[u64]) -> Result<String, Box<dyn Error>> {
    let mut result = String::new();
    result.push('[');
    for element in array {
        write!(result, "{element}, \n")?;
    }
    result.push(']');
    Ok(result)
}

fn generate_attacks(
    source_column: i32,
    source_row: i32,
    directions: &[(i32, i32); 4],
    occupancy_mask: u64,
) -> u64 {
    let mut result = 0u64;
    for (d_column, d_row) in directions {
        let mut column = source_column + d_column;
        let mut row = source_row + d_row;
        loop {
            if !is_within_board(column + d_column, row + d_row) {
                break;
            }
            let attacked_square = to_square(column, row);
            result |= attacked_square;
            if (occupancy_mask & attacked_square) != 0 {
                break;
            }
            column += d_column;
            row += d_row;
        }
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
fn generate_table(identifier: &str, directions: &[(i32, i32); 4]) -> Result<usize, Box<dyn Error>> {
    let mut attacks = vec![];
    let mut relevant_occupancies = vec![];
    let mut table_offsets = vec![];
    let mut offset = 0;
    for source_column in 0..BOARD_WIDTH {
        for source_row in 0..BOARD_WIDTH {
            let mut relevant_occupancy_mask = 0u64;
            for (d_column, d_row) in directions {
                let mut column = source_column + d_column;
                let mut row = source_row + d_row;
                loop {
                    if !is_within_board(column + d_column, row + d_row) {
                        break;
                    }
                    relevant_occupancy_mask |= to_square(column, row);
                    column += d_column;
                    row += d_row;
                }
            }
            table_offsets.push(offset);
            let indices = (1 << relevant_occupancy_mask.count_ones()) as u64;
            for index in 0..indices {
                let occupancies = unsafe { _pdep_u64(index, relevant_occupancy_mask) };
                attacks.push(generate_attacks(
                    source_column,
                    source_row,
                    directions,
                    occupancies,
                ));
            }
            offset += indices;
            relevant_occupancies.push(relevant_occupancy_mask);
        }
    }
    generate_file(
        &(identifier.to_owned() + "_attacks"),
        &serialize_bitboard_array(&attacks)?,
    );
    generate_file(
        &(identifier.to_owned() + "_occupancies"),
        &serialize_bitboard_array(&relevant_occupancies)?,
    );
    generate_file(
        &(identifier.to_owned() + "_offsets"),
        &serialize_array(&table_offsets)?,
    );
    Ok(offset.try_into()?)
}

fn generate_attack_tables() -> Result<(), Box<dyn Error>> {
    assert_eq!(generate_table("bishop", &BISHOP_ATTACK_DIRECTIONS)?, 5248);
    assert_eq!(generate_table("rook", &ROOK_ATTACK_DIRECTIONS)?, 102400);
    Ok(())
}

// TODO: This can fail at several levels: be more principled about it.
fn git_revision_hash() -> String {
    let output = process::Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .unwrap()
        .stdout;
    std::str::from_utf8(&output).unwrap().trim().to_string()
}

fn generate_version() -> Result<(), Box<dyn Error>> {
    let mut version = String::new();
    write!(
        version,
        "{} ({})\n",
        clap::crate_version!(),
        git_revision_hash()
    )?;
    write!(version, "Build type: {}\n", env::var("PROFILE").unwrap())?;
    write!(version, "Target: {}", env::var("TARGET").unwrap())?;
    generate_file("version", &version);
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    generate_version()?;
    generate_attack_tables()?;
    Ok(())
}
