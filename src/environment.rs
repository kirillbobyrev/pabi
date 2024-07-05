//! Interface for Reinforcement Learning environment to abstract the chess
//! rules implementation.

use std::fmt;
use std::ops::Not;

use anyhow::bail;

/// A standard game of chess is played between two players: White (having the
/// advantage of the first turn) and Black.
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Player {
    White,
    Black,
}

impl Not for Player {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

impl TryFrom<&str> for Player {
    type Error = anyhow::Error;

    fn try_from(color: &str) -> anyhow::Result<Self> {
        match color {
            "w" => Ok(Self::White),
            "b" => Ok(Self::Black),
            _ => bail!("color should be 'w' or 'b', got '{color}'"),
        }
    }
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                Self::White => 'w',
                Self::Black => 'b',
            }
        )
    }
}

/// Result of the game from the perspective of the player to move at root.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameResult {
    Win,
    Draw,
    Loss,
}

// TODO: Require features tensor?
pub trait Observation {}

pub trait Action: Sized {
    fn get_index(&self) -> u16;
}

/// Standard gym-like Reinforcement Learning environment interface.
pub trait Environment<A: Action, O: Observation>: Sized {
    fn actions(&self) -> &[A];
    fn apply(&mut self, action: &A) -> &O;
    fn result(&self) -> Option<GameResult>;
}
