//! Implements Zobrist hashing and [Transposition Table] functionality.
//!
//! [Transposition Table]: https://www.chessprogramming.org/Transposition_Table

use std::collections::HashMap;

use crate::chess::zobrist::Key;
use crate::evaluation::Score;

pub(super) struct Entry {
    pub(super) depth: u8,
    pub(super) score: Score,
    pub(super) best_move: Option<u16>,
    pub(super) bound: Bound,
    pub(super) flags: u8,
}

pub(super) enum Bound {
    Exact,
    Lower,
    Upper,
}

pub(super) struct TranspositionTable {
    // TODO: Migrate to RawTable instead for better performance?
    table: HashMap<Key, Entry>,
    size: usize,
}

impl TranspositionTable {
    #[must_use]
    pub(super) fn new(size: usize) -> Self {
        Self {
            table: HashMap::with_capacity(size),
            size,
        }
    }

    pub(super) fn clear(&mut self) {
        self.table.clear();
    }

    #[must_use]
    pub(super) fn probe(&self, key: Key) -> Option<&Entry> {
        todo!()
    }

    pub(super) fn store(&mut self, key: Key, entry: Entry) {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn clear() {
        todo!()
    }
}
