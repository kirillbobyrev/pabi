//! Implementation of the AlphaZero-style [Monte Carlo Tree Search]:
//!
//! 1. Selection: starting from the root, descend by picking the most promising
//!    child ([`crate::search::policy`]) until a leaf is reached.
//! 2. Expansion: generate the leaf's children and assign prior probabilities.
//! 3. Evaluation: score the leaf with the value function
//!    ([`crate::evaluation`]); terminal positions get their exact game value.
//!    There are no random playouts, matching the AlphaZero variant of MCTS.
//! 4. Backup: propagate the value up the path, flipping the sign at each level
//!    (the value of a position for one player is the negation of its value for
//!    the opponent).
//!
//! [Monte Carlo Tree Search]: https://en.wikipedia.org/wiki/Monte_Carlo_tree_search

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use super::policy;
use super::tree::{NodeId, ROOT_ID, Tree};
use crate::chess::core::Move;
use crate::chess::position::Position;
use crate::chess::zobrist::Key;
use crate::evaluation;

/// Exploration constant ($c_{puct}$ in the AlphaZero paper).
const CPUCT: f32 = 1.5;

/// How often (in search iterations) the time limit is checked and search info
/// is potentially reported.
const CHECK_INTERVAL: u64 = 128;

/// Maximum length of the principal variation to report.
const MAX_PV_LENGTH: usize = 16;

/// Limits controlling when the search stops. Multiple limits can be active at
/// once: the search stops as soon as any of them is reached. Regardless of the
/// limits, the search can always be stopped externally through the stop flag.
#[derive(Debug, Clone, Default)]
pub struct Limits {
    /// Hard limit on the search wall clock time.
    pub move_time: Option<Duration>,
    /// Limit on the number of search iterations (playouts).
    pub nodes: Option<u64>,
    /// If set, the search runs until stopped externally, ignoring other
    /// limits.
    pub infinite: bool,
}

/// Final result of a search.
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// The move the engine considers best. `None` if the root position has no
    /// legal moves.
    pub best_move: Option<Move>,
    /// Number of search iterations performed.
    pub nodes: u64,
}

/// Searches the position and returns the best move.
///
/// `history` should contain the Zobrist hashes of all positions of the game so
/// far (including the root position): it is used to score repetitions as
/// draws.
///
/// `on_info` is called with UCI `info` lines as the search progresses.
pub fn search(
    root_position: &Position,
    history: &[Key],
    limits: &Limits,
    stop: &AtomicBool,
    mut on_info: impl FnMut(&str),
) -> SearchResult {
    let root_moves = root_position.generate_moves();
    if root_moves.is_empty() {
        return SearchResult {
            best_move: None,
            nodes: 0,
        };
    }

    let mut tree = Tree::new();
    #[allow(clippy::cast_precision_loss, reason = "the number of moves is small")]
    let prior = 1.0 / root_moves.len() as f32;
    for next_move in &root_moves {
        let _ = tree.add_child(ROOT_ID, *next_move, prior);
    }
    tree.node_mut(ROOT_ID).expanded = true;

    let start = Instant::now();
    let mut iterations = 0u64;
    let mut max_depth = 0usize;
    let mut last_report = Instant::now();

    loop {
        if stop.load(Ordering::Relaxed) {
            break;
        }
        if !limits.infinite {
            if let Some(nodes) = limits.nodes
                && iterations >= nodes
            {
                break;
            }
            if let Some(move_time) = limits.move_time
                && start.elapsed() >= move_time
            {
                break;
            }
        }

        let depth = simulate(&mut tree, root_position, history);
        max_depth = max_depth.max(depth);
        iterations += 1;

        if iterations.is_multiple_of(CHECK_INTERVAL)
            && last_report.elapsed() >= Duration::from_secs(1)
        {
            on_info(&info_line(&tree, iterations, max_depth, start));
            last_report = Instant::now();
        }
    }

    on_info(&info_line(&tree, iterations, max_depth, start));

    SearchResult {
        best_move: Some(best_move(&tree)),
        nodes: iterations,
    }
}

