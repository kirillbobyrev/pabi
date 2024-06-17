//! This module implements "static" [evaluation], i.e. predicting the relative
//! value/score of given position without [`crate::search`].
//!
//! For convenience, the score is returned in centipawn units.
//!
//! [evaluation]: https://www.chessprogramming.org/Evaluation

use std::fmt::Display;
use std::ops::Neg;

pub(crate) mod material;

/// A thin wrapper around i32 for ergonomics and type safety.
// TODO: Support "Mate in X" by using the unoccupied range of i32.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Score {
    /// Evaluation relative value in centipawn (100 CP = 1 "pawn") units.
    value: i32,
}

impl Score {
    /// Corresponds to checkmating the opponent.
    pub(crate) const MAX: Self = Self { value: 32_000 };
    /// Corresponds to being checkmated by opponent.
    pub(crate) const MIN: Self = Self { value: -32_000 };
}

impl Neg for Score {
    type Output = Self;

    /// Mirrors evaluation to other player's perspective.
    fn neg(self) -> Self::Output {
        Self {
            value: self.value.neg(),
        }
    }
}

impl From<i32> for Score {
    fn from(value: i32) -> Self {
        Self { value }
    }
}

impl Display for Score {
    /// Formats the score as centipawn units for UCI interface.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "cp {}", self.value)
    }
}
