use arrayvec::ArrayVec;

use crate::chess::position::Position;
use crate::chess::zobrist::RepetitionTable;

pub(super) struct State {
    position_history: ArrayVec<Position, 256>,
    repetitions: RepetitionTable,
    searched_nodes: u64,
    // TODO: num_pruned for debugging
}

impl State {
    pub(super) fn new(root: Position) -> Self {
        let mut repetitions = RepetitionTable::new();
        let _ = repetitions.record(root.hash());

        let mut position_history = ArrayVec::new();
        position_history.push(root);

        Self {
            position_history,
            repetitions,
            searched_nodes: 1,
        }
    }

    #[must_use]
    pub(super) fn push(&mut self, position: Position) -> bool {
        let draw = self.repetitions.record(position.hash());
        self.position_history.push(position);
        self.searched_nodes += 1;
        draw
    }

    pub(super) fn pop(&mut self) {
        debug_assert!(!self.position_history.is_empty());
        debug_assert!(!self.repetitions.is_empty());

        self.repetitions
            .remove(self.position_history.last().unwrap().hash());
        self.position_history.pop();
    }

    #[must_use]
    pub(super) fn last(&self) -> &Position {
        debug_assert!(!self.position_history.is_empty());
        self.position_history.last().unwrap()
    }

    #[must_use]
    pub(super) fn searched_nodes(&self) -> u64 {
        self.searched_nodes
    }

    /// Returns the number of full moves since the start of the search.
    #[must_use]
    pub(super) fn moves(&self) -> u8 {
        assert!(!self.position_history.is_empty());
        let plies = self.position_history.len();
        if plies == 1 {
            // Only the root is present: no moves have been made.
            0
        } else {
            // Two plies per move, excluding the root.
            plies as u8 / 2
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chess::core::Move;
    use crate::chess::position::Position;

    // #[test]
    // fn detect_repetition() {
    //     let mut state = State::new(Position::starting());
    //     assert_eq!(state.searched_nodes(), 1);
    //     assert_eq!(state.moves(), 0);

    //     let mut position = Position::starting();
    //     position.make_move(&Move::from_uci("e2e4").unwrap());

    //     assert!(!state.push(position.clone()));
    //     assert_eq!(state.searched_nodes(), 2);
    //     assert_eq!(state.moves(), 1);

    //     assert!(!state.push(position.clone()));
    //     assert_eq!(state.searched_nodes(), 3);
    //     assert_eq!(state.moves(), 1);

    //     // 3-fold "repetition" (the same position was pushed multiple times).
    //     assert!(state.push(position.clone()));
    //     assert_eq!(state.searched_nodes(), 4);
    //     assert_eq!(state.moves(), 2);

    //     position.make_move(&Move::from_uci("e7e5").unwrap());
    //     // Next move is not a repetition.
    //     assert!(!state.push(position.clone()));
    //     assert_eq!(state.searched_nodes(), 5);
    //     assert_eq!(state.moves(), 2);

    //     state.pop();
    // }
}
