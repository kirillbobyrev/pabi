//! Interface for Reinforcement Learning environment to abstract the chess
//! rules implementation.

/// Result of the game from the perspective of the player to move at root.
pub enum GameResult {
    Win,
    Loss,
    Draw,
}

// TODO: Require features tensor?
pub trait Observation {}

pub trait Action: Sized {
    fn get_index(&self) -> u16;
}

/// Standard gym-like Reinforcement Learning environment interface.
pub trait Environment<A: Action, O: Observation>: Sized {
    fn actions(&self) -> &[A];
    fn apply(&mut self, action: impl Action) -> O;
    fn result(&self) -> Option<GameResult>;
}
