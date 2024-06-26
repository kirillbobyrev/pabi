//! This module implements "static" [evaluation], i.e. predicting the relative
//! value/score of given position without [`crate::search`].
//!
//! For convenience, the score is returned in centipawn units.
//!
//! [evaluation]: https://www.chessprogramming.org/Evaluation

pub(crate) mod brain;
pub(crate) mod features;

mod score;
pub use score::Score;
