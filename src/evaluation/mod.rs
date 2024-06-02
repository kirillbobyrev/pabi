//! This module implements "static" [evaluation], i.e. predicting the relative
//! value/score of given position without [crate::search].
//!
//! For convenience, the score is returned in centipawn units.
//!
//! [evaluation]: https://www.chessprogramming.org/Evaluation

pub(crate) mod material;

/// Evaluation relative value in centipawn (100 CP = 1 "pawn") units.
pub(crate) type Value = i32;
