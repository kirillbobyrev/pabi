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
    outcome: Option<GameResult>,
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
            outcome: None,
        }
    }
}

impl Environment<Move, Position> for Game {
    fn actions(&self) -> &[Move] {
        &self.moves
    }

    fn apply(&mut self, action: &Move) -> &Position {
        self.position.make_move(action);
        let _ = self.repetitions.record(self.position.hash());
        self.moves = self.position.generate_moves();
        &self.position
    }

    fn result(&self) -> Option<GameResult> {
        // TODO: Check 50-move rule.
        // TODO: Check threefold repetition.
        // TODO: Check checkmate.
        // TODO: Check Syzygy tablebases.
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
