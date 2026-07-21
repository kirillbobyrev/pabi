//! Zobrist hashing-related utilities.

/// Zobrist keys are 64-bit unsigned integers that are computed once a position
/// is created and updated incrementally whenever a move is made.
pub type Key = u64;
