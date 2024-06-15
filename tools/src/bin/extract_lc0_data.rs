use std::io::{self, BufRead, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use anyhow::bail;
use clap::Parser;
use flate2::read::GzDecoder;
use tar::Archive;

const BOARD_SIZE: usize = 64;
const NUM_PLANES: usize = 12;
const TABLEBASE_MIN_PIECES: u32 = 6;
const STRUCT_SIZE: usize = 8356;

/// Extract training data from the Leela Chess Zero data archive.
///
/// Archives can be found at <https://storage.lczero.org/files/training_data/test80/>
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to .tar archive with the raw lc0 training data.
    // TODO: Make this a directory and iterate over all files.
    archive_path: PathBuf,
    /// Path to the directory where the extracted data will be stored.
    output_dir: PathBuf,
    /// Maximum number of samples to extract.
    #[arg(long)]
    limit: Option<usize>,
    /// Only positions with |eval| <= q_threshold will be kept. Practically, distinguishing between
    /// very high evals shouldn't be very important, because if an engine reaches that position, it
    /// is likely to be winning/losing anyway.
    ///
    /// Q-value to CP conversion formula:
    ///
    /// cp = 660.6 * q / (1 - 0.9751875 * q^10)
    ///
    /// q = 0.9 corresponds to cp = 900
    #[arg(long, default_value_t = 0.9)]
    q_threshold: f32,
    /// Remove positions with less than min_pieces pieces on the board. This is useful, because
    /// most tournaments allow using 6 man tablebases.
    #[arg(long, default_value_t = 6)]
    min_pieces: u8,
    /// Drop positions where the best move is capturing a piece. It is likely to
    /// be a tactical position and should benefit from search and not from the
    /// eval.
    #[arg(long, default_value_t = true)]
    filter_captures: bool,
    /// Drop positions where the best move is giving a check. It is likely to be
    /// a tactical position/attack.
    #[arg(long, default_value_t = true)]
    filter_checks: bool,
    /// Drop positions where the best move is promotion.
    #[arg(long, default_value_t = false)]
    filter_promotions: bool,
    /// Remove duplicate positions by using Zobrist hashing.
    #[arg(long, default_value_t = true)]
    deduplicate: bool,
}

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

impl V6TrainingData {
    fn from_bytes(bytes: &[u8]) -> Self {
        assert_eq!(bytes.len(), STRUCT_SIZE);
        unsafe { std::ptr::read(bytes.as_ptr() as *const V6TrainingData) }
    }
}

fn decompress_gzip_data(data: &[u8]) -> io::Result<Vec<u8>> {
    let mut decoder = GzDecoder::new(data);
    let mut decompressed_data = Vec::new();
    decoder.read_to_end(&mut decompressed_data)?;
    Ok(decompressed_data)
}

fn extract_training_samples(archive: impl BufRead) -> io::Result<Vec<V6TrainingData>> {
    let mut archive = Archive::new(archive);

    let mut samples = Vec::new();

    for entry in archive.entries()? {
        let mut entry = entry?;
        if !entry.header().entry_type().is_file()
            || entry.header().path()?.extension().is_none()
            || entry.header().path()?.extension().unwrap() != "gz"
        {
            continue;
        }
        let mut data = Vec::new();
        entry.read_to_end(&mut data)?;
        let decompressed_data = decompress_gzip_data(&data)?;

        assert!(
            decompressed_data.len() % STRUCT_SIZE == 0,
            "Invalid archive size"
        );
        let num_samples = decompressed_data.len() / STRUCT_SIZE;
        for i in 0..num_samples {
            let (start, end) = (i * STRUCT_SIZE, (i + 1) * STRUCT_SIZE);
            let num_samples = decompressed_data.len() / STRUCT_SIZE;
            for i in 0..num_samples {
                let (start, end) = (i * STRUCT_SIZE, (i + 1) * STRUCT_SIZE);
                let sample = V6TrainingData::from_bytes(&decompressed_data[start..end]);
                samples.push(sample);
            }
        }
    }

    Ok(samples)
}

// TODO: Flip the planes.
fn extract_planes(sample: &V6TrainingData) -> Vec<u64> {
    vec![
        // Our pieces.
        sample.planes[0],
        sample.planes[1],
        sample.planes[2],
        sample.planes[3],
        sample.planes[4],
        sample.planes[5],
        // Opponent pieces.
        sample.planes[6],
        sample.planes[7],
        sample.planes[8],
        sample.planes[9],
        sample.planes[10],
        sample.planes[11],
    ]
}

