use std::rc::Weak;

use crate::environment::Action;

struct Tree<A: Action> {
    nodes: Node<A>,
}

// TODO: Measure the performance and see if switching to ArrayVec will make it
// faster.
/// Use Win-Draw-Loss (WDL) evaluation, similar to lc0:
/// https://lczero.org/blog/2020/04/wdl-head/
struct Node<A: Action> {
    children: Vec<Node<A>>,
    actions: Vec<A>,
    /// Number of wins from the perspective of player to move.
    wins: u16,
    /// Number of draws in all searches that went through this node.
    draws: u16,
    /// Number of losses from the perspective of player to move.
    losses: u16,
    /// Total number of search iterations that went through this node.
    visits: u16,
}

impl<A: Action> Node<A> {
    #[must_use]
    const fn visited(&self) -> bool {
        self.visits > 0
    }

    #[must_use]
    const fn is_leaf(&self) -> bool {
        todo!()
    }
}
