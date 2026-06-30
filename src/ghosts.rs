use crate::entities::{Ghost, GhostMode, GhostPersonality, PacMan};
use crate::maze::{is_wall, Direction, MazeGrid};
use rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GhostEvent {
    Moved(usize),
    ReachedLair(usize),
    #[allow(dead_code)]
    None,
}

const SCATTER_TARGET: (usize, usize) = (1, 29);

fn ghost_target(ghost: &Ghost, pacman: &PacMan, blinky_pos: (usize, usize)) -> (usize, usize) {
    match ghost.personality {
        GhostPersonality::Blinky => pacman.pos,
        GhostPersonality::Pinky => {
            let (dx, dy) = pacman.dir.delta();
            let tx = pacman.pos.0 as isize + dx * 4;
            let ty = pacman.pos.1 as isize + dy * 4;
            (tx.clamp(0, 30) as usize, ty.clamp(0, 30) as usize)
        }
        GhostPersonality::Inky => {
            let (dx, dy) = pacman.dir.delta();
            let ahead_x = pacman.pos.0 as isize + dx * 2;
            let ahead_y = pacman.pos.1 as isize + dy * 2;
            let ahead = (ahead_x.clamp(0, 30) as usize, ahead_y.clamp(0, 30) as usize);
            // Double the vector from Blinky to 2-ahead
            let bx = blinky_pos.0 as isize;
            let by = blinky_pos.1 as isize;
            let ax = ahead.0 as isize;
            let ay = ahead.1 as isize;
            let tx = ax + (ax - bx);
            let ty = ay + (ay - by);
            (tx.clamp(0, 30) as usize, ty.clamp(0, 30) as usize)
        }
        GhostPersonality::Clyde => {
            let dx = (pacman.pos.0 as isize - ghost.pos.0 as isize).abs();
            let dy = (pacman.pos.1 as isize - ghost.pos.1 as isize).abs();
            let dist = dx + dy;
            if dist > 8 {
                pacman.pos
            } else {
                SCATTER_TARGET
            }
        }
    }
}

fn dist_sq(a: (usize, usize), b: (usize, usize)) -> i64 {
    let dx = a.0 as i64 - b.0 as i64;
    let dy = a.1 as i64 - b.1 as i64;
    dx * dx + dy * dy
}

fn ghost_direction(
    pos: (usize, usize),
    current_dir: Direction,
    target: (usize, usize),
    maze: &MazeGrid,
    can_reverse: bool,
) -> Direction {
    let dirs = [
        Direction::Up,
        Direction::Down,
        Direction::Left,
        Direction::Right,
    ];
    let reverse = current_dir.opposite();

    let mut best_dir = current_dir;
    let mut best_dist = i64::MAX;

    for &dir in &dirs {
        if dir == reverse && !can_reverse {
            continue;
        }
        let (dx, dy) = dir.delta();
        let nx = pos.0 as isize + dx;
        let ny = pos.1 as isize + dy;
        if nx < 0 || ny < 0 || nx >= 31 || ny >= 31 {
            continue;
        }
        let np = (nx as usize, ny as usize);
        if is_wall(maze, np) {
            continue;
        }
        let d = dist_sq(np, target);
        if d < best_dist {
            best_dist = d;
            best_dir = dir;
        }
    }

    if best_dist == i64::MAX {
        // No valid direction, try any non-wall
        for &dir in &dirs {
            let (dx, dy) = dir.delta();
            let nx = pos.0 as isize + dx;
            let ny = pos.1 as isize + dy;
            if nx >= 0 && ny >= 0 && nx < 31 && ny < 31 {
                let np = (nx as usize, ny as usize);
                if !is_wall(maze, np) {
                    return dir;
                }
            }
        }
        return Direction::None;
    }

    best_dir
}

fn random_direction(pos: (usize, usize), maze: &MazeGrid, rng: &mut impl Rng) -> Direction {
    let dirs = [
        Direction::Up,
        Direction::Down,
        Direction::Left,
        Direction::Right,
    ];
    let valid: Vec<Direction> = dirs
        .iter()
        .filter(|&&dir| {
            let (dx, dy) = dir.delta();
            let nx = pos.0 as isize + dx;
            let ny = pos.1 as isize + dy;
            nx >= 0 && ny >= 0 && nx < 31 && ny < 31 && !is_wall(maze, (nx as usize, ny as usize))
        })
        .copied()
        .collect();

    if valid.is_empty() {
        return Direction::None;
    }

    let idx = rng.gen_range(0..valid.len());
    valid[idx]
}