fn keep_sample(sample: &V6TrainingData, q_threshold: f32, filter_captures: bool) -> bool {
    assert!(sample.version == 6 && sample.input_format == 1);
    if sample.invariance_info & (1 << 6) != 0 {
        return false;
    }
    if sample.best_q.abs() > q_threshold {
        return false;
    }

    let planes = extract_planes(sample);
    if planes.iter().map(|plane| plane.count_ones()).sum::<u32>() <= TABLEBASE_MIN_PIECES {
        return false;
    }

    // TODO: Filter the capturing moves, positions in check and stalemates.

    let board = pabi::chess::position::Position::empty();
    let best_move =
        pabi::chess::core::Move::from_uci(pabi_tools::IDX_TO_MOVE[sample.best_idx as usize]);
    // TODO: Just check the target square manually?
    // TODO: Set the bitboards...

    // for &color in &[Color::White, Color::Black] {
    // for &piece in &[
    // Piece::Pawn,
    // Piece::Knight,
    // Piece::Bishop,
    // Piece::Rook,
    // Piece::Queen,
    // Piece::King,
    // ] {
    // let plane = features[plane_id];
    // for square in 0..BOARD_SIZE {
    // if (plane & (1 << square)) != 0 {
    // let corrected_square = (square & !7) + (7 - (square % 8));
    // board.set_piece_at(
    // Square::new(corrected_square as u8),
    // Some(Piece::new(piece, color)),
    // );
    // }
    // }
    // plane_id += 1;
    // }
    // }
    //
    // if board.is_check() || board.is_stalemate() {
    // return true;
    // }
    //
    // let best_move = Move::new(sample.best_idx as u8, sample.best_idx as u8,
    // None); if board.is_capture(best_move) || board.gives_check(best_move) ||
    // board.is_castling(best_move) { return true;
    // }

    true
}

fn serialize_sample<W: Write>(sample: &V6TrainingData, out: &mut BufWriter<W>) -> io::Result<()> {
    // TODO: Correct the planes.
    let planes = extract_planes(sample);

    let bytes = unsafe {
        std::slice::from_raw_parts(
            planes.as_ptr() as *const u8,
            planes.len() * std::mem::size_of::<u64>(),
        )
    };
    out.write_all(bytes)?;

    let target = sample.best_q;
    out.write_all(&target.to_le_bytes())
}

fn process_archive<W: Write>(
    archive: impl BufRead,
    output: &mut BufWriter<W>,
    q_threshold: f32,
    filter_captures: bool,
) -> io::Result<usize> {
    let mut num_samples = 0;

    for sample in extract_training_samples(archive)?
        .into_iter()
        .filter(|sample| keep_sample(sample, q_threshold, filter_captures))
    {
        serialize_sample(&sample, output)?;
        num_samples += 1
    }

    Ok(num_samples)
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if !std::fs::metadata(&args.archive_path)?.is_file() {
        bail!("{:?} is not a file", &args.archive_path);
    }
    let archive = std::fs::File::open(Path::new(&args.archive_path))?;

    let archive = std::fs::File::open(Path::new(&args.archive_path))?;

    if !std::fs::metadata(&args.output_dir)?.is_dir() {
        bail!("{:?} is not a directory", &args.output_dir);
    }
    let output_filename = Path::new(&args.archive_path)
        .with_extension("bin")
        .file_name()
        .unwrap()
        .to_owned();
    let output_path = args.output_dir.join(output_filename);
    if output_path.exists() {
        bail!("{:?} already exists", &output_path);
    }
    let out_file = std::fs::File::create_new(&output_path)?;

    println!(
        "Extracting data from {:?} to {:?}",
        &args.archive_path, &output_path
    );
    println!(
        "Filtering |q| <= {:.2}, filtering out captures: {}",
        args.q_threshold, args.filter_captures
    );

    let total_samples = process_archive(
        io::BufReader::new(archive),
        &mut io::BufWriter::new(out_file),
        args.q_threshold,
        args.filter_captures,
    )?;
    println!("Extracted {:} samples", total_samples);

    Ok(())
}
