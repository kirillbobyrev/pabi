//! Command-line front-end for self-play training data generation.

use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::PathBuf;

use clap::Parser;
use pabi::datagen::{self, Config};

/// Generates training data for the policy and value networks through self-play.
#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    /// Number of self-play games to generate.
    #[arg(long, default_value_t = 100)]
    games: usize,
    /// MCTS iterations per move.
    #[arg(long, default_value_t = 800)]
    nodes: u64,
    /// Opening plies played by sampling the visit counts (for variety).
    #[arg(long, default_value_t = 30)]
    exploration_plies: usize,
    /// Hard cap on game length.
    #[arg(long, default_value_t = 400)]
    max_plies: usize,
    /// Base RNG seed.
    #[arg(long, default_value_t = 0)]
    seed: u64,
    /// Directory with Syzygy tablebases for endgame adjudication.
    #[arg(long)]
    tablebase: Option<PathBuf>,
    /// Output file; writes to stdout if omitted.
    #[arg(long, short)]
    output: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = Config {
        games: cli.games,
        nodes_per_move: cli.nodes,
        exploration_plies: cli.exploration_plies,
        max_plies: cli.max_plies,
        seed: cli.seed,
        tablebase: cli.tablebase,
    };

    let written = match cli.output {
        Some(path) => {
            let mut out = BufWriter::new(File::create(&path)?);
            let written = datagen::generate(&config, &mut out)?;
            out.flush()?;
            written
        }
        None => {
            let stdout = io::stdout();
            let mut out = BufWriter::new(stdout.lock());
            datagen::generate(&config, &mut out)?
        }
    };

    eprintln!("generated {written} samples from {} games", config.games);
    Ok(())
}
