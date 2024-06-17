//! Implements Zobrist hashing and [Transposition Table] functionality.
//!
//! [Transposition Table](https://www.chessprogramming.org/Transposition_Table

use crate::chess::position::Position;
use core::hash;
use std::hash::{Hash, Hasher};

pub type Key = u64;

pub(super) struct Entry {}

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
    pub(super) fn probe(&self, key: u64) -> Option<&Entry> {
        todo!()
    }

    pub(super) fn store(&mut self, key: u64, entry: Entry) {
        todo!()
    }
}
