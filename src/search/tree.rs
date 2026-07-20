//! Search tree structure for Monte Carlo Tree Search.
//!
//! The tree is stored as an arena ([`Tree`]) of [`Node`]s referencing their
//! children by index. Each node stores the statistics of the *edge* leading
//! into it (the action taken from its parent), following the AlphaZero
//! convention: `visits` is $N(s, a)$, `total_value` is $W(s, a)$ and the mean
//! value $Q(s, a)$ is valued from the perspective of the player to move at the
//! parent node.

use crate::chess::core::Move;

/// Index of a node in the [`Tree`] arena.
pub(super) type NodeId = u32;

/// Index of the root node: the tree always contains the root at index 0.
pub(super) const ROOT_ID: NodeId = 0;

pub(super) struct Node {
    /// The action that leads to this node. `None` only for the root.
    pub(super) action: Option<Move>,
    /// Arena indices of the children. Empty until the node is expanded.
    pub(super) children: Vec<NodeId>,
    /// Number of search iterations that went through this node.
    pub(super) visits: u32,
    /// Sum of backed-up values, from the perspective of the player to move at
    /// the parent node.
    pub(super) total_value: f64,
    /// Prior probability of selecting this node's action from the parent.
    pub(super) prior: f32,
    /// Exact value of the node if it is terminal (checkmate, stalemate or a
    /// draw by the rules), from the perspective of the player to move at this
    /// node's position.
    pub(super) terminal: Option<f64>,
    /// Whether the node's children have been generated.
    pub(super) expanded: bool,
}

impl Node {
    fn new(action: Option<Move>, prior: f32) -> Self {
        Self {
            action,
            children: Vec::new(),
            visits: 0,
            total_value: 0.0,
            prior,
            terminal: None,
            expanded: false,
        }
    }

    /// Mean action value $Q(s, a)$ from the perspective of the player to move
    /// at the parent node. Unvisited nodes have a neutral value.
    #[must_use]
    pub(super) fn mean_value(&self) -> f64 {
        if self.visits == 0 {
            0.0
        } else {
            self.total_value / f64::from(self.visits)
        }
    }
}

pub(super) struct Tree {
    nodes: Vec<Node>,
}

impl Tree {
    /// Creates a tree containing only an unexpanded root node.
    pub(super) fn new() -> Self {
        Self {
            nodes: vec![Node::new(None, 1.0)],
        }
    }

    pub(super) fn node(&self, id: NodeId) -> &Node {
        &self.nodes[id as usize]
    }

    pub(super) fn node_mut(&mut self, id: NodeId) -> &mut Node {
        &mut self.nodes[id as usize]
    }

    /// Adds a child node for the given action and returns its id.
    pub(super) fn add_child(&mut self, parent: NodeId, action: Move, prior: f32) -> NodeId {
        let id = NodeId::try_from(self.nodes.len()).expect("search tree exceeded u32 capacity");
        self.nodes.push(Node::new(Some(action), prior));
        self.nodes[parent as usize].children.push(id);
        id
    }
}
