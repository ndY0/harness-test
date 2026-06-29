#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScoreEvent {
    Dot,
    PowerPellet,
    Ghost,
    Fruit(u32),
    LevelComplete(u8),
}

#[derive(Debug, Clone)]
pub struct Score {
    pub score: u32,
    pub lives: u8,
    pub ghost_eat_chain: u8,
}

impl Score {
    pub fn new() -> Self {
        Score {
            score: 0,
            lives: 3,
            ghost_eat_chain: 0,
        }
    }
}

pub fn add_points(score: &mut Score, event: ScoreEvent) {
    match event {
        ScoreEvent::Dot => score.score += 10,
        ScoreEvent::PowerPellet => score.score += 50,
        ScoreEvent::Ghost => {
            let points = match score.ghost_eat_chain {
                0 => 200,
                1 => 400,
                2 => 800,
                _ => 1600,
            };
            score.score += points;
            score.ghost_eat_chain = score.ghost_eat_chain.saturating_add(1);
        }
        ScoreEvent::Fruit(val) => score.score += val,
        ScoreEvent::LevelComplete(level) => {
            score.score += level_bonus(level);
        }
    }
}

pub fn lose_life(lives: &mut u8) -> bool {
    if *lives > 0 {
        *lives -= 1;
    }
    *lives == 0
}

pub fn reset_ghost_eat_chain(score: &mut Score) {
    score.ghost_eat_chain = 0;
}

pub fn level_bonus(level: u8) -> u32 {
    100 * level as u32
}

pub fn fruit_value(level: u8) -> u32 {
    match level {
        1 => 100,
        2 => 300,
        3 => 500,
        4 => 700,
        5 => 1000,
        6 => 2000,
        7 => 3000,
        8 => 5000,
        9 => 5000,
        10 => 5000,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dot_scoring() {
        let mut score = Score::new();
        add_points(&mut score, ScoreEvent::Dot);
        assert_eq!(score.score, 10);
    }

    #[test]
    fn test_power_pellet_scoring() {
        let mut score = Score::new();
        add_points(&mut score, ScoreEvent::PowerPellet);
        assert_eq!(score.score, 50);
    }

    #[test]
    fn test_ghost_eat_chain() {
        let mut score = Score::new();
        // First ghost: 200
        add_points(&mut score, ScoreEvent::Ghost);
        assert_eq!(score.score, 200);
        assert_eq!(score.ghost_eat_chain, 1);

        // Second ghost: 400
        add_points(&mut score, ScoreEvent::Ghost);
        assert_eq!(score.score, 600);
        assert_eq!(score.ghost_eat_chain, 2);

        // Third ghost: 800
        add_points(&mut score, ScoreEvent::Ghost);
        assert_eq!(score.score, 1400);
        assert_eq!(score.ghost_eat_chain, 3);

        // Fourth ghost: 1600
        add_points(&mut score, ScoreEvent::Ghost);
        assert_eq!(score.score, 3000);
        assert_eq!(score.ghost_eat_chain, 4);
    }

    #[test]
    fn test_ghost_eat_chain_reset() {
        let mut score = Score::new();
        add_points(&mut score, ScoreEvent::Ghost);
        add_points(&mut score, ScoreEvent::Ghost);
        assert_eq!(score.ghost_eat_chain, 2);

        reset_ghost_eat_chain(&mut score);
        assert_eq!(score.ghost_eat_chain, 0);

        // After reset, next ghost starts at 200 again
        add_points(&mut score, ScoreEvent::Ghost);
        assert_eq!(score.score, 800); // 200+400+200
    }

    #[test]
    fn test_fruit_scoring() {
        let mut score = Score::new();
        add_points(&mut score, ScoreEvent::Fruit(100));
        assert_eq!(score.score, 100);
        add_points(&mut score, ScoreEvent::Fruit(5000));
        assert_eq!(score.score, 5100);
    }

    #[test]
    fn test_level_complete_bonus() {
        let mut score = Score::new();
        add_points(&mut score, ScoreEvent::LevelComplete(5));
        assert_eq!(score.score, 500);
    }

    #[test]
    fn test_lose_life() {
        let mut score = Score::new();
        assert_eq!(score.lives, 3);

        assert!(!lose_life(&mut score.lives));
        assert_eq!(score.lives, 2);

        assert!(!lose_life(&mut score.lives));
        assert_eq!(score.lives, 1);

        assert!(lose_life(&mut score.lives));
        assert_eq!(score.lives, 0);
    }

    #[test]
    fn test_lose_life_at_zero() {
        let mut lives = 0u8;
        assert!(lose_life(&mut lives));
        assert_eq!(lives, 0);
    }

    #[test]
    fn test_fruit_value_per_level() {
        assert_eq!(fruit_value(1), 100);
        assert_eq!(fruit_value(2), 300);
        assert_eq!(fruit_value(3), 500);
        assert_eq!(fruit_value(4), 700);
        assert_eq!(fruit_value(5), 1000);
        assert_eq!(fruit_value(6), 2000);
        assert_eq!(fruit_value(7), 3000);
        assert_eq!(fruit_value(8), 5000);
        assert_eq!(fruit_value(9), 5000);
        assert_eq!(fruit_value(10), 5000);
    }

    #[test]
    fn test_level_bonus_values() {
        assert_eq!(level_bonus(1), 100);
        assert_eq!(level_bonus(5), 500);
        assert_eq!(level_bonus(10), 1000);
    }

    #[test]
    fn test_score_new() {
        let score = Score::new();
        assert_eq!(score.score, 0);
        assert_eq!(score.lives, 3);
        assert_eq!(score.ghost_eat_chain, 0);
    }
}
