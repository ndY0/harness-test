use crate::maze::{is_wall, Direction, MazeGrid, Tile};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacManState {
    Alive,
    #[allow(dead_code)]
    Dying(u8),
    #[allow(dead_code)]
    Respawning,
}

#[derive(Debug, Clone, Copy)]
pub struct PacMan {
    pub pos: (usize, usize),
    pub dir: Direction,
    pub next_dir: Direction,
    pub state: PacManState,
}

impl PacMan {
    pub fn new(pos: (usize, usize)) -> Self {
        PacMan {
            pos,
            dir: Direction::None,
            next_dir: Direction::None,
            state: PacManState::Alive,
        }
    }

    pub fn reset(&mut self, spawn: (usize, usize)) {
        self.pos = spawn;
        self.dir = Direction::None;
        self.next_dir = Direction::None;
        self.state = PacManState::Alive;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GhostPersonality {
    Blinky,
    Pinky,
    Inky,
    Clyde,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GhostMode {
    Chase,
    #[allow(dead_code)]
    Scatter,
    Frightened(u32),
    Eaten,
}

#[derive(Debug, Clone, Copy)]
pub struct Ghost {
    pub pos: (usize, usize),
    pub dir: Direction,
    pub personality: GhostPersonality,
    pub mode: GhostMode,
    #[allow(dead_code)]
    pub spawn: (usize, usize),
    pub tick_counter: u8,
}

impl Ghost {
    pub fn new(pos: (usize, usize), personality: GhostPersonality) -> Self {
        Ghost {
            pos,
            dir: Direction::Up,
            personality,
            mode: GhostMode::Chase,
            spawn: pos,
            tick_counter: 0,
        }
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.pos = self.spawn;
        self.dir = Direction::Up;
        self.mode = GhostMode::Chase;
        self.tick_counter = 0;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionEvent {
    GhostContact,
    GhostEaten,
    #[allow(dead_code)]
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveResult {
    Moved((usize, usize)),
    Blocked,
    PortalTeleport((usize, usize)),
    #[allow(dead_code)]
    Dying,
}

pub fn move_pacman(pacman: &mut PacMan, maze: &MazeGrid) -> MoveResult {
    if pacman.state != PacManState::Alive {
        return MoveResult::Blocked;
    }

    // Check buffered direction first
    let mut dir = pacman.dir;
    if pacman.next_dir != Direction::None {
        let (dx, dy) = pacman.next_dir.delta();
        let nx = pacman.pos.0 as isize + dx;
        let ny = pacman.pos.1 as isize + dy;
        if nx >= 0 && ny >= 0 && nx < 31 && ny < 31 && !is_wall(maze, (nx as usize, ny as usize)) {
            dir = pacman.next_dir;
            pacman.dir = dir;
            pacman.next_dir = Direction::None;
        }
    }

    if dir == Direction::None {
        return MoveResult::Blocked;
    }

    let (dx, dy) = dir.delta();
    let nx = pacman.pos.0 as isize + dx;
    let ny = pacman.pos.1 as isize + dy;

    if nx < 0 || ny < 0 || nx >= 31 || ny >= 31 {
        return MoveResult::Blocked;
    }

    let next_pos = (nx as usize, ny as usize);

    if is_wall(maze, next_pos) {
        return MoveResult::Blocked;
    }

    // Check for portal
    if let Tile::Portal(_) = maze.tiles[next_pos.1][next_pos.0] {
        pacman.pos = next_pos;
        return MoveResult::PortalTeleport(next_pos);
    }

    pacman.pos = next_pos;
    MoveResult::Moved(next_pos)
}

pub fn check_collisions(pacman: &PacMan, ghosts: &[Ghost]) -> Vec<CollisionEvent> {
    let mut events = Vec::new();
    for ghost in ghosts {
        if ghost.pos == pacman.pos {
            match ghost.mode {
                GhostMode::Frightened(_) => {
                    events.push(CollisionEvent::GhostEaten);
                }
                GhostMode::Chase | GhostMode::Scatter => {
                    if pacman.state == PacManState::Alive {
                        events.push(CollisionEvent::GhostContact);
                    }
                }
                GhostMode::Eaten => {}
            }
        }
    }
    events
}

#[allow(dead_code)]
pub fn wrap_direction(d: Direction) -> Direction {
    match d {
        Direction::Up => Direction::Down,
        Direction::Down => Direction::Up,
        Direction::Left => Direction::Right,
        Direction::Right => Direction::Left,
        Direction::None => Direction::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::maze::Tile;

    #[allow(clippy::needless_range_loop)]
    fn simple_maze() -> MazeGrid {
        let mut tiles = [[Tile::Wall; 31]; 31];
        for y in 1..30 {
            for x in 1..30 {
                tiles[y][x] = Tile::Dot;
            }
        }
        // Inner wall
        for x in 5..10 {
            tiles[10][x] = Tile::Wall;
        }
        MazeGrid::new(tiles)
    }

    #[test]
    fn test_pacman_new() {
        let pacman = PacMan::new((15, 23));
        assert_eq!(pacman.pos, (15, 23));
        assert_eq!(pacman.dir, Direction::None);
        assert_eq!(pacman.state, PacManState::Alive);
    }

    #[test]
    fn test_pacman_move_right() {
        let maze = simple_maze();
        let mut pacman = PacMan::new((15, 23));
        pacman.dir = Direction::Right;
        let result = move_pacman(&mut pacman, &maze);
        assert_eq!(result, MoveResult::Moved((16, 23)));
        assert_eq!(pacman.pos, (16, 23));
    }

    #[test]
    fn test_pacman_move_into_wall() {
        let maze = simple_maze();
        let mut pacman = PacMan::new((4, 10)); // left of wall at x=5..10, y=10
        pacman.dir = Direction::Right;
        let result = move_pacman(&mut pacman, &maze);
        assert_eq!(result, MoveResult::Blocked);
        assert_eq!(pacman.pos, (4, 10)); // didn't move
    }

    #[test]
    fn test_buffered_direction_change() {
        let maze = simple_maze();
        let mut pacman = PacMan::new((15, 23));
        pacman.dir = Direction::Right;
        pacman.next_dir = Direction::Up;
        // Moving up from (15,23) should work since (15,22) is not a wall
        let result = move_pacman(&mut pacman, &maze);
        assert_eq!(result, MoveResult::Moved((15, 22)));
        assert_eq!(pacman.dir, Direction::Up);
        assert_eq!(pacman.next_dir, Direction::None);
    }

    #[test]
    fn test_buffered_direction_blocked() {
        let maze = simple_maze();
        let mut pacman = PacMan::new((15, 23));
        pacman.dir = Direction::Right;
        pacman.next_dir = Direction::Left; // back into previous cell
                                           // Buffered direction Left is valid (no wall), so pacman turns around
        let result = move_pacman(&mut pacman, &maze);
        assert_eq!(result, MoveResult::Moved((14, 23)));
    }

    #[test]
    fn test_collision_ghost_contact() {
        let pacman = PacMan::new((10, 10));
        let ghosts = vec![Ghost {
            pos: (10, 10),
            dir: Direction::Up,
            personality: GhostPersonality::Blinky,
            mode: GhostMode::Chase,
            spawn: (5, 5),
            tick_counter: 0,
        }];
        let events = check_collisions(&pacman, &ghosts);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], CollisionEvent::GhostContact);
    }

    #[test]
    fn test_collision_ghost_eaten() {
        let pacman = PacMan::new((10, 10));
        let ghosts = vec![Ghost {
            pos: (10, 10),
            dir: Direction::Up,
            personality: GhostPersonality::Blinky,
            mode: GhostMode::Frightened(50),
            spawn: (5, 5),
            tick_counter: 0,
        }];
        let events = check_collisions(&pacman, &ghosts);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], CollisionEvent::GhostEaten);
    }

    #[test]
    fn test_no_collision() {
        let pacman = PacMan::new((10, 10));
        let ghosts = vec![Ghost {
            pos: (12, 12),
            dir: Direction::Up,
            personality: GhostPersonality::Blinky,
            mode: GhostMode::Chase,
            spawn: (5, 5),
            tick_counter: 0,
        }];
        let events = check_collisions(&pacman, &ghosts);
        assert!(events.is_empty());
    }

    #[test]
    fn test_collision_eaten_ghost_ignored() {
        let pacman = PacMan::new((10, 10));
        let ghosts = vec![Ghost {
            pos: (10, 10),
            dir: Direction::Up,
            personality: GhostPersonality::Blinky,
            mode: GhostMode::Eaten,
            spawn: (5, 5),
            tick_counter: 0,
        }];
        let events = check_collisions(&pacman, &ghosts);
        assert!(events.is_empty());
    }
}