/// Runs a single search iteration (selection, expansion, evaluation, backup)
/// and returns the depth reached.
fn simulate(tree: &mut Tree, root_position: &Position, history: &[Key]) -> usize {
    let mut position = root_position.clone();
    let mut path = vec![ROOT_ID];
    let mut path_hashes: Vec<Key> = Vec::new();
    let mut node_id = ROOT_ID;

    // Selection: descend until a terminal or unexpanded node.
    while tree.node(node_id).terminal.is_none() && tree.node(node_id).expanded {
        let child_id = policy::select_child(tree, node_id, CPUCT);
        let action = tree
            .node(child_id)
            .action
            .expect("non-root nodes always have an action");
        position.make_move(&action);
        path_hashes.push(position.hash());
        path.push(child_id);
        node_id = child_id;
    }

    // Expansion and evaluation.
    let value = match tree.node(node_id).terminal {
        Some(value) => value,
        None => expand_and_evaluate(tree, node_id, &position, &path_hashes, history),
    };

    backup(tree, &path, value);
    path.len() - 1
}

/// Expands the leaf and returns its value from the perspective of the player
/// to move at the leaf. Terminal values are cached in the node: the path from
/// the root to a node is unique, so the terminal status (including draws by
/// repetition) is deterministic.
fn expand_and_evaluate(
    tree: &mut Tree,
    node_id: NodeId,
    position: &Position,
    path_hashes: &[Key],
    history: &[Key],
) -> f64 {
    let moves = position.generate_moves();
    if moves.is_empty() {
        // Checkmate is the worst outcome for the player to move; stalemate is
        // a draw.
        let value = if position.in_check() { -1.0 } else { 0.0 };
        tree.node_mut(node_id).terminal = Some(value);
        return value;
    }

    let is_root = path_hashes.is_empty();
    if !is_root {
        if position.halfmove_clock_expired() || position.is_insufficient_material() {
            tree.node_mut(node_id).terminal = Some(0.0);
            return 0.0;
        }
        // Score any repetition of an earlier position (in the game or in the
        // current search path) as an immediate draw.
        let current = *path_hashes.last().expect("non-root path is not empty");
        let earlier = &path_hashes[..path_hashes.len() - 1];
        if earlier.contains(&current) || history.contains(&current) {
            tree.node_mut(node_id).terminal = Some(0.0);
            return 0.0;
        }
    }

    #[allow(clippy::cast_precision_loss, reason = "the number of moves is small")]
    let prior = 1.0 / moves.len() as f32;
    for next_move in &moves {
        let _ = tree.add_child(node_id, *next_move, prior);
    }
    tree.node_mut(node_id).expanded = true;

    value_from_centipawns(evaluation::evaluate(position))
}

/// Propagates the value up the path. `value` is from the perspective of the
/// player to move at the leaf (the last node in the path); node statistics
/// store values from the parent's perspective, so the sign flips at each
/// level.
fn backup(tree: &mut Tree, path: &[NodeId], leaf_value: f64) {
    let mut value = leaf_value;
    for &node_id in path.iter().rev() {
        let node = tree.node_mut(node_id);
        node.visits += 1;
        node.total_value -= value;
        value = -value;
    }
}

/// Returns the root child with the most visits, breaking ties by mean value.
fn best_move(tree: &Tree) -> Move {
    let root = tree.node(ROOT_ID);
    let best = root
        .children
        .iter()
        .max_by(|&&a, &&b| {
            let (a, b) = (tree.node(a), tree.node(b));
            a.visits
                .cmp(&b.visits)
                .then(a.mean_value().total_cmp(&b.mean_value()))
        })
        .expect("root is always expanded");
    tree.node(*best).action.expect("root children have actions")
}

/// Maps a centipawn score to the value domain `[-1, 1]` through the logistic
/// "win probability" curve.
fn value_from_centipawns(centipawns: i32) -> f64 {
    2.0 / (1.0 + 10f64.powf(-f64::from(centipawns) / 400.0)) - 1.0
}

/// Inverse of [`value_from_centipawns`], used for reporting the score.
#[allow(
    clippy::cast_possible_truncation,
    reason = "the score is clamped to a small range"
)]
fn centipawns_from_value(value: f64) -> i32 {
    let value = value.clamp(-0.9999, 0.9999);
    (400.0 * ((1.0 + value) / (1.0 - value)).log10()).round() as i32
}

