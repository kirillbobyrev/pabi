use crate::chess::position::Position;

struct Tree {
    root: Node,
}

struct Node {
    position: Position,

    wins: u32,
    visits: u32,
}
