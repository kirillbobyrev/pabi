//! Implementation of the AlphaZero-style [Monte Carlo Tree Search]:
//!
//! 1. Selection: starting from the root, descend by picking the most promising
//!    child ([`crate::search::policy`]) until a leaf is reached.
//! 2. Expansion: generate the leaf's children and assign prior probabilities.
//! 3. Evaluation: score the leaf with the value function
//!    ([`crate::evaluation`]); terminal positions get their exact game value,
//!    including Syzygy tablebase adjudication. There are no random playouts,
//!    matching the AlphaZero variant of MCTS.
//! 4. Backup: propagate the value up the path, flipping the sign at each level
//!    (the value of a position for one player is the negation of its value for
//!    the opponent).
//!
//! [Monte Carlo Tree Search]: https://en.wikipedia.org/wiki/Monte_Carlo_tree_search

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use shakmaty::Chess;
use shakmaty_syzygy::Tablebase;

use super::policy;
use super::tree::{NodeId, ROOT_ID, Tree};
use crate::chess::core::Move;
use crate::chess::game;
use crate::chess::position::Position;
use crate::chess::zobrist::Key;
use crate::environment::GameResult;
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
    /// If set, the search runs until stopped externally, ignoring other limits.
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
/// `history` holds the Zobrist hashes of all positions of the game so far
/// (including the root); it is used to score repetitions as draws. `tablebase`,
/// when present, adjudicates positions with few enough pieces exactly.
///
/// `on_info` is called with UCI `info` lines as the search progresses.
pub fn search(
    root: &Position,
    history: &[Key],
    tablebase: Option<&Tablebase<Chess>>,
    limits: &Limits,
    stop: &AtomicBool,
    mut on_info: impl FnMut(&str),
) -> SearchResult {
    if root.generate_moves().is_empty() {
        return SearchResult {
            best_move: None,
            nodes: 0,
        };
    }

    let mut searcher = Searcher::new(root, history, tablebase);
    let start = Instant::now();
    let mut iterations = 0u64;
    let mut max_depth = 0usize;
    let mut last_report = Instant::now();

    while !stop.load(Ordering::Relaxed) && !limits_reached(limits, iterations, start) {
        max_depth = max_depth.max(searcher.simulate());
        iterations += 1;

        if iterations.is_multiple_of(CHECK_INTERVAL)
            && last_report.elapsed() >= Duration::from_secs(1)
        {
            on_info(&searcher.info_line(iterations, max_depth, start));
            last_report = Instant::now();
        }
    }

    on_info(&searcher.info_line(iterations, max_depth, start));
    SearchResult {
        best_move: Some(searcher.best_move()),
        nodes: iterations,
    }
}

fn limits_reached(limits: &Limits, iterations: u64, start: Instant) -> bool {
    if limits.infinite {
        return false;
    }
    limits.nodes.is_some_and(|nodes| iterations >= nodes)
        || limits
            .move_time
            .is_some_and(|budget| start.elapsed() >= budget)
}

/// Carries the immutable search inputs alongside the growing tree so that the
/// per-iteration steps do not have to thread them through as parameters.
struct Searcher<'a> {
    root: &'a Position,
    history: &'a [Key],
    tablebase: Option<&'a Tablebase<Chess>>,
    tree: Tree,
}

impl<'a> Searcher<'a> {
    /// Creates a searcher with the root node already expanded. The caller
    /// guarantees the root has at least one legal move.
    fn new(
        root: &'a Position,
        history: &'a [Key],
        tablebase: Option<&'a Tablebase<Chess>>,
    ) -> Self {
        let mut tree = Tree::new();
        expand(&mut tree, ROOT_ID, &root.generate_moves());
        Self {
            root,
            history,
            tablebase,
            tree,
        }
    }

    /// Runs one iteration (selection, expansion, evaluation, backup) and
    /// returns the depth reached.
    fn simulate(&mut self) -> usize {
        let mut position = self.root.clone();
        let mut path = vec![ROOT_ID];
        let mut path_hashes: Vec<Key> = Vec::new();
        let mut node_id = ROOT_ID;

        // Selection: descend until a terminal or unexpanded node.
        while self.tree.node(node_id).terminal.is_none() && self.tree.node(node_id).expanded {
            node_id = policy::select_child(&self.tree, node_id, CPUCT);
            let action = self
                .tree
                .node(node_id)
                .action
                .expect("non-root nodes always have an action");
            position.make_move(&action);
            path_hashes.push(position.hash());
            path.push(node_id);
        }

        let value = match self.tree.node(node_id).terminal {
            Some(value) => value,
            None => self.expand_and_evaluate(node_id, &position, &path_hashes),
        };
        self.backup(&path, value);
        path.len() - 1
    }

    /// Expands the leaf and returns its value from the perspective of the
    /// player to move at the leaf. Terminal values are cached in the node:
    /// the path from the root to a node is unique, so its terminal status
    /// (including draws by repetition) is deterministic.
    fn expand_and_evaluate(
        &mut self,
        node_id: NodeId,
        position: &Position,
        path_hashes: &[Key],
    ) -> f64 {
        let moves = position.generate_moves();
        if moves.is_empty() {
            // Checkmate is the worst outcome for the player to move; stalemate
            // is a draw.
            return self.mark_terminal(node_id, if position.in_check() { -1.0 } else { 0.0 });
        }

        // The root itself (empty path) is never a leaf here, so any repetition
        // or rule-based draw along the path ends the game.
        if let [earlier @ .., current] = path_hashes
            && (position.halfmove_clock_expired()
                || position.is_insufficient_material()
                || earlier.contains(current)
                || self.history.contains(current))
        {
            return self.mark_terminal(node_id, 0.0);
        }

        if let Some(tablebase) = self.tablebase
            && let Some(result) = game::probe_tablebase(tablebase, position)
        {
            return self.mark_terminal(node_id, value_from_result(result));
        }

        expand(&mut self.tree, node_id, &moves);
        value_from_centipawns(evaluation::evaluate(position))
    }

