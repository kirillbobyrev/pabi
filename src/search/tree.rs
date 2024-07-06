use crate::environment::Action;

/// Node stores (wins, draws, losses) statistics instead of expanded "value"
/// (or total score), which is usually wins + 0.5 * draws.
///
/// For more details, ses https://lczero.org/blog/2020/04/wdl-head/
// TODO: Measure the performance and see if switching to ArrayVec will make it
// faster.
pub(super) struct Node<A: Action> {
    children: Vec<Node<A>>,
    actions: Vec<A>,
    prior: f32,
    /// Total number of search iterations that went through this node.
    visits: u32,
    /// Number of wins from the perspective of player to move.
    wins: u32,
    /// Number of losses from the perspective of player to move.
    losses: u32,
}

impl<A: Action> Default for Node<A> {
    fn default() -> Self {
        Self {
            children: Vec::new(),
            actions: Vec::new(),
            prior: 0.0,
            visits: 0,
            wins: 0,
            losses: 0,
        }
    }
}

impl<A: Action> Node<A> {
    fn expand(&mut self) {
        todo!()
    }

    /// Returns true if the node has been visited at least once.
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

    #[must_use]
    const fn draws(&self) -> u32 {
        self.visits - self.wins - self.losses
    }
}
