use arrayvec::ArrayVec;

use crate::chess::position::Position;

pub(crate) mod minimax;

pub(crate) struct SearchState {
    stack: ArrayVec<Position, 256>,
}

impl SearchState {
    pub(crate) fn new() -> Self {
        Self {
            stack: ArrayVec::<Position, 256>::new(),
        }
    }

    pub(crate) fn root<'a>() -> &'a Position {
        todo!()
    }
}
