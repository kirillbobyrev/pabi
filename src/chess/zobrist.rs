//! Implements Zobrist hashing and [Transposition Table] functionality.
//!
//! [Transposition Table](https://www.chessprogramming.org/Transposition_Table

use super::position::Position;
use std::hash::{Hash, Hasher};

pub type Key = u64;

pub struct Entry {}

pub struct TranspositionTable {}

impl TranspositionTable {
    fn new() -> Self {
        todo!()
    }

    fn clear(&mut self) {
        todo!()
    }

    fn probe(&self, key: u64) -> Option<&Entry> {
        todo!()
    }

    fn store(&mut self, key: u64, entry: Entry) {
        todo!()
    }
}

impl Hash for Position {
    fn hash<H: Hasher>(&self, state: &mut H) {
        todo!()
    }
}