pub fn move_ghosts(
    ghosts: &mut [Ghost],
    maze: &MazeGrid,
    pacman: &PacMan,
    blinky_pos: (usize, usize),
    lair_pos: (usize, usize),
) -> Vec<GhostEvent> {
    let mut events = Vec::new();
    let mut rng = rand::thread_rng();

    for (i, ghost) in ghosts.iter_mut().enumerate() {
        let target = ghost_target(ghost, pacman, blinky_pos);

        match ghost.mode {
            GhostMode::Chase | GhostMode::Scatter => {
                let dir = ghost_direction(ghost.pos, ghost.dir, target, maze, false);
                let (dx, dy) = dir.delta();
                let nx = ghost.pos.0 as isize + dx;
                let ny = ghost.pos.1 as isize + dy;
                if nx >= 0 && ny >= 0 && nx < 31 && ny < 31 {
                    let np = (nx as usize, ny as usize);
                    if !is_wall(maze, np) {
                        ghost.pos = np;
                        ghost.dir = dir;
                        events.push(GhostEvent::Moved(i));
                    }
                }
            }
            GhostMode::Frightened(remaining) => {
                // Half speed: move every other tick
                if ghost.tick_counter % 2 == 0 {
                    let dir = random_direction(ghost.pos, maze, &mut rng);
                    let (dx, dy) = dir.delta();
                    let nx = ghost.pos.0 as isize + dx;
                    let ny = ghost.pos.1 as isize + dy;
                    if nx >= 0 && ny >= 0 && nx < 31 && ny < 31 {
                        let np = (nx as usize, ny as usize);
                        if !is_wall(maze, np) {
                            ghost.pos = np;
                            ghost.dir = dir;
                            events.push(GhostEvent::Moved(i));
                        }
                    }
                }
                // Decrement fright timer
                ghost.tick_counter = ghost.tick_counter.wrapping_add(1);
                if remaining > 0 {
                    ghost.mode = GhostMode::Frightened(remaining - 1);
                }
            }
            GhostMode::Eaten => {
                // Move toward lair at fast speed
                let dir = ghost_direction(ghost.pos, ghost.dir, lair_pos, maze, true);
                let (dx, dy) = dir.delta();
                let nx = ghost.pos.0 as isize + dx;
                let ny = ghost.pos.1 as isize + dy;
                if nx >= 0 && ny >= 0 && nx < 31 && ny < 31 {
                    let np = (nx as usize, ny as usize);
                    if !is_wall(maze, np) {
                        ghost.pos = np;
                        ghost.dir = dir;
                        if ghost.pos == lair_pos {
                            ghost.mode = GhostMode::Chase;
                            events.push(GhostEvent::ReachedLair(i));
                        } else {
                            events.push(GhostEvent::Moved(i));
                        }
                    }
                }
            }
        }
    }

    events
}

pub fn enter_frightened_mode(ghosts: &mut [Ghost], duration: u32) {
    for ghost in ghosts.iter_mut() {
        if matches!(ghost.mode, GhostMode::Chase | GhostMode::Scatter) {
            ghost.dir = ghost.dir.opposite();
            ghost.mode = GhostMode::Frightened(duration);
            ghost.tick_counter = 0;
        }
    }
}

pub fn exit_frightened_mode(ghosts: &mut [Ghost]) {
    for ghost in ghosts.iter_mut() {
        if matches!(ghost.mode, GhostMode::Frightened(_)) {
            ghost.mode = GhostMode::Chase;
        }
    }
}

