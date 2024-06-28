struct Tree {
    root: Node,
}

type NodeIndex = usize;
// This is a special value that is used to indicate that the node has no parent.
const TOMBSTONE_PARENT: NodeIndex = usize::MAX;

// TODO: Measure the performance and see if switching to ArrayVec will make it
// faster.
struct Node {
    parent: NodeIndex,
    children: Vec<NodeIndex>,
    wins: u32,
    visits: u32,
}
