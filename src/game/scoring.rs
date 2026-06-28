pub struct ScoreState {
    pub score: u32,
    pub ghost_chain: u8,
    pub extra_life_awarded: bool,
}

impl ScoreState {
    pub fn new() -> Self {
        Self { score: 0, ghost_chain: 0, extra_life_awarded: false }
    }

    /// +10 points for eating a dot.
    pub fn add_dot(&mut self) {
        self.score += 10;
    }

    /// +50 points for eating a power pellet; resets ghost chain to 0.
    pub fn add_power_pellet(&mut self) {
        self.score += 50;
        self.ghost_chain = 0;
    }

    /// Returns 200 * 2^ghost_chain points, then increments ghost_chain (capped at 3).
    pub fn eat_ghost(&mut self) -> u32 {
        let points = 200u32 * (1u32 << self.ghost_chain);
        if self.ghost_chain < 3 {
            self.ghost_chain += 1;
        }
        points
    }

    /// Resets ghost chain; called on player death or frightened mode expiry.
    pub fn reset_chain(&mut self) {
        self.ghost_chain = 0;
    }

    /// Returns true once when score first reaches >= 10 000. Sets extra_life_awarded on first call.
    pub fn check_extra_life(&mut self) -> bool {
        if !self.extra_life_awarded && self.score >= 10_000 {
            self.extra_life_awarded = true;
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_initialises_correctly() {
        let s = ScoreState::new();
        assert_eq!(s.score, 0);
        assert_eq!(s.ghost_chain, 0);
        assert!(!s.extra_life_awarded);
    }

    #[test]
    fn add_dot_ten_times_gives_100() {
        let mut s = ScoreState::new();
        for _ in 0..10 {
            s.add_dot();
        }
        assert_eq!(s.score, 100);
    }

    #[test]
    fn add_power_pellet_adds_50_and_resets_chain() {
        let mut s = ScoreState::new();
        s.ghost_chain = 2;
        s.add_power_pellet();
        assert_eq!(s.score, 50);
        assert_eq!(s.ghost_chain, 0);
    }

    #[test]
    fn eat_ghost_chain_200_400_800_1600() {
        let mut s = ScoreState::new();
        assert_eq!(s.eat_ghost(), 200);
        assert_eq!(s.eat_ghost(), 400);
        assert_eq!(s.eat_ghost(), 800);
        assert_eq!(s.eat_ghost(), 1600);
    }

    #[test]
    fn reset_chain_sets_to_zero() {
        let mut s = ScoreState::new();
        s.ghost_chain = 3;
        s.reset_chain();
        assert_eq!(s.ghost_chain, 0);
    }

    #[test]
    fn check_extra_life_returns_true_once_at_10000() {
        let mut s = ScoreState::new();
        s.score = 10_000;
        assert!(s.check_extra_life());
        assert!(!s.check_extra_life());
    }

    #[test]
    fn check_extra_life_returns_false_below_threshold() {
        let mut s = ScoreState::new();
        s.score = 9_999;
        assert!(!s.check_extra_life());
    }
}
