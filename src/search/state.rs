use arrayvec::ArrayVec;

use crate::chess::{position::Position, zobrist::RepetitionTable};

use super::Depth;

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
        let plies = self.position_history.len() as u8;
        // Two plies per move, excluding the root.
        plies / 2
    }
}

/*
impl PositionHistory {
    /// Creates an empty position history for new search.
    #[must_use]
    pub(crate) fn new() -> Self {
        Self {
            history: ArrayVec::new(),
            repetitions: RepetitionTable::new(),
        }
    }

    #[must_use]
    pub(crate) fn current_position(&self) -> &Position {
        self.history.last().expect("no positions in history")
    }

    #[must_use]
    pub(crate) fn push(&mut self, position: Position) -> bool {
        let hash = position.hash();
        self.history.push(position);
        self.repetitions.record(hash)
    }

    /// Removes the last position from the history and reduces the number of
    /// times that position is counted for repretitions.
    pub(crate) fn pop(&mut self) {
        let _ = self.history.pop();
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.history.is_empty()
    }

    pub(crate) fn last(&self) -> &Position {
        debug_assert!(!self.is_empty());
        self.history.last().unwrap()
    }
}
*/
