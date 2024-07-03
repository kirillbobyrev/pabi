use std::path::Path;

use shakmaty::Chess;
use shakmaty_syzygy::{AmbiguousWdl, Tablebase};

use super::core::{Color, Move, MoveList};
use crate::chess::position::Position;
use crate::chess::zobrist::RepetitionTable;
use crate::environment::{Action, Environment, GameResult, Observation};

impl Action for Move {
    // The action space is actually smaller than 64 * 64 * 4 for chess:
    // https://github.com/LeelaChessZero/lc0/blob/master/src/chess/bitboard.cc
    fn get_index(&self) -> u16 {
        todo!();
    }
}

impl Observation for Position {}

pub struct Game {
    position: Position,
    perspective: Color,
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
        if self.threefold_repetition {
            return Some(GameResult::Draw);
        }
        if self.position.halfmove_clock_expired() {
            return Some(GameResult::Draw);
        }
        debug_assert!(self.position.num_pieces() >= self.tablebase.max_pieces());
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
                    }
                },
                AmbiguousWdl::Draw | AmbiguousWdl::BlessedLoss | AmbiguousWdl::CursedWin => {
                    return Some(GameResult::Draw)
                },
                AmbiguousWdl::Loss | AmbiguousWdl::MaybeLoss => {
                    return if self.perspective == self.position.us() {
                        Some(GameResult::Loss)
                    } else {
                        Some(GameResult::Win)
                    }
                },
            }
        }
        if self.moves.is_empty() {
            if !self.position.in_check() {
                return Some(GameResult::Draw);
            }
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

// TODO: This should be ineffective.
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

    // #[test]
    // fn tablebase_adjudication() {
    //     todo!();
    // }

    // #[test]
    // fn checkmate() {
    //     todo!();
    // }


    // #[test]
    // fn stalemate() {
    //     todo!();
    // }


    // #[test]
    // fn fifty_move_rule() {
    //     todo!();
    // }
}
