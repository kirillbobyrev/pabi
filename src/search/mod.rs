//! [Search] is a "dynamic" position evaluation and one of the most imporant
//! parts of the engine. It uses move generation and knowledge about the chess
//! rules to efficiently look ahead into possible continuations and their
//! respective static evaluation to combine them into a final score that the
//! engine assigns to the position.
//!
//! [Search]: https://www.chessprogramming.org/Search

use arrayvec::ArrayVec;

use crate::chess::core::Move;
use crate::chess::position::Position;
use crate::evaluation::Value;
use std::ops::Neg;

pub(crate) mod minimax;

/// An evaluation result can be either relative numerical value ([Score]) or a
/// "checkmate in x (moves)" if one is found.
#[derive(PartialEq, Eq, Debug)]
pub(crate) enum Score {
    /// Relative evaluation in centipawn units (see [Score]). Positive value
    /// means an advantage, negative - the opponent has higher chances of
    /// winning.
    Relative(Value),
    /// Represents "checkmate in X" (full moves). Positive value means winning
    /// in X moves, otherwise the engine is losing.
    Checkmate(i16),
}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            // Both results are scores: compare the scores directly.
            (Self::Relative(a), Self::Relative(b)) => a.partial_cmp(b),
            // Both results are checkmates.
            (Self::Checkmate(a), Self::Checkmate(b)) => {
                if a.is_positive() == b.is_positive() {
                    // Reverse: checkmating faster is better, losing slower is better.
                    b.partial_cmp(a)
                } else {
                    // Different signs: whichever is positive is the checkmating side.
                    a.partial_cmp(b)
                }
            },
            // Losing is worse than any score, winning is better than any score.
            (Self::Relative(_), Self::Checkmate(checkmate_in_x)) => {
                if checkmate_in_x.is_positive() {
                    Some(std::cmp::Ordering::Less)
                } else {
                    Some(std::cmp::Ordering::Greater)
                }
            },
            (Self::Checkmate(checkmate_in_x), Self::Relative(_)) => {
                if checkmate_in_x.is_positive() {
                    Some(std::cmp::Ordering::Greater)
                } else {
                    Some(std::cmp::Ordering::Less)
                }
            },
        }
    }
}

impl Neg for Score {
    type Output = Self;

    /// Flips perspective.
    fn neg(self) -> Self::Output {
        match self {
            Self::Relative(value) => Self::Relative(-value),
            Self::Checkmate(checkmate_in_n) => Self::Checkmate(-checkmate_in_n),
        }
    }
}

impl Score {
    pub(crate) fn to_uci(&self) -> String {
        match self {
            Self::Relative(score) => format!("cp {score}"),
            Self::Checkmate(x) => format!("mate {x}"),
        }
    }
}

/// Maximum depth of the search. Realistically, it is probably way lower
/// than 256, but it should not affect performance too much.
const MAX_SEARCH_DEPTH: usize = 256;

pub(crate) struct SearchState {
    /// The stack is quite shallow and is kept on stack rather than heap for
    /// performance.
    stack: ArrayVec<Position, MAX_SEARCH_DEPTH>,
}

#[derive(PartialEq, Eq, Debug)]
pub(crate) struct SearchResult {
    pub(crate) score: Score,
    pub(crate) best_move: Option<Move>,
}

impl SearchState {
    pub(crate) fn new() -> Self {
        Self {
            stack: ArrayVec::<Position, MAX_SEARCH_DEPTH>::new(),
        }
    }

    pub(crate) fn reset(&mut self, starting_position: &Position) {
        self.stack.clear();
        self.stack.push(starting_position.clone())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn score_ordering() {
        assert!(Score::Checkmate(1) > Score::Checkmate(2));
        assert!(Score::Checkmate(10) < Score::Checkmate(5));
        assert!(Score::Checkmate(5) == Score::Checkmate(5));

        assert!(Score::Checkmate(-1) < Score::Checkmate(-2));
        assert!(Score::Checkmate(-2) > Score::Checkmate(-1));
        assert!(Score::Checkmate(-1) == Score::Checkmate(-1));

        assert!(Score::Relative(100) > Score::Relative(50));
        assert!(Score::Relative(1) > Score::Relative(-40));
        assert!(Score::Relative(-1) < Score::Relative(0));
        assert!(Score::Relative(-1) < Score::Relative(20));

        assert!(Score::Checkmate(1) > Score::Relative(20));
        assert!(Score::Checkmate(1) > Score::Relative(-20));

        assert!(Score::Checkmate(1) > Score::Relative(-20));
        assert!(Score::Checkmate(1) > Score::Relative(20));
        assert!(Score::Checkmate(-1) < Score::Relative(20));

        assert!(Score::Checkmate(0) < Score::Checkmate(-1));
        assert!(Score::Checkmate(0) < Score::Checkmate(1));
        assert!(Score::Checkmate(0) < Score::Relative(20));
        assert!(Score::Checkmate(0) < Score::Relative(-20));
    }
}
