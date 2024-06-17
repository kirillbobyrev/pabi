//! Implements Zobrist hashing and [Transposition Table] functionality.
//!
//! [Transposition Table](https://www.chessprogramming.org/Transposition_Table

// TODO: Migrate to RawTable instead for better performance?
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use super::position::Position;

/// Zobrist key is a 64-bit integer.
pub type Key = u64;

pub(crate) struct RepetitionTable {
    table: HashMap<Key, u8>,
}

impl RepetitionTable {
    pub(crate) fn new() -> Self {
        Self {
            table: HashMap::new(),
        }
    }

    pub(crate) fn clear(&mut self) {
        self.table.clear();
    }

    /// Returns true if the position has occurred 3 times.
    pub(crate) fn record(&mut self, key: Key) -> bool {
        let count = self.table.entry(key).or_insert(0);
        *count += 1;
        *count == 3
    }

    /// Reduces the number of times a position has occurred by one.
    pub(crate) fn remove(&mut self, key: Key) {
        let count = self.table.entry(key).or_insert(0);
        match count {
            1 => {
                let _ = self.table.remove_entry(&key);
            },
            2 | 3 => {
                *count -= 1;
            },
            _ => {
                unreachable!("can not call remove on position that has not occurred yet")
            },
        }
    }
}

mod test {
    use super::*;
    use crate::chess::core::Move;

    #[test]
    fn repetition_table() {
        let mut table = RepetitionTable::new();

        let mut position = Position::starting();
        assert!(!table.record(position.compute_hash()));

        position.make_move(&Move::from_uci("g1f3").expect("valid move"));
        assert!(!table.record(position.compute_hash()));
        position.make_move(&Move::from_uci("g8f6").expect("valid move"));
        assert!(!table.record(position.compute_hash()));

        position.make_move(&Move::from_uci("f3g1").expect("valid move"));
        assert!(!table.record(position.compute_hash()));
        // Two-fold repetition.
        position.make_move(&Move::from_uci("f6g8").expect("valid move"));
        assert!(!table.record(position.compute_hash()));

        position.make_move(&Move::from_uci("g1f3").expect("valid move"));
        assert!(!table.record(position.compute_hash()));
        position.make_move(&Move::from_uci("g8f6").expect("valid move"));
        assert!(!table.record(position.compute_hash()));

        position.make_move(&Move::from_uci("f3g1").expect("valid move"));
        assert!(!table.record(position.compute_hash()));
       // Three-fold repetition.
        position.make_move(&Move::from_uci("f6g8").expect("valid move"));
        assert!(table.record(position.compute_hash()));
    }
}
