use anyhow::bail;
use clap::Parser;
use flate2::read::GzDecoder;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use std::path::PathBuf;
use tar::Archive;

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

fn extract_training_samples(archive_path: &Path) -> io::Result<Vec<V6TrainingData>> {
    let file = File::open(archive_path).unwrap();
    let mut archive = Archive::new(file);

    let mut samples = Vec::new();

    for entry in archive.entries()? {
        let mut entry = entry?;
        let mut data = Vec::new();
        if entry.header().entry_type().is_dir() {
            continue;
        }
        entry.read_to_end(&mut data)?;

        let decompressed_data = decompress_gzip_data(&data)?;

        assert!(
            decompressed_data.len() % STRUCT_SIZE == 0,
            "Invalid archive size"
        );
        let num_structs = decompressed_data.len() / STRUCT_SIZE;
        for i in 0..num_structs {
            let start = i * STRUCT_SIZE;
            let end = start + STRUCT_SIZE;
            let sample = V6TrainingData::from_bytes(&decompressed_data[start..end]);
            samples.push(sample);
        }
    }

    Ok(samples)
}

fn process_archive(archive_path: &Path, output_path: &Path) -> io::Result<usize> {
    println!("Processing {:?}", archive_path);

    let samples = extract_training_samples(archive_path)?;

    // let (features, targets): (Vec<_>, Vec<_>) = samples.into_par_iter()
    //     .filter(|sample| !should_filter(sample))
    //     .map(|sample| (extract_features(&sample), sample.best_q))
    //     .unzip();

    // let features: HashSet<_> = features.into_iter().collect();
    // let targets: Vec<f32> = targets.into_iter().collect();

    // let num_samples = features.len();

    // let features: Vec<_> = features.into_iter().collect();
    // save_to_file(features_path, &features)?;

    // println!("Extracted {:} samples", num_samples);

    // Ok(num_samples)

    Ok(samples.len())
}

/// The data should be downloaded from <https://storage.lczero.org/files/training_data/test80/>
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to .tar archive with the raw lc0 training data.
    archive_path: PathBuf,
    /// Path to the directory where the extracted data will be stored.
    output_dir: PathBuf,
    #[arg(long, default_value_t = 0.05)]
    threshold: f32,
    /// Filter positions where the best move is capturing a piece.
    #[arg(long, default_value_t = true)]
    filter_captures: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    if !std::fs::metadata(&args.archive_path)?.is_file() {
        bail!("{:?} is not a file", &args.archive_path);
    }
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

    println!(
        "Extracting data from {:?} to {:?}",
        &args.archive_path, &output_path
    );
    println!(
        "Filtering |q| <= {:.2}, captures: {}",
        args.threshold, args.filter_captures
    );

    let total_samples = process_archive(Path::new(&args.archive_path), &output_path)?;
    println!("Extracted {:} samples", total_samples);

    Ok(())
}