    /// Records an exact terminal value on the node and returns it.
    fn mark_terminal(&mut self, node_id: NodeId, value: f64) -> f64 {
        self.tree.node_mut(node_id).terminal = Some(value);
        value
    }

    /// Propagates `leaf_value` (from the perspective of the player to move at
    /// the leaf) up the path, flipping the sign at each level because node
    /// statistics are stored from the parent's perspective.
    fn backup(&mut self, path: &[NodeId], leaf_value: f64) {
        let mut value = leaf_value;
        for &node_id in path.iter().rev() {
            let node = self.tree.node_mut(node_id);
            node.visits += 1;
            node.total_value -= value;
            value = -value;
        }
    }

    /// Returns the root child with the most visits, breaking ties by mean
    /// value.
    fn best_move(&self) -> Move {
        let best = self
            .most_visited_child(ROOT_ID)
            .expect("root is always expanded");
        self.tree
            .node(best)
            .action
            .expect("root children have actions")
    }

    /// Returns the most-visited child of a node, breaking ties by mean value.
    fn most_visited_child(&self, node_id: NodeId) -> Option<NodeId> {
        self.tree
            .node(node_id)
            .children
            .iter()
            .copied()
            .max_by(|&a, &b| {
                let (a, b) = (self.tree.node(a), self.tree.node(b));
                a.visits
                    .cmp(&b.visits)
                    .then(a.mean_value().total_cmp(&b.mean_value()))
            })
    }

    fn info_line(&self, iterations: u64, max_depth: usize, start: Instant) -> String {
        let elapsed = start.elapsed();
        let millis = elapsed.as_millis();
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_precision_loss,
            clippy::cast_sign_loss,
            reason = "nps can not realistically overflow or be negative"
        )]
        let nps = (iterations as f64 / elapsed.as_secs_f64().max(f64::MIN_POSITIVE)) as u64;

        let score = self.most_visited_child(ROOT_ID).map_or(0, |best| {
            centipawns_from_value(self.tree.node(best).mean_value())
        });
        let pv = self.principal_variation();

        format!(
            "info depth {max_depth} nodes {iterations} nps {nps} time {millis} score cp {score} \
             pv {pv}"
        )
    }

    /// Builds the principal variation by following the most-visited child from
    /// the root while nodes have been visited.
    fn principal_variation(&self) -> String {
        let mut pv = Vec::new();
        let mut node_id = ROOT_ID;
        while pv.len() < MAX_PV_LENGTH {
            let Some(best) = self.most_visited_child(node_id) else {
                break;
            };
            if self.tree.node(best).visits == 0 {
                break;
            }
            pv.push(
                self.tree
                    .node(best)
                    .action
                    .expect("non-root nodes have actions")
                    .to_string(),
            );
            node_id = best;
        }
        // Even with zero completed iterations there is a legal first move.
        if pv.is_empty() {
            pv.push(self.best_move().to_string());
        }
        pv.join(" ")
    }
}

/// Adds a child node for every legal move, with a uniform prior, and marks the
/// node expanded.
fn expand(tree: &mut Tree, node_id: NodeId, moves: &[Move]) {
    #[allow(clippy::cast_precision_loss, reason = "the number of moves is small")]
    let prior = 1.0 / moves.len() as f32;
    for next_move in moves {
        let _ = tree.add_child(node_id, *next_move, prior);
    }
    tree.node_mut(node_id).expanded = true;
}

/// Maps a game result to the search value domain `[-1, 1]`.
fn value_from_result(result: GameResult) -> f64 {
    match result {
        GameResult::Win => 1.0,
        GameResult::Draw => 0.0,
        GameResult::Loss => -1.0,
    }
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

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn run(fen: &str, limits: &Limits) -> SearchResult {
        let position = Position::from_fen(fen).unwrap();
        let history = vec![position.hash()];
        let stop = AtomicBool::new(false);
        search(&position, &history, None, limits, &stop, |_| {})
    }

    #[test]
    fn returns_legal_move_from_starting_position() {
        let position = Position::starting();
        let result = run(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            &Limits {
                nodes: Some(100),
                ..Limits::default()
            },
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
            None,
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
    fn searches_with_tablebase() {
        use crate::chess::game;

        const TABLEBASE_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/syzygy");
        let tablebase = game::load_tablebase(TABLEBASE_PATH.as_ref()).unwrap();

        // KQvK is a tablebase win for White; the search probes the 3-piece
        // children and must still return a legal move.
        let position = Position::from_fen("4k3/8/8/8/4KQ2/8/8/8 w - - 0 1").unwrap();
        let history = vec![position.hash()];
        let stop = AtomicBool::new(false);
        let result = search(
            &position,
            &history,
            Some(&tablebase),
            &Limits {
                nodes: Some(500),
                ..Limits::default()
            },
            &stop,
            |_| {},
        );
        let best = result.best_move.expect("KQvK has legal moves");
        assert!(position.generate_moves().contains(&best));
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
