use std::cmp::Ordering;
use std::fmt::Display;
use std::ops::Neg;

/// The score represents the relative value of the position (in centipawns) or
/// checkmate in N moves (if one is found).
///
/// A compact i32 representation is used to store the score in both cases. Since
/// score is stored in [`crate::search::transposition::TranspositionTable`], it
/// is important to keep the size small.
// TODO: Use i16 once the evaluation is NN-based.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Score {
    value: i32,
}

impl Score {
    pub(crate) const DRAW: Self = Self { value: 0 };
    pub(crate) const INFINITY: Self = Self {
        value: 2_000_000_000,
    };
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
    #[must_use]
    pub fn cp(value: i32) -> Self {
        assert!(value.abs() < Self::INFINITY.value - Self::MATE_RANGE);
        Self { value }
    }

    /// Creates a new score representing player's victory in `moves` *full*
    /// moves.
    #[must_use]
    pub fn mate(moves: u8) -> Self {
        Self {
            value: Self::INFINITY.value - i32::from(moves),
        }
    }

    /// Returns the number of moves until mate.
    ///
    /// # Panics
    ///
    /// Panics if the score is not a mate score.
    #[must_use]
    pub fn mate_in(&self) -> i8 {
        assert!(self.is_mate());
        let moves = Self::INFINITY.value - self.value.abs();
        match self.value.cmp(&0) {
            Ordering::Greater => moves as i8,
            Ordering::Less => -moves as i8,
            _ => unreachable!(),
        }
    }

    /// Returns `true` if the score represents a mate, not centipawn evaluation.
    #[must_use]
    pub fn is_mate(&self) -> bool {
        self.value.abs() >= Self::INFINITY.value - Self::MATE_RANGE
    }
}

impl Neg for Score {
    type Output = Self;

    /// Mirrors evaluation to other player's perspective.
    fn neg(self) -> Self::Output {
        Self { value: -self.value }
    }
}

impl Display for Score {
    /// Formats the score as centipawn units for UCI interface.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_mate() {
            write!(f, "mate {}", self.mate_in())
        } else {
            write!(f, "cp {}", self.value)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mate() {
        assert!(Score::mate(42).is_mate());
        assert_eq!(Score::mate(42).mate_in(), 42);
    }

    #[test]
    fn cp() {
        let cp = Score::cp(42);
        assert_eq!(cp, Score { value: 42 });
        assert!(!cp.is_mate());

        assert!(Score::cp(42) < Score::cp(43));
        assert!(Score::cp(0) > Score::cp(-42));
    }

    #[test]
    fn neg() {
        let cp = Score::cp(42);
        assert_eq!(-cp, Score { value: -42 });

        assert_eq!(Score::mate(42).mate_in(), 42);
        assert_eq!(-Score::mate(42).mate_in(), -42);
    }

    #[test]
    fn display() {
        let cp = Score::cp(123);
        assert_eq!(cp.to_string(), "cp 123");

        let mate = Score::mate(3);
        assert_eq!(mate.to_string(), "mate 3");
    }

    #[test]
    fn mate_vs_cp() {
        assert!(Score::mate(42) > Score::cp(42));
        assert!(-Score::mate(1) < Score::cp(42));
        assert!(Score::mate(2) > Score::cp(-42));
    }

    #[test]
    #[should_panic]
    fn cp_panic() {
        let _ = Score::cp(Score::INFINITY.value - Score::MATE_RANGE);
    }
}
