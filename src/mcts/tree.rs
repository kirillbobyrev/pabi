use crate::{environment::Action, evaluation::QValue};

struct Tree {
    nodes: Vec<Node>,
}

type NodeIndex = usize;
// This is a special value that is used to indicate that the node has no parent.
const TOMBSTONE_PARENT: NodeIndex = usize::MAX;

// TODO: Measure the performance and see if switching to ArrayVec will make it
// faster.
struct Node {
    parent: NodeIndex,
    children: Vec<NodeIndex>,
    // Use Win-Draw-Loss evaluation, similar to lc0:
    // https://lczero.org/blog/2020/04/wdl-head/
    w_count: u32,
    d_count: u32,
    l_count: u32,
    visits: u32,
}

impl Node {
    #[must_use]
    const fn visited(&self) -> bool {
        self.visits > 0
    }

    #[must_use]
    const fn q_value(action: impl Action) -> QValue {
        todo!()
    }
}
