use std::path::Path;

use shakmaty::Chess;
use shakmaty_syzygy::{AmbiguousWdl, Tablebase};

use super::core::{Move, MoveList};
use crate::chess::position::Position;
use crate::chess::zobrist::RepetitionTable;
use crate::environment::{Action, Environment, GameResult, Observation, Player};

impl Action for Move {
    // Action space compression from lc0:
    // https://github.com/LeelaChessZero/lc0/blob/master/src/chess/bitboard.cc
    fn get_index(&self) -> u16 {
        todo!();
    }
}

impl Observation for Position {}

pub struct Game {
    position: Position,
    perspective: Player,
    repetitions: RepetitionTable,
    moves: MoveList,
    tablebase: Tablebase<Chess>,
    threefold_repetition: bool,
}

impl Game {
    pub(super) fn new(root: Position, tablebase_dir: &Path) -> Self {
        let mut repetitions = RepetitionTable::new();
        let _ = repetitions.record(root.hash());

        let perspective = root.us();
        let moves = root.generate_moves();

        Self {
            position: root,
            perspective,
            repetitions,
            moves,
            tablebase: read_tablebase(tablebase_dir),
            threefold_repetition: false,
        }
    }
}

impl Environment<Move, Position> for Game {
    fn actions(&self) -> &[Move] {
        &self.moves
    }

    fn apply(&mut self, action: &Move) -> &Position {
        self.position.make_move(action);
        self.threefold_repetition = self.repetitions.record(self.position.hash());
        self.moves = self.position.generate_moves();
        &self.position
    }

    fn result(&self) -> Option<GameResult> {
        debug_assert!(self.position.num_pieces() >= self.tablebase.max_pieces());

        if self.threefold_repetition {
            return Some(GameResult::Draw);
        }
        if self.position.halfmove_clock_expired() {
            return Some(GameResult::Draw);
        }
        if self.position.num_pieces() == self.tablebase.max_pieces() {
            // TODO: This is a bit of a hack right now and not precise. Maybe
            // it's not that inmportant, but worth revisiting.
            let wdl = self
                .tablebase
                .probe_wdl(&to_shakmaty_position(&self.position))
                .unwrap();
            match wdl {
                AmbiguousWdl::Win | AmbiguousWdl::MaybeWin => {
                    return if self.perspective == self.position.us() {
                        Some(GameResult::Win)
                    } else {
                        Some(GameResult::Loss)
                    };
                }
                AmbiguousWdl::Draw | AmbiguousWdl::BlessedLoss | AmbiguousWdl::CursedWin => {
                    return Some(GameResult::Draw);
                }
                AmbiguousWdl::Loss | AmbiguousWdl::MaybeLoss => {
                    return if self.perspective == self.position.us() {
                        Some(GameResult::Loss)
                    } else {
                        Some(GameResult::Win)
                    };
                }
            }
        }
        if self.moves.is_empty() {
            // Stalemate.
            if !self.position.in_check() {
                return Some(GameResult::Draw);
            }
            // Player to move is in checkmate.
            return if self.perspective == self.position.us() {
                Some(GameResult::Loss)
            } else {
                Some(GameResult::Win)
            };
        }
        None
    }
}

fn read_tablebase(path: &Path) -> Tablebase<Chess> {
    let mut tablebase = Tablebase::new();
    tablebase.add_directory(path).unwrap();
    tablebase
}

// TODO: Converting to FEN and back is ineffective. It's possible to manipulate
// the bitboard values directly.
fn to_shakmaty_position(position: &Position) -> Chess {
    position
        .to_string()
        .parse::<shakmaty::fen::Fen>()
        .unwrap()
        .into_position(shakmaty::CastlingMode::Standard)
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    const TABLEBASE_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/syzygy");

    #[test]
    fn syzygy_tablebases() {
        let tables = read_tablebase(TABLEBASE_PATH.as_ref());
        assert_eq!(tables.max_pieces(), 3);
    }

    #[test]
    fn detect_repetition() {
        let mut game = Game::new(Position::starting(), TABLEBASE_PATH.as_ref());
        assert!(game.result().is_none());
        // Move 1.
        game.apply(&Move::from_uci("g1f3").unwrap());
        assert!(game.result().is_none());
        game.apply(&Move::from_uci("g8f6").unwrap());
        assert!(game.result().is_none());
        // Move 2: returning to starting position.
        game.apply(&Move::from_uci("f3g1").unwrap());
        assert!(game.result().is_none());
        game.apply(&Move::from_uci("f6g8").unwrap());
        assert!(game.result().is_none());
        // Move 3.
        game.apply(&Move::from_uci("g1f3").unwrap());
        assert!(game.result().is_none());
        game.apply(&Move::from_uci("g8f6").unwrap());
        assert!(game.result().is_none());
        // Move 4: returning to starting position with threefold repetition.
        game.apply(&Move::from_uci("f3g1").unwrap());
        assert!(game.result().is_none());
        game.apply(&Move::from_uci("f6g8").unwrap());
        assert_eq!(game.result(), Some(GameResult::Draw));
    }

    #[test]
    fn tablebase_adjudication() {
        // KQvKR position with a forced win for white.
        let mut game = Game::new(
            Position::from_fen("4k3/8/8/5r2/4KQ2/8/8/8 w - - 0 1").expect("valid_position"),
            TABLEBASE_PATH.as_ref(),
        );
        // Test tablebases only support 3 pieces, so it will not be adjudicated
        // until the rook is captured.
        assert!(game.result().is_none());

        // KQvK is a win after Qxg5 (rook capture).
        game.apply(&Move::from_uci("f4f5").unwrap());
        assert_eq!(game.position.to_string(), "4k3/8/8/5Q2/4K3/8/8/8 b - - 0 1");
        // Black is to move, but the game is evaluated from white's perspective at root.
        assert_eq!(game.perspective, Player::White);
        assert_eq!(game.result(), Some(GameResult::Win));
    }

    #[test]
    fn stalemate() {
        let mut game = Game::new(
            Position::from_fen("3b2qk/p6p/1p3Q1P/8/8/n7/PP6/K7 b - - 3 2").expect("valid_position"),
            TABLEBASE_PATH.as_ref(),
        );
        assert!(game.result().is_none());

        // Black has no moves and is not in check.
        game.apply(&Move::from_uci("d8f6").unwrap());
        assert!(game.moves.is_empty());
        assert_eq!(game.result(), Some(GameResult::Draw));
    }

    #[test]
    fn checkmate() {
        let mut game = Game::new(
            Position::from_fen("3b3k/p5qp/1p3Q1P/8/8/n7/PP6/K7 w - - 4 3").expect("valid_position"),
            TABLEBASE_PATH.as_ref(),
        );
        assert!(game.result().is_none());

        game.apply(&Move::from_uci("f6g7").unwrap());
        assert!(game.moves.is_empty());
        assert_eq!(game.result(), Some(GameResult::Win));
    }

    #[test]
    fn fifty_move_rule() {
        // All legal moves are just moving the kings back and forth, the
        // halfmove clock expires on the next turn.
        let mut game = Game::new(
            Position::from_fen("8/5k2/3p4/1p1Pp2p/pP2Pp1P/P4P1K/8/8 b - - 99 50")
                .expect("valid_position"),
            TABLEBASE_PATH.as_ref(),
        );
        assert!(game.result().is_none());

        game.apply(&Move::from_uci("f7f6").unwrap());
        assert_eq!(game.result(), Some(GameResult::Draw));
    }
}
