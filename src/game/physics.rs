use crate::game::{Direction, GhostMode, Position};
use crate::game::ghost::Ghost;
use crate::game::maze::Maze;

/// Attempt to move entity at `pos` one tile in `dir` within `maze`.
/// Returns Some(new_pos) if the tile is open, None if blocked by a wall.
/// Applies tunnel wrapping when pos is at col 0/20 and maze.tunnel_row(row) is true.
pub fn apply_move(pos: Position, dir: Direction, maze: &Maze) -> Option<Position> {
    let (delta_row, delta_col): (isize, isize) = match dir {
        Direction::Up => (-1, 0),
        Direction::Down => (1, 0),
        Direction::Left => (0, -1),
        Direction::Right => (0, 1),
    };

    let mut new_row = pos.row as isize + delta_row;
    let mut new_col = pos.col as isize + delta_col;

    // Tunnel wrapping: only on designated tunnel rows, only at column boundaries.
    if maze.tunnel_row(pos.row) {
        if dir == Direction::Left && pos.col == 0 {
            new_col = 20;
        } else if dir == Direction::Right && pos.col == 20 {
            new_col = 0;
        }
    }

    // Bounds check (maze is 21×21, cols/rows 0..=20).
    if new_row < 0 || new_row >= 21 || new_col < 0 || new_col >= 21 {
        return None;
    }

    let new_pos = Position { row: new_row as usize, col: new_col as usize };
    if maze.is_wall(new_pos) {
        return None;
    }

    Some(new_pos)
}

/// Returns the index of a colliding ghost, or None.
/// Non-frightened ghosts take priority over frightened ones.
/// Ghosts with is_in_house == true are excluded.
pub fn check_player_ghost_collision(
    player_pos: Position,
    ghosts: &[Ghost],
) -> Option<usize> {
    // First pass: non-frightened ghosts (they cause a life-lost and take priority).
    for (i, ghost) in ghosts.iter().enumerate() {
        if ghost.is_in_house {
            continue;
        }
        if ghost.pos == player_pos && ghost.mode != GhostMode::Frightened {
            return Some(i);
        }
    }
    // Second pass: frightened ghosts (player eats them).
    for (i, ghost) in ghosts.iter().enumerate() {
        if ghost.is_in_house {
            continue;
        }
        if ghost.pos == player_pos && ghost.mode == GhostMode::Frightened {
            return Some(i);
        }
    }
    None
}
