use std::collections::HashSet;
use std::env;
use std::ffi::c_void;
use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::Path;
use std::sync::Mutex;

use byteorder::{LittleEndian, ReadBytesExt};
use clap::Parser;
use flate2::read::GzDecoder;
use tar::Archive;

const Q_THRESHOLD: f32 = 0.6;
const PIECES_THRESHOLD: u32 = 6;
const BOARD_SIZE: usize = 64;
const NUM_PLANES: usize = 12;
const STRUCT_SIZE: usize = 8356;

// Latest Leela Chess Zero training data format:
// https://lczero.org/dev/wiki/training-data-format-versions/
#[repr(C, packed)]
struct V6TrainingData {
    version: u32,
    input_format: u32,
    probabilities: [f32; 1858],
    planes: [u64; 104],
    castling_us_ooo: u8,
    castling_us_oo: u8,
    castling_them_ooo: u8,
    castling_them_oo: u8,
    side_to_move_or_en_passant: u8,
    rule50_count: u8,
    invariance_info: u8,
    dummy: u8,
    root_q: f32,
    best_q: f32,
    root_d: f32,
    best_d: f32,
    root_m: f32,
    best_m: f32,
    plies_left: f32,
    result_q: f32,
    result_d: f32,
    played_q: f32,
    played_d: f32,
    played_m: f32,
    orig_q: f32,
    orig_d: f32,
    orig_m: f32,
    visits: u32,
    played_idx: u16,
    best_idx: u16,
    policy_kld: f32,
    reserved: u32,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    name: String,

    /// Number of times to greet
    #[arg(short, long, default_value_t = 1)]
    count: u8,
}

fn main() {
    let args = Args::parse();
    println!("Hello, world!");
}
