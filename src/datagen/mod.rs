//! Self-play training data generation for the policy and value networks.
//!
//! Each self-play game is played by the [MCTS search](crate::search); at every
//! move the search's visit distribution over the legal moves becomes the policy
//! training target, and the game's eventual outcome becomes the value target
//! (from the perspective of the side to move). Games are independent and are
//! generated in parallel.

use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use rand::rngs::SmallRng;
use rand::{RngExt, SeedableRng};
use rayon::prelude::*;
use shakmaty::Chess;
use shakmaty_syzygy::Tablebase;

use crate::chess::core::Move;
use crate::chess::game::{self, Game};
use crate::chess::position::Position;
use crate::environment::{Environment, GameResult, Player};
use crate::search::mcts;

/// Configuration for a self-play data generation run.
pub struct Config {
    /// Number of self-play games to generate.
    pub games: usize,
    /// MCTS iterations per move.
    pub nodes_per_move: u64,
    /// Number of opening plies played by sampling proportionally to the visit
    /// counts (for opening variety); later moves are played greedily.
    pub exploration_plies: usize,
    /// Hard cap on game length to bound runaway games.
    pub max_plies: usize,
    /// Base RNG seed; game `i` is seeded with `seed ^ i`, so a run is fully
    /// reproducible and independent of scheduling.
    pub seed: u64,
    /// Optional Syzygy tablebase directory for endgame adjudication.
    pub tablebase: Option<PathBuf>,
}

/// A single training example.
pub struct Sample {
    /// The position, in FEN.
    pub fen: String,
    /// The side to move in the position.
    pub side_to_move: Player,
    /// Visit count of each legal move (the policy target).
    pub visits: Vec<(Move, u32)>,
    /// Game outcome from the side-to-move's perspective: `+1` win, `0` draw,
    /// `-1` loss. Filled in once the game ends.
    pub value: f32,
}

impl Sample {
    /// Serializes the sample as `<fen> | <value> | <uci>:<visits> ...`.
    fn write(&self, out: &mut impl Write) -> std::io::Result<()> {
        write!(out, "{} | {:+} | ", self.fen, self.value)?;
        for (i, (mv, visits)) in self.visits.iter().enumerate() {
            if i > 0 {
                write!(out, " ")?;
            }
            write!(out, "{mv}:{visits}")?;
        }
        writeln!(out)
    }
}

/// Generates the self-play games described by `config` and writes the samples
/// to `out`, returning the number of samples written.
///
/// # Errors
///
/// Returns an error if the tablebase can not be loaded or writing fails.
pub fn generate(config: &Config, out: &mut impl Write) -> anyhow::Result<usize> {
    let tablebase = match &config.tablebase {
        Some(directory) => Some(Arc::new(
            game::load_tablebase(directory)
                .with_context(|| format!("loading tablebase from {}", directory.display()))?,
        )),
        None => None,
    };

    let games: Vec<Vec<Sample>> = (0..config.games)
        .into_par_iter()
        .map(|index| {
            play_game(
                config,
                Position::starting(),
                tablebase.clone(),
                config.seed ^ index as u64,
            )
        })
        .collect();

    let mut written = 0;
    for samples in games {
        for sample in samples {
            sample.write(out)?;
            written += 1;
        }
    }
    Ok(written)
}

/// Plays a single self-play game from `root` and returns its training samples.
fn play_game(
    config: &Config,
    root: Position,
    tablebase: Option<Arc<Tablebase<Chess>>>,
    seed: u64,
) -> Vec<Sample> {
    let mut rng = SmallRng::seed_from_u64(seed);
    let mut game = Game::new(root);
    game.set_tablebase(tablebase);
    let limits = mcts::Limits {
        nodes: Some(config.nodes_per_move),
        ..mcts::Limits::default()
    };

    let mut samples = Vec::new();
    while game.result().is_none() && samples.len() < config.max_plies {
        let visits = mcts::policy(&game, &limits);
        if visits.is_empty() {
            break;
        }
        samples.push(Sample {
            fen: game.position().to_string(),
            side_to_move: game.position().us(),
            visits: visits.clone(),
            value: 0.0,
        });
        let explore = samples.len() <= config.exploration_plies;
        game.apply(&pick_move(&visits, explore, &mut rng));
    }

    // A game that is cut off at `max_plies` without a natural result is treated
    // as a draw (value 0), which the samples already default to.
    if let Some(result) = game.result() {
        let outcome = value_of(result);
        for sample in &mut samples {
            sample.value = if sample.side_to_move == game.perspective() {
                outcome
            } else {
                -outcome
            };
        }
    }
    samples
}

/// Picks a move from the visit counts: when `explore` is set, proportionally to
/// the visits (for variety), otherwise the most-visited move.
fn pick_move(visits: &[(Move, u32)], explore: bool, rng: &mut SmallRng) -> Move {
    let total: u32 = visits.iter().map(|(_, count)| count).sum();
    if explore && total > 0 {
        let mut choice = rng.random_range(0..total);
        for (mv, count) in visits {
            if choice < *count {
                return *mv;
            }
            choice -= *count;
        }
    }
    visits
        .iter()
        .max_by_key(|(_, count)| *count)
        .map(|(mv, _)| *mv)
        .expect("visit counts are non-empty")
}

/// Maps a game result to a value target.
fn value_of(result: GameResult) -> f32 {
    match result {
        GameResult::Win => 1.0,
        GameResult::Draw => 0.0,
        GameResult::Loss => -1.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn small_config() -> Config {
        Config {
            games: 3,
            nodes_per_move: 40,
            exploration_plies: 4,
            max_plies: 30,
            seed: 1,
            tablebase: None,
        }
    }

    #[test]
    fn generates_well_formed_samples() {
        let config = small_config();
        let mut out = Vec::new();
        let written = generate(&config, &mut out).unwrap();
        assert!(written > 0, "self-play should produce samples");

        let text = String::from_utf8(out).unwrap();
        assert_eq!(text.lines().count(), written);
        for line in text.lines() {
            let parts: Vec<_> = line.split(" | ").collect();
            assert_eq!(parts.len(), 3, "each line has fen | value | policy");
            let value: f32 = parts[1].parse().unwrap();
            assert!(value == -1.0 || value == 0.0 || value == 1.0);
            assert!(parts[2].split(' ').all(|token| token.contains(':')));
        }
    }

    #[test]
    fn is_reproducible() {
        let config = small_config();
        let mut first = Vec::new();
        let mut second = Vec::new();
        generate(&config, &mut first).unwrap();
        generate(&config, &mut second).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn value_target_reflects_outcome() {
        // White has a forced mate in one (Ra8#). The recorded sample is the
        // position with White to move, so its value target must be a win.
        let root = Position::from_fen("6k1/5ppp/8/8/8/8/8/R5K1 w - - 0 1").unwrap();
        let config = Config {
            games: 1,
            nodes_per_move: 3000,
            exploration_plies: 0,
            max_plies: 4,
            seed: 1,
            tablebase: None,
        };
        let samples = play_game(&config, root, None, 1);
        assert_eq!(samples.len(), 1, "the game ends after the mating move");
        assert_eq!(samples[0].side_to_move, Player::White);
        assert_eq!(samples[0].value, 1.0);
    }
}