#[allow(dead_code)]
pub fn is_ghost_in_lair(ghost: &Ghost, lair_pos: (usize, usize)) -> bool {
    ghost.pos == lair_pos
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::{Ghost, GhostMode, GhostPersonality, PacMan};
    use crate::maze::Tile;

    #[allow(clippy::needless_range_loop)]
    fn test_maze() -> MazeGrid {
        let mut tiles = [[Tile::Wall; 31]; 31];
        for y in 1..30 {
            for x in 1..30 {
                tiles[y][x] = Tile::Dot;
            }
        }
        MazeGrid::new(tiles)
    }

    #[test]
    fn test_blinky_targets_pacman() {
        let ghost = Ghost::new((10, 10), GhostPersonality::Blinky);
        let pacman = PacMan::new((5, 5));
        let target = ghost_target(&ghost, &pacman, (10, 10));
        assert_eq!(target, (5, 5));
    }

    #[test]
    fn test_pinky_targets_ahead() {
        let ghost = Ghost::new((10, 10), GhostPersonality::Pinky);
        let mut pacman = PacMan::new((10, 10));
        pacman.dir = Direction::Right;
        let target = ghost_target(&ghost, &pacman, (10, 10));
        assert_eq!(target, (14, 10)); // 4 ahead
    }

    #[test]
    fn test_inky_targets_double_vector() {
        let mut pacman = PacMan::new((10, 10));
        pacman.dir = Direction::Right;
        let blinky_pos = (12, 10);
        let ghost = Ghost::new((10, 10), GhostPersonality::Inky);
        let target = ghost_target(&ghost, &pacman, blinky_pos);
        // Ahead = (12, 10), vector Blinky→Ahead = (0, 0), double = (0, 0), target = (12, 10)
        assert_eq!(target, (12, 10));
    }

    #[test]
    fn test_clyde_chase_far() {
        let ghost = Ghost::new((1, 1), GhostPersonality::Clyde);
        let pacman = PacMan::new((20, 20));
        let target = ghost_target(&ghost, &pacman, (10, 10));
        // Distance > 8, chase Pac-Man
        assert_eq!(target, (20, 20));
    }

    #[test]
    fn test_clyde_scatter_close() {
        let ghost = Ghost::new((5, 5), GhostPersonality::Clyde);
        let pacman = PacMan::new((6, 6));
        let target = ghost_target(&ghost, &pacman, (5, 5));
        // Distance <= 8, scatter to bottom-left
        assert_eq!(target, (1, 29));
    }

    #[test]
    fn test_enter_frightened_mode_reverses_direction() {
        let mut ghosts = vec![Ghost::new((10, 10), GhostPersonality::Blinky)];
        ghosts[0].dir = Direction::Right;
        enter_frightened_mode(&mut ghosts, 100);
        assert_eq!(ghosts[0].dir, Direction::Left);
        assert!(matches!(ghosts[0].mode, GhostMode::Frightened(100)));
    }

    #[test]
    fn test_ghost_movement_toward_target() {
        let maze = test_maze();
        let mut ghosts = vec![Ghost {
            pos: (10, 10),
            dir: Direction::Up,
            personality: GhostPersonality::Blinky,
            mode: GhostMode::Chase,
            spawn: (10, 10),
            tick_counter: 0,
        }];
        let pacman = PacMan::new((20, 20));
        let events = move_ghosts(&mut ghosts, &maze, &pacman, (10, 10), (15, 15));

        // Blinky targets (20,20), should move toward it
        assert!(!events.is_empty());
        // Ghost should have moved
        assert_ne!(ghosts[0].pos, (10, 10));
    }

    #[test]
    #[allow(clippy::needless_range_loop)]
    fn test_ghost_blocked_by_wall() {
        let mut tiles = [[Tile::Wall; 31]; 31];
        for y in 1..30 {
            for x in 1..30 {
                tiles[y][x] = Tile::Dot;
            }
        }
        // Put ghost in a corner
        tiles[9][10] = Tile::Wall;
        tiles[11][10] = Tile::Wall;
        tiles[10][9] = Tile::Wall;
        tiles[10][11] = Tile::Wall;
        let maze = MazeGrid::new(tiles);

        let mut ghosts = vec![Ghost {
            pos: (10, 10),
            dir: Direction::Up,
            personality: GhostPersonality::Blinky,
            mode: GhostMode::Chase,
            spawn: (10, 10),
            tick_counter: 0,
        }];
        let pacman = PacMan::new((20, 20));
        let _events = move_ghosts(&mut ghosts, &maze, &pacman, (10, 10), (15, 15));
        // Ghost trapped in 1x1 cell
        assert_eq!(ghosts[0].pos, (10, 10));
    }

    #[test]
    fn test_frightened_ghost_half_speed() {
        let maze = test_maze();
        let mut ghosts = vec![Ghost {
            pos: (10, 10),
            dir: Direction::Up,
            personality: GhostPersonality::Blinky,
            mode: GhostMode::Frightened(100),
            spawn: (10, 10),
            tick_counter: 0,
        }];
        let pacman = PacMan::new((20, 20));

        // First tick (counter 0): should move (0 % 2 == 0)
        let _events = move_ghosts(&mut ghosts, &maze, &pacman, (10, 10), (15, 15));
        assert_ne!(ghosts[0].pos, (10, 10), "first tick should move");

        // Second tick (counter 1): should NOT move (1 % 2 != 0)
        let pos_before = ghosts[0].pos;
        let _events2 = move_ghosts(&mut ghosts, &maze, &pacman, (10, 10), (15, 15));
        assert_eq!(ghosts[0].pos, pos_before, "second tick should not move");
    }

    #[test]
    fn test_eaten_ghost_moves_to_lair() {
        let maze = test_maze();
        let mut ghosts = vec![Ghost {
            pos: (20, 20),
            dir: Direction::Up,
            personality: GhostPersonality::Blinky,
            mode: GhostMode::Eaten,
            spawn: (15, 15),
            tick_counter: 0,
        }];
        let pacman = PacMan::new((5, 5));
        let lair = (15, 15);
        move_ghosts(&mut ghosts, &maze, &pacman, (20, 20), lair);
        // Should move toward lair
        let dist_after = (ghosts[0].pos.0 as i64 - lair.0 as i64).abs()
            + (ghosts[0].pos.1 as i64 - lair.1 as i64).abs();
        let dist_before = (20i64 - lair.0 as i64).abs() + (20i64 - lair.1 as i64).abs();
        assert!(dist_after < dist_before);
    }

    #[test]
    fn test_exit_frightened_mode() {
        let mut ghosts = vec![Ghost {
            pos: (10, 10),
            dir: Direction::Up,
            personality: GhostPersonality::Blinky,
            mode: GhostMode::Frightened(50),
            spawn: (10, 10),
            tick_counter: 0,
        }];
        exit_frightened_mode(&mut ghosts);
        assert_eq!(ghosts[0].mode, GhostMode::Chase);
    }
}
