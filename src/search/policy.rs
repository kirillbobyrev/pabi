//! Action selection policy for the tree traversal phase of MCTS.
//!
//! Implements the [PUCT] (Polynomial Upper Confidence Trees) rule used by
//! AlphaZero: children are selected by maximizing $Q(s, a) + U(s, a)$, where
//! $U(s, a) = c_{puct} \cdot P(s, a) \cdot \frac{\sqrt{N(s)}}{1 + N(s, a)}$.
//!
//! [PUCT]: https://www.chessprogramming.org/UCT#PUCT

use super::tree::{NodeId, Tree};

/// Selects the most promising child of an expanded node according to the PUCT
/// formula.
///
/// # Panics
///
/// Panics if the node has no children (the caller must only select from
/// expanded, non-terminal nodes).
pub(super) fn select_child(tree: &Tree, parent: NodeId, cpuct: f32) -> NodeId {
    let parent_node = tree.node(parent);
    debug_assert!(
        !parent_node.children.is_empty(),
        "can only select from expanded nodes"
    );

    let sqrt_parent_visits = (parent_node.visits.max(1) as f32).sqrt();

    let mut best_child = parent_node.children[0];
    let mut best_score = f32::NEG_INFINITY;

    for &child_id in &parent_node.children {
        let child = tree.node(child_id);
        #[allow(
            clippy::cast_possible_truncation,
            reason = "Q is in [-1, 1], truncation cannot happen"
        )]
        let exploitation = child.mean_value() as f32;
        let exploration = cpuct * child.prior * sqrt_parent_visits / (1.0 + child.visits as f32);
        let score = exploitation + exploration;
        if score > best_score {
            best_score = score;
            best_child = child_id;
        }
    }

    best_child
}
