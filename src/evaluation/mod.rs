//! This module implements "static" [evaluation], i.e. predicting the relative
//! value/score of given position without [`crate::search`].
//!
//! For convenience, the score is returned in centipawn units.
//!
//! [evaluation]: https://www.chessprogramming.org/Evaluation

use std::fmt::Display;
use std::ops::Neg;

pub(crate) mod material;

// TODO: Document: a thin wrapper around i32, same size and ergonomics
// for performance reasons.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Score {
    /// Evaluation relative value in centipawn (100 CP = 1 "pawn") units.
    value: i32,
}

impl Score {
    pub const LOSE: Self = Self { value: -32_000 };
    pub const MAX: Self = Self::WIN;
    pub const MIN: Self = Self::LOSE;
    pub const WIN: Self = Self { value: 32_000 };
}

impl Neg for Score {
    type Output = Self;

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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "cp {}", self.value)
    }
}
