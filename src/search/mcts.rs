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

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use shakmaty::Chess;
use shakmaty_syzygy::Tablebase;

use super::policy;
use super::tree::{Node, NodeId, ROOT_ID, Tree};
use crate::chess::core::Move;
use crate::chess::game::{self, Game};
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

/// Converts a Hash option value (in MB) into a search tree node budget. The
/// per-node estimate accounts for a [`Node`] and the small heap-allocated child
/// list it typically owns.
#[must_use]
pub fn node_budget(hash_mb: usize) -> usize {
    const BYTES_PER_NODE: usize = size_of::<Node>() + 32;
    (hash_mb * 1_000_000 / BYTES_PER_NODE).max(1)
}

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

/// Static configuration of a search: how large the tree may grow and how many
/// principal variations to report.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Config {
    /// Maximum number of tree nodes (see [`node_budget`]).
    pub(crate) node_budget: usize,
    /// Number of best lines to report (the UCI `MultiPV` option).
    pub(crate) multipv: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            node_budget: usize::MAX,
            multipv: 1,
        }
    }
}

/// Final result of a search.
pub(crate) struct SearchResult {
    /// The move the engine considers best. `None` if the root position has no
    /// legal moves.
    pub(crate) best_move: Option<Move>,
    /// Number of search iterations performed.
    pub(crate) nodes: u64,
    /// Visit count of each legal root move. This is the MCTS policy: a move's
    /// share of the total visits is the search's improved probability of
    /// playing it, which self-play uses as the policy training target.
    pub(crate) root_visits: Vec<(Move, u32)>,
    /// The search tree, returned so its statistics can seed a later search.
    pub(crate) tree: Tree,
}

/// Searches the game's current position and returns the best move along with
/// the (grown) search tree.
///
/// `tree` seeds the search: pass [`Tree::new`] for a fresh search or a
/// [`Tree::rerooted`] subtree to reuse earlier statistics. `config` controls
/// the tree size budget and the number of reported lines.
///
/// `on_info` is called with UCI `info` blocks as the search progresses.
pub(crate) fn search(
    game: &Game,
    tree: Tree,
    config: &Config,
    limits: &Limits,
    stop: &AtomicBool,
    mut on_info: impl FnMut(&str),
) -> SearchResult {
    let root_moves = game.position().generate_moves();
    if root_moves.is_empty() {
        return SearchResult {
            best_move: None,
            nodes: 0,
            root_visits: Vec::new(),
            tree,
        };
    }

    let mut searcher = Searcher::new(game, tree, config, &root_moves);
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
            on_info(&searcher.report(iterations, max_depth, start));
            last_report = Instant::now();
        }
    }

    on_info(&searcher.report(iterations, max_depth, start));
    SearchResult {
        best_move: Some(searcher.best_move()),
        nodes: iterations,
        root_visits: searcher.root_visits(),
        tree: searcher.tree,
    }
}

/// Convenience entry point for self-play: runs a fresh, unbounded search under
/// `limits` and returns the root move visit counts (the policy target).
pub(crate) fn policy(game: &Game, limits: &Limits) -> Vec<(Move, u32)> {
    let stop = AtomicBool::new(false);
    search(game, Tree::new(), &Config::default(), limits, &stop, |_| {}).root_visits
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
    tablebase: Option<Arc<Tablebase<Chess>>>,
    config: Config,
    tree: Tree,
}

