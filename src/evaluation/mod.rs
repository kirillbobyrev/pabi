//! This module implements "static" [evaluation], i.e. predicting the relative
//! value/score of given position without [`crate::search`].
//!
//! For convenience, the score is returned in centipawn units.
//!
//! [evaluation]: https://www.chessprogramming.org/Evaluation

use std::fmt::Display;
use std::ops::Neg;

pub(crate) mod brain;
pub(crate) mod features;
pub(crate) mod pesto;

/// A thin wrapper around i32 for ergonomics and type safety.
// TODO: Use i16 once the evaluation is NN-based.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Score(i32);

impl Score {
    pub(crate) const DRAW: Self = Self(0);
    pub(crate) const INFINITY: Self = Self(2_000_000_000);
    /// `[-INFINITY, -INFINITY + MATE_RANGE)` and `(INFINITY - MATE_RANGE,
    /// INFINITY]` are reserved for mate scores.
    /// `[-INFINITY + MATE_RANGE, INFINITY - MATE_RANGE]` if for centipawn
    /// evaluations.
    const MATE_RANGE: i32 = 1000;

    /// Creates a new score in centipawn units. Centipawn units do not mean in
    /// terms of NNUE evaluation, but it is convenient for GUIs and UCI
    /// purposes, as well as human intepretation.
    ///
    /// The value must be in the range `[-INFINITY + MATE_RANGE, INFINITY -
    /// MATE_RANGE]`.
    #[must_use] pub fn cp(value: i32) -> Self {
        assert!(value.abs() < Self::INFINITY.0 - Self::MATE_RANGE);
        Self(value)
    }

    /// Creates a new score representing player's victory in `moves` *full*
    /// moves.
    #[must_use] pub fn mate(moves: u8) -> Self {
        Self(Self::INFINITY.0 - i32::from(moves))
    }

    /// Returns the number of moves until mate.
    ///
    /// # Panics
    ///
    /// Panics if the score is not a mate score.
    #[must_use] pub fn mate_in(&self) -> u8 {
        assert!(self.is_mate());
        todo!()
    }

    /// Returns `true` if the score represents a mate, not centipawn evaluation.
    #[must_use] pub fn is_mate(&self) -> bool {
        self.0.abs() >= Self::INFINITY.0 - Self::MATE_RANGE
    }
}

impl Neg for Score {
    type Output = Self;

    /// Mirrors evaluation to other player's perspective.
    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl Display for Score {
    /// Formats the score as centipawn units for UCI interface.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_mate() {
            write!(f, "mate {}", self.mate_in())
        } else {
            write!(f, "cp {}", self.0)
        }
    }
}
