//! Implements Zobrist hashing and [Transposition Table] functionality.
//!
//! [Transposition Table](https://www.chessprogramming.org/Transposition_Table

use crate::chess::zobrist::Key;

pub(super) struct Entry {}

// TODO: Migrate to RawTable instead for better performance?
pub(super) struct TranspositionTable {}

impl TranspositionTable {
    #[must_use]
    pub(super) fn new() -> Self {
        todo!()
    }

    pub(super) fn clear(&mut self) {
        todo!()
    }

    #[must_use]
    pub(super) fn probe(&self, key: Key) -> Option<&Entry> {
        todo!()
    }

    pub(super) fn store(&mut self, key: Key, entry: Entry) {
        todo!()
    }
}
