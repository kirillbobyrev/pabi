//! Retrieves information about the version of the engine from Git and the build
//! environment. This information is then written to a file in the output
//! directory and can be accessed at runtime by the engine.

fn generate_file(filename: &str, contents: &str) {
    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let dest_path = std::path::Path::new(&out_dir).join(filename);
    std::fs::write(dest_path, contents).unwrap();
}

// TODO: Add AVX, SSE, potentially MKL and other Candle features support.
fn generate_build_info() {
    let features = format!(
        "{}bmi2",
        if cfg!(target_feature = "bmi2") {
            "+"
        } else {
            "-"
        }
    );
    generate_file("features", &features);
}

type ZobristKey = u64;

fn generate_zobrist_keys() {
    const NUM_COLORS: usize = 2;
    const NUM_PIECES: usize = 6;
    const NUM_SQUARES: usize = 64;

    let mut rng = rand::thread_rng();

    let piece_keys: [ZobristKey; NUM_COLORS * NUM_PIECES * NUM_SQUARES] =
        std::array::from_fn(|_| rand::Rng::gen(&mut rng));
    generate_file(
        &format!("pieces_zobrist_keys"),
        &format!("{:?}", piece_keys),
    );

    let en_passant_keys: [ZobristKey; 8] = std::array::from_fn(|_| rand::Rng::gen(&mut rng));
    generate_file("en_passant_zobrist_keys", &format!("{:?}", en_passant_keys));
}

fn main() -> shadow_rs::SdResult<()> {
    generate_zobrist_keys();
    generate_build_info();
    shadow_rs::new()
}
