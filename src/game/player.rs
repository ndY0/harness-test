use crate::game::{Direction, Position};

pub struct Player {
    pub pos: Position,
    pub facing: Option<Direction>,
    pub buffered_dir: Option<Direction>,
    pub lives: u8,
}

impl Player {
    pub fn new(start: Position) -> Self {
        Self {
            pos: start,
            facing: None,
            buffered_dir: None,
            lives: 3,
        }
    }

    pub fn buffer_direction(&mut self, dir: Direction) {
        self.buffered_dir = Some(dir);
    }

    pub fn consume_direction(&mut self) -> Option<Direction> {
        self.buffered_dir.take()
    }

    pub fn lose_life(&mut self) {
        self.lives = self.lives.saturating_sub(1);
    }

    pub fn is_dead(&self) -> bool {
        self.lives == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{Direction, Position};

    fn start() -> Position {
        Position { row: 7, col: 10 }
    }

    #[test]
    fn new_creates_player_with_defaults() {
        let p = Player::new(start());
        assert_eq!(p.pos, start());
        assert_eq!(p.lives, 3);
        assert!(p.facing.is_none());
        assert!(p.buffered_dir.is_none());
    }

    #[test]
    fn buffer_then_consume_returns_direction_and_clears() {
        let mut p = Player::new(start());
        p.buffer_direction(Direction::Left);
        assert_eq!(p.consume_direction(), Some(Direction::Left));
        assert!(p.buffered_dir.is_none());
    }

    #[test]
    fn consume_on_empty_returns_none() {
        let mut p = Player::new(start());
        assert_eq!(p.consume_direction(), None);
    }

    #[test]
    fn lose_life_decrements_and_saturates_at_zero() {
        let mut p = Player::new(start());
        p.lose_life();
        assert_eq!(p.lives, 2);
        p.lose_life();
        assert_eq!(p.lives, 1);
        p.lose_life();
        assert_eq!(p.lives, 0);
        p.lose_life();
        assert_eq!(p.lives, 0);
    }

    #[test]
    fn is_dead_true_only_when_lives_zero() {
        let mut p = Player::new(start());
        assert!(!p.is_dead());
        p.lose_life();
        assert!(!p.is_dead());
        p.lose_life();
        assert!(!p.is_dead());
        p.lose_life();
        assert!(p.is_dead());
    }
}
