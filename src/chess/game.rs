use std::path::Path;
use std::sync::Arc;

use shakmaty::Chess;
use shakmaty_syzygy::{AmbiguousWdl, Tablebase};

use super::core::{Move, MoveList};
use crate::chess::position::Position;
use crate::chess::zobrist::Key;
use crate::environment::{Action, Environment, GameResult, Observation, Player};

impl Action for Move {
    /// AlphaZero-style action space encoding, see [`Move::policy_index`].
    ///
    /// The move has to be from the perspective of the player making it: flip
    /// Black's moves with [`Move::flip_perspective`] first.
    fn get_index(&self) -> u16 {
        self.policy_index()
    }
}

impl Observation for Position {}

/// A chess game in progress: the current position together with the history
/// needed to adjudicate draws by repetition, and an optional Syzygy tablebase
/// for endgame adjudication.
///
/// `Game` is the shared substrate for both the UCI engine (which searches from
/// the current position) and self-play data generation (which drives a game to
/// a terminal result through the [`Environment`] interface).
pub struct Game {
    position: Position,
    /// Zobrist hashes of every position reached so far, including the current
    /// one. Used to detect repetitions.
    history: Vec<Key>,
    moves: MoveList,
    tablebase: Option<Arc<Tablebase<Chess>>>,
    /// The player to move at the root, from whose perspective [`Self::result`]
    /// reports the outcome.
    perspective: Player,
}

impl Game {
    /// Starts a game from the given root position without a tablebase.
    #[must_use]
    pub fn new(root: Position) -> Self {
        Self {
            perspective: root.us(),
            history: vec![root.hash()],
            moves: root.generate_moves(),
            position: root,
            tablebase: None,
        }
    }

    /// Sets (or clears, with `None`) the Syzygy tablebase used to adjudicate
    /// endgames.
    pub fn set_tablebase(&mut self, tablebase: Option<Arc<Tablebase<Chess>>>) {
        self.tablebase = tablebase;
    }

    #[must_use]
    pub fn position(&self) -> &Position {
        &self.position
    }

    /// Zobrist hashes of all positions in the game, including the current one.
    #[must_use]
    pub fn history(&self) -> &[Key] {
        &self.history
    }

    #[must_use]
    pub(crate) fn tablebase(&self) -> Option<Arc<Tablebase<Chess>>> {
        self.tablebase.clone()
    }

    /// Number of times the current position has occurred in the game.
    fn repetitions(&self) -> usize {
        let current = self.position.hash();
        self.history.iter().filter(|&&hash| hash == current).count()
    }

    /// Reorients a result reported from the side-to-move's perspective to the
    /// root perspective used by [`Self::result`].
    fn orient(&self, side_to_move: GameResult) -> GameResult {
        if self.position.us() == self.perspective {
            side_to_move
        } else {
            side_to_move.flip()
        }
    }
}

impl Environment<Move, Position> for Game {
    fn actions(&self) -> &[Move] {
        &self.moves
    }

    fn apply(&mut self, action: &Move) -> &Position {
        self.position.make_move(action);
        self.history.push(self.position.hash());
        self.moves = self.position.generate_moves();
        &self.position
    }

    fn result(&self) -> Option<GameResult> {
        if self.repetitions() >= 3
            || self.position.halfmove_clock_expired()
            || self.position.is_insufficient_material()
        {
            return Some(GameResult::Draw);
        }
        if let Some(tablebase) = &self.tablebase
            && let Some(result) = probe_tablebase(tablebase, &self.position)
        {
            return Some(self.orient(result));
        }
        if self.moves.is_empty() {
            // Checkmate for the side to move, or stalemate.
            return Some(if self.position.in_check() {
                self.orient(GameResult::Loss)
            } else {
                GameResult::Draw
            });
        }
        None
    }
}

/// Probes the Syzygy tablebase for the [win/draw/loss] value of `position` from
/// the perspective of the side to move, returning `None` when the position has
/// too many pieces or the probe fails.
///
/// [win/draw/loss]: https://www.chessprogramming.org/Syzygy_Bases
pub(crate) fn probe_tablebase(
    tablebase: &Tablebase<Chess>,
    position: &Position,
) -> Option<GameResult> {
    if position.num_pieces() > tablebase.max_pieces() {
        return None;
    }
    let wdl = tablebase.probe_wdl(&to_shakmaty_position(position)?).ok()?;
    Some(match wdl {
        AmbiguousWdl::Win | AmbiguousWdl::MaybeWin => GameResult::Win,
        AmbiguousWdl::Loss | AmbiguousWdl::MaybeLoss => GameResult::Loss,
        AmbiguousWdl::Draw | AmbiguousWdl::BlessedLoss | AmbiguousWdl::CursedWin => {
            GameResult::Draw
        }
    })
}

