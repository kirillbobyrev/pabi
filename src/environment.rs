use crate::chess::core::Color;

/// Result of the game from the perspective of the player to move at root.
enum GameResult {
    Win,
    Loss,
    Draw,
}

trait Action: Sized {
    fn get_index(&self) -> u16;
}

trait Environment {
    fn get_actions<T: Action>(&self) -> Vec<T>;
    fn apply(&mut self, action: impl Action);
    fn get_result(&self) -> Option<GameResult>;
}
