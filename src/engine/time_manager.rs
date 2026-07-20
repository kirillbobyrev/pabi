//! Decides how much time to spend on a move given the game time controls.

use std::time::Duration;

/// A safety margin subtracted from the remaining time to account for the
/// communication overhead between the engine and the tournament manager/GUI.
const MOVE_OVERHEAD: Duration = Duration::from_millis(10);

/// Returns the time budget for the next move.
///
/// The heuristic assumes the game will last around 20 more moves and spends
/// the increment eagerly, while never using more than half of the remaining
/// time on a single move.
#[must_use]
pub(super) fn time_budget(remaining: Duration, increment: Duration) -> Duration {
    const MINIMUM_BUDGET: Duration = Duration::from_millis(1);
    let usable = remaining.saturating_sub(MOVE_OVERHEAD);
    let budget = usable / 20 + increment / 2;
    budget.clamp(MINIMUM_BUDGET, (usable / 2).max(MINIMUM_BUDGET))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn budget_is_a_fraction_of_remaining_time() {
        let budget = time_budget(Duration::from_secs(60), Duration::ZERO);
        assert!(budget >= Duration::from_secs(2));
        assert!(budget <= Duration::from_secs(30));
    }

    #[test]
    fn increment_is_partially_spent() {
        let with_increment = time_budget(Duration::from_secs(60), Duration::from_secs(2));
        let without_increment = time_budget(Duration::from_secs(60), Duration::ZERO);
        assert!(with_increment > without_increment);
    }

    #[test]
    fn never_exceeds_half_of_remaining_time() {
        let budget = time_budget(Duration::from_millis(100), Duration::from_secs(10));
        assert!(budget <= Duration::from_millis(50));
    }

    #[test]
    fn minimal_budget_with_no_time_left() {
        let budget = time_budget(Duration::ZERO, Duration::ZERO);
        assert_eq!(budget, Duration::from_millis(1));
    }
}