/// Loads all Syzygy tables found in the given directory.
///
/// # Errors
///
/// Returns an error if the directory can not be read or contains malformed
/// tables.
pub fn load_tablebase(directory: &Path) -> anyhow::Result<Tablebase<Chess>> {
    let mut tablebase = Tablebase::new();
    tablebase.add_directory(directory)?;
    Ok(tablebase)
}

// TODO: Converting to FEN and back is inefficient; the bitboards could be
// translated into shakmaty's representation directly.
fn to_shakmaty_position(position: &Position) -> Option<Chess> {
    position
        .to_string()
        .parse::<shakmaty::fen::Fen>()
        .ok()?
        .into_position(shakmaty::CastlingMode::Standard)
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    const TABLEBASE_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/syzygy");

    fn game_with_tablebase(position: Position) -> Game {
        let tablebase = Arc::new(load_tablebase(TABLEBASE_PATH.as_ref()).unwrap());
        let mut game = Game::new(position);
        game.set_tablebase(Some(tablebase));
        game
    }

    #[test]
    fn syzygy_tablebases() {
        let tablebase = load_tablebase(TABLEBASE_PATH.as_ref()).unwrap();
        assert_eq!(tablebase.max_pieces(), 3);
    }

    #[test]
    fn detect_repetition() {
        let mut game = Game::new(Position::starting());
        assert!(game.result().is_none());
        // Move 1.
        game.apply(&Move::from_uci("g1f3").unwrap());
        assert!(game.result().is_none());
        game.apply(&Move::from_uci("g8f6").unwrap());
        assert!(game.result().is_none());
        // Move 2: returning to the starting position.
        game.apply(&Move::from_uci("f3g1").unwrap());
        assert!(game.result().is_none());
        game.apply(&Move::from_uci("f6g8").unwrap());
        assert!(game.result().is_none());
        // Move 3.
        game.apply(&Move::from_uci("g1f3").unwrap());
        assert!(game.result().is_none());
        game.apply(&Move::from_uci("g8f6").unwrap());
        assert!(game.result().is_none());
        // Move 4: returning to the starting position with a threefold repetition.
        game.apply(&Move::from_uci("f3g1").unwrap());
        assert!(game.result().is_none());
        game.apply(&Move::from_uci("f6g8").unwrap());
        assert_eq!(game.result(), Some(GameResult::Draw));
    }

    #[test]
    fn tablebase_adjudication() {
        // KQvKR position with a forced win for white.
        let mut game = game_with_tablebase(
            Position::from_fen("4k3/8/8/5r2/4KQ2/8/8/8 w - - 0 1").expect("valid position"),
        );
        // The test tablebases only support 3 pieces, so the position is not
        // adjudicated until the rook is captured.
        assert!(game.result().is_none());

        // KQvK is a win after Qxf5 (rook capture).
        game.apply(&Move::from_uci("f4f5").unwrap());
        assert_eq!(
            game.position().to_string(),
            "4k3/8/8/5Q2/4K3/8/8/8 b - - 0 1"
        );
        // Black is to move, but the game is evaluated from white's perspective.
        assert_eq!(game.perspective, Player::White);
        assert_eq!(game.result(), Some(GameResult::Win));
    }

    #[test]
    fn stalemate() {
        let mut game = Game::new(
            Position::from_fen("3b2qk/p6p/1p3Q1P/8/8/n7/PP6/K7 b - - 3 2").expect("valid position"),
        );
        assert!(game.result().is_none());

        // Black has no moves and is not in check.
        game.apply(&Move::from_uci("d8f6").unwrap());
        assert!(game.actions().is_empty());
        assert_eq!(game.result(), Some(GameResult::Draw));
    }

    #[test]
    fn checkmate() {
        let mut game = Game::new(
            Position::from_fen("3b3k/p5qp/1p3Q1P/8/8/n7/PP6/K7 w - - 4 3").expect("valid position"),
        );
        assert!(game.result().is_none());

        game.apply(&Move::from_uci("f6g7").unwrap());
        assert!(game.actions().is_empty());
        assert_eq!(game.result(), Some(GameResult::Win));
    }

    #[test]
    fn fifty_move_rule() {
        // All legal moves just shuffle the kings; the halfmove clock expires on
        // the next move.
        let mut game = Game::new(
            Position::from_fen("8/5k2/3p4/1p1Pp2p/pP2Pp1P/P4P1K/8/8 b - - 99 50")
                .expect("valid position"),
        );
        assert!(game.result().is_none());

        game.apply(&Move::from_uci("f7f6").unwrap());
        assert_eq!(game.result(), Some(GameResult::Draw));
    }
}
