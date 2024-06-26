//! Implements [Monte Carlo Tree Search] (MCTS) algorithm.
//!
//! [Monte Carlo Tree Search]: https://en.wikipedia.org/wiki/Monte_Carlo_tree_search

mod policy;
mod search;
mod tree;

/// Search depth in plies.
pub type Depth = u8;
