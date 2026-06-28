pub mod scoring;

/// Integer tile coordinates (row, col).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

/// The four cardinal movement directions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

/// What a maze tile contains.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cell {
    Wall,
    Dot,
    PowerPellet,
    PlayerStart,
    GhostHouse,
    Empty,
}

/// The three behavioural modes a ghost can be in.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GhostMode {
    Chase,
    Scatter,
    Frightened,
}

/// Semantic game commands produced by the input mapping layer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameAction {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    Pause,
    Quit,
}
