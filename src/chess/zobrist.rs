//! Zobrist hashing-related utilities`.

use std::collections::HashMap;

/// Zobrist keys are 64-bit unsigned integers that are computed once position is
/// created and updated whenever a move is made.
pub type Key = u64;

pub(crate) struct RepetitionTable {
    table: HashMap<Key, u8>,
}

impl RepetitionTable {
    /// Creates an empty repetition table.
    pub(crate) fn new() -> Self {
        Self {
            table: HashMap::new(),
        }
    }

    /// Removes all entries from the repetition history.
    pub(crate) fn clear(&mut self) {
        self.table.clear();
    }

    /// Checks whether the repetition table has no entries.
    ///
    /// This is mostly used for debugging purposes.
    #[must_use]
    pub(crate) fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    /// Returns true if the position has occurred 3 times.
    ///
    /// In the tournament setting 3-fold repetition is a draw.
    #[must_use]
    pub(crate) fn record(&mut self, key: Key) -> bool {
        let count = self.table.entry(key).or_insert(0);
        *count += 1;
        *count == 3
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chess::core::Move;
    use crate::chess::position::Position;

    #[test]
    fn repetition_table() {
        let mut table = RepetitionTable::new();

        let mut position = Position::starting();
        let initial_hash = position.hash();
        assert!(!table.record(initial_hash));

        position.make_move(&Move::from_uci("g1f3").expect("valid move"));
        assert_ne!(initial_hash, position.hash());
        assert!(!table.record(position.hash()));
        position.make_move(&Move::from_uci("g8f6").expect("valid move"));
        assert!(!table.record(position.hash()));

        position.make_move(&Move::from_uci("f3g1").expect("valid move"));
        assert!(!table.record(position.hash()));
        // Two-fold repetition.
        position.make_move(&Move::from_uci("f6g8").expect("valid move"));
        assert!(!table.record(position.hash()));

        position.make_move(&Move::from_uci("g1f3").expect("valid move"));
        assert!(!table.record(position.hash()));
        position.make_move(&Move::from_uci("g8f6").expect("valid move"));
        assert!(!table.record(position.hash()));

        position.make_move(&Move::from_uci("f3g1").expect("valid move"));
        assert!(!table.record(position.hash()));
        // Three-fold repetition.
        position.make_move(&Move::from_uci("f6g8").expect("valid move"));
        assert!(table.record(position.hash()));
    }
}
