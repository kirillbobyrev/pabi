use super::core::{Color, Move, MoveList};
use crate::chess::position::Position;
use crate::chess::zobrist::RepetitionTable;
use crate::environment::{Action, Environment, GameResult, Observation};

impl Observation for Position {}

pub struct Game {
    position: Position,
    perspective: Color,
    repetitions: RepetitionTable,
    moves: MoveList,
}

impl Game {
    pub(super) fn new(root: Position) -> Self {
        let mut repetitions = RepetitionTable::new();
        let _ = repetitions.record(root.hash());

        let perspective = root.us();
        let moves = root.generate_moves();

        Self {
            position: root,
            perspective,
            repetitions,
            moves,
        }
    }
}

impl Environment<Move, Position> for Game {
    fn actions(&self) -> &[Move] {
        &self.moves
    }

    fn apply(&mut self, action: impl Action) -> Position {
        todo!();
    }

    fn result(&self) -> Option<GameResult> {
        // TODO: Check 50-move rule.
        // TODO: Check threefold repetition.
        // TODO: Check checkmate.
        // TODO: Check Syzygy tablebases.
        todo!();
    }
}

impl Action for Move {
    fn get_index(&self) -> u16 {
        todo!();
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn detect_repetition() {
        todo!();
    }

    #[test]
    fn game_result() {
        todo!();
    }
}