impl<'a> Searcher<'a> {
    /// Creates a searcher, expanding the root node if the seed tree left it
    /// unexpanded. The caller guarantees the root has at least one legal move.
    fn new(game: &'a Game, mut tree: Tree, config: &Config, root_moves: &[Move]) -> Self {
        if !tree.node(ROOT_ID).expanded {
            expand(&mut tree, ROOT_ID, root_moves);
        }
        Self {
            root: game.position(),
            history: game.history(),
            tablebase: game.tablebase(),
            config: *config,
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

        if let Some(tablebase) = &self.tablebase
            && let Some(result) = game::probe_tablebase(tablebase, position)
        {
            return self.mark_terminal(node_id, value_from_result(result));
        }

        // Stop growing the tree once it reaches the memory budget; the leaf is
        // still evaluated, it just is not expanded.
        if self.tree.len() < self.config.node_budget {
            expand(&mut self.tree, node_id, &moves);
        }
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

    /// Returns the visit count of each root move (the MCTS policy).
    fn root_visits(&self) -> Vec<(Move, u32)> {
        self.tree
            .node(ROOT_ID)
            .children
            .iter()
            .map(|&child| {
                let node = self.tree.node(child);
                (
                    node.action.expect("root children have actions"),
                    node.visits,
                )
            })
            .collect()
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

    /// Builds the UCI `info` block: one line per reported principal variation
    /// (up to `MultiPV`), ordered best first.
    fn report(&self, iterations: u64, max_depth: usize, start: Instant) -> String {
        let elapsed = start.elapsed();
        let millis = elapsed.as_millis();
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_precision_loss,
            clippy::cast_sign_loss,
            reason = "nps can not realistically overflow or be negative"
        )]
        let nps = (iterations as f64 / elapsed.as_secs_f64().max(f64::MIN_POSITIVE)) as u64;
        let hashfull =
            (self.tree.len() as u64 * 1000 / self.config.node_budget.max(1) as u64).min(1000);

        // Report the most-visited root moves first, as many as MultiPV asks for.
        let mut roots: Vec<NodeId> = self.tree.node(ROOT_ID).children.clone();
        roots.sort_by(|&a, &b| {
            let (a, b) = (self.tree.node(a), self.tree.node(b));
            b.visits
                .cmp(&a.visits)
                .then(b.mean_value().total_cmp(&a.mean_value()))
        });

        roots
            .iter()
            .take(self.config.multipv.max(1))
            .enumerate()
            .map(|(rank, &child)| {
                let pv = self.line(child);
                let score = self.score(&pv, self.tree.node(child).mean_value());
                let moves: Vec<String> = pv.iter().map(ToString::to_string).collect();
                format!(
                    "info depth {} seldepth {max_depth} multipv {} score {score} nodes \
                     {iterations} nps {nps} hashfull {hashfull} time {millis} pv {}",
                    pv.len(),
                    rank + 1,
                    moves.join(" ")
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Builds a principal variation starting with `first`, following the
    /// most-visited children.
    fn line(&self, first: NodeId) -> Vec<Move> {
        let mut pv = vec![self.tree.node(first).action.expect("child has action")];
        let mut node_id = first;
        while pv.len() < MAX_PV_LENGTH {
            let Some(next) = self.most_visited_child(node_id) else {
                break;
            };
            if self.tree.node(next).visits == 0 {
                break;
            }
            pv.push(
                self.tree
                    .node(next)
                    .action
                    .expect("non-root nodes have actions"),
            );
            node_id = next;
        }
        pv
    }

    /// Formats the score of a line: `mate N` when the line ends in checkmate,
    /// otherwise `cp N` from the root player's perspective.
    fn score(&self, pv: &[Move], mean_value: f64) -> String {
        self.mate_distance(pv).map_or_else(
            || format!("cp {}", centipawns_from_value(mean_value)),
            |mate| format!("mate {mate}"),
        )
    }

    /// If the line ends in checkmate, returns the mate distance in moves:
    /// positive if the root player mates, negative if it is mated. Tablebase
    /// wins are not mates and return `None`.
    fn mate_distance(&self, pv: &[Move]) -> Option<i32> {
        let mut position = self.root.clone();
        for next_move in pv {
            position.make_move(next_move);
        }
        if !position.generate_moves().is_empty() || !position.in_check() {
            return None;
        }
        let plies = i32::try_from(pv.len()).expect("pv length is small");
        // Odd length: the root player delivered mate; even: the root player was
        // mated.
        Some(if plies % 2 == 1 {
            (plies + 1) / 2
        } else {
            -(plies / 2)
        })
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
    use crate::environment::Environment;

    fn search_game(game: &Game, limits: &Limits) -> SearchResult {
        let stop = AtomicBool::new(false);
        search(game, Tree::new(), &Config::default(), limits, &stop, |_| {})
    }

    fn run(fen: &str, limits: &Limits) -> SearchResult {
        search_game(&Game::new(Position::from_fen(fen).unwrap()), limits)
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
        let stop = AtomicBool::new(true);
        // Infinite search with a pre-set stop flag has to terminate and still
        // produce a legal move.
        let result = search(
            &Game::new(Position::starting()),
            Tree::new(),
            &Config::default(),
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
        const TABLEBASE_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/syzygy");
        let tablebase = Arc::new(game::load_tablebase(TABLEBASE_PATH.as_ref()).unwrap());

        // KQvK is a tablebase win for White; the search probes the 3-piece
        // children and must still return a legal move.
        let position = Position::from_fen("4k3/8/8/8/4KQ2/8/8/8 w - - 0 1").unwrap();
        let mut game = Game::new(position.clone());
        game.set_tablebase(Some(tablebase));
        let result = search_game(
            &game,
            &Limits {
                nodes: Some(500),
                ..Limits::default()
            },
        );
        let best = result.best_move.expect("KQvK has legal moves");
        assert!(position.generate_moves().contains(&best));
    }

    #[test]
    fn reuses_tree_across_moves() {
        // Search the starting position, then reuse the subtree under 1. e4 e5
        // for the next search: the reused tree already carries visits, so the
        // continuation search starts from a non-empty tree.
        let mut game = Game::new(Position::starting());
        let stop = AtomicBool::new(false);
        let first = search(
            &game,
            Tree::new(),
            &Config::default(),
            &Limits {
                nodes: Some(2000),
                ..Limits::default()
            },
            &stop,
            |_| {},
        );

        let line = [
            Move::from_uci("e2e4").unwrap(),
            Move::from_uci("e7e5").unwrap(),
        ];
        let reused = first
            .tree
            .rerooted(&line)
            .expect("the 1. e4 e5 line was explored");
        assert!(reused.len() > 1, "reused subtree should carry statistics");

        for played in line {
            game.apply(&played);
        }
        let second = search(
            &game,
            reused,
            &Config::default(),
            &Limits {
                nodes: Some(100),
                ..Limits::default()
            },
            &stop,
            |_| {},
        );
        let best = second.best_move.expect("position has moves");
        assert!(game.position().generate_moves().contains(&best));
    }

    #[test]
    fn node_budget_scales_with_hash() {
        assert!(node_budget(64) > node_budget(1));
        assert!(node_budget(0) >= 1);
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
