use std::rc::Weak;

use crate::environment::Action;

struct Tree<A: Action> {
    nodes: Node<A>,
}

// TODO: Measure the performance and see if switching to ArrayVec will make it
// faster.
struct Node<A: Action> {
    parent: Weak<Node<A>>,
    children: Vec<Box<Node<A>>>,
    actions: Vec<A>,
    // Use Win-Draw-Loss (WDL) evaluation, similar to lc0:
    // https://lczero.org/blog/2020/04/wdl-head/
    wins: u32,
    draws: u32,
    losses: u32,
    visits: u32,
}

impl<A: Action> Node<A> {
    #[must_use]
    const fn visited(&self) -> bool {
        self.visits > 0
    }
}
