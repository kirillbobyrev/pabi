use crate::environment::Action;

struct Tree<A: Action> {
    nodes: Node<A>,
}

/// Node stores (wins, draws, losses) statistics instead of expanded "value"
/// (or total score), which is usually wins + 0.5 * draws.
/// https://lczero.org/blog/2020/04/wdl-head/
// TODO: Measure the performance and see if switching to ArrayVec will make it
// faster.
struct Node<A: Action> {
    children: Vec<Node<A>>,
    actions: Vec<A>,
    prior: f32,
    /// Total number of search iterations that went through this node.
    visits: u16,
    /// Number of wins from the perspective of player to move.
    wins: u16,
    /// Number of draws in all searches that went through this node.
    draws: u16,
    /// Number of losses from the perspective of player to move.
    losses: u16,
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

    #[must_use]
    const fn is_terminal(&self) -> bool {
        todo!()
    }
}