fn info_line(tree: &Tree, iterations: u64, max_depth: usize, start: Instant) -> String {
    let elapsed = start.elapsed();
    let millis = elapsed.as_millis();
    let nps = if elapsed.as_secs_f64() > 0.0 {
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_precision_loss,
            clippy::cast_sign_loss,
            reason = "nps can not realistically overflow or be negative"
        )]
        let nps = (iterations as f64 / elapsed.as_secs_f64()) as u64;
        nps
    } else {
        0
    };

    let root = tree.node(ROOT_ID);
    let best = root
        .children
        .iter()
        .max_by_key(|&&child| tree.node(child).visits)
        .expect("root is always expanded");
    let score = centipawns_from_value(tree.node(*best).mean_value());

    let mut pv = String::new();
    let mut node_id = ROOT_ID;
    for _ in 0..MAX_PV_LENGTH {
        let node = tree.node(node_id);
        if node.children.is_empty() {
            break;
        }
        let next = *node
            .children
            .iter()
            .max_by_key(|&&child| tree.node(child).visits)
            .expect("children are not empty");
        if tree.node(next).visits == 0 {
            break;
        }
        if !pv.is_empty() {
            pv.push(' ');
        }
        pv.push_str(
            &tree
                .node(next)
                .action
                .expect("non-root nodes have actions")
                .to_string(),
        );
        node_id = next;
    }
    if pv.is_empty() {
        // Even with zero completed iterations there is a legal first move.
        pv = tree
            .node(root.children[0])
            .action
            .expect("root children have actions")
            .to_string();
    }

    format!(
        "info depth {max_depth} nodes {iterations} nps {nps} time {millis} score cp {score} pv \
         {pv}"
    )
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn run(fen: &str, limits: &Limits) -> SearchResult {
        let position = Position::from_fen(fen).unwrap();
        let history = vec![position.hash()];
        let stop = AtomicBool::new(false);
        search(&position, &history, limits, &stop, |_| {})
    }

    #[test]
    fn returns_legal_move_from_starting_position() {
        let position = Position::starting();
        let history = vec![position.hash()];
        let stop = AtomicBool::new(false);
        let result = search(
            &position,
            &history,
            &Limits {
                nodes: Some(100),
                ..Limits::default()
            },
            &stop,
            |_| {},
        );
        let best = result.best_move.expect("starting position has moves");
        assert!(position.generate_moves().contains(&best));
    }

    #[test]
    fn finds_backrank_mate_in_one() {
        let result = run(
            "6k1/5ppp/8/8/8/8/8/R5K1 w - - 0 1",
            &Limits {
                nodes: Some(3000),
                ..Limits::default()
            },
        );
        assert_eq!(result.best_move.unwrap().to_string(), "a1a8");
    }

    #[test]
    fn no_moves_in_checkmate() {
        // Black is checkmated: no moves to search.
        let result = run(
            "rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 4 3",
            &Limits::default(),
        );
        assert!(result.best_move.is_none());
    }

    #[test]
    fn stops_immediately_when_stop_flag_is_set() {
        let position = Position::starting();
        let history = vec![position.hash()];
        let stop = AtomicBool::new(true);
        // Infinite search with a pre-set stop flag has to terminate and still
        // produce a legal move.
        let result = search(
            &position,
            &history,
            &Limits {
                infinite: true,
                ..Limits::default()
            },
            &stop,
            |_| {},
        );
        assert!(result.best_move.is_some());
        assert_eq!(result.nodes, 0);
    }

    #[test]
    fn respects_node_limit() {
        let result = run(
            "6k1/5ppp/8/8/8/8/8/R5K1 w - - 0 1",
            &Limits {
                nodes: Some(64),
                ..Limits::default()
            },
        );
        assert_eq!(result.nodes, 64);
    }

    #[test]
    fn centipawn_value_round_trip() {
        for centipawns in [-500, -100, 0, 100, 500] {
            assert_eq!(
                centipawns_from_value(value_from_centipawns(centipawns)),
                centipawns
            );
        }
    }
}
