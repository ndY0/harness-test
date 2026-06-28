use crate::game::{Direction, GhostMode, Position};
use crate::game::maze::Maze;

const MAZE_MAX_ROW: usize = 20;
const MAZE_MAX_COL: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Personality {
    Blinky,
    Pinky,
    Inky,
    Clyde,
}

pub struct Ghost {
    pub pos: Position,
    pub mode: GhostMode,
    pub personality: Personality,
    pub home_pos: Position,
    pub scatter_target: Position,
    pub prev_pos: Option<Position>,
    pub is_in_house: bool,
}

impl Ghost {
    pub fn new(personality: Personality, start: Position, scatter: Position) -> Self {
        Ghost {
            pos: start,
            mode: GhostMode::Scatter,
            personality,
            home_pos: start,
            scatter_target: scatter,
            prev_pos: None,
            // Blinky starts outside; all others start in the ghost house.
            is_in_house: personality != Personality::Blinky,
        }
    }

    /// Advance ghost one step (only acts on even ticks).
    pub fn step(
        &mut self,
        maze: &Maze,
        player_pos: Position,
        player_facing: Option<Direction>,
        _blinky_pos: Position,
        dots_eaten: u32,
        tick: u64,
    ) {
        if tick % 2 != 0 {
            return;
        }

        // House exit: stay put until threshold; then release.
        if self.is_in_house {
            if dots_eaten < self.dot_threshold() {
                return;
            }
            self.is_in_house = false;
            // Fall through to normal movement — greedy step exits the house.
        }

        // Frightened: random adjacent open tile (ghost house not excluded per spec).
        if self.mode == GhostMode::Frightened {
            let candidates = self.neighbors_filtered(maze, true);
            if candidates.is_empty() {
                return;
            }
            let idx = pseudo_rand(tick, self.pos, candidates.len());
            self.prev_pos = Some(self.pos);
            self.pos = candidates[idx];
            return;
        }

        // Inky in Chase: random adjacent open tile (ghost house excluded).
        if self.mode == GhostMode::Chase && self.personality == Personality::Inky {
            let candidates = self.neighbors_filtered(maze, false);
            if candidates.is_empty() {
                return;
            }
            let idx = pseudo_rand(tick, self.pos, candidates.len());
            self.prev_pos = Some(self.pos);
            self.pos = candidates[idx];
            return;
        }

        // Greedy movement toward target (Scatter or Chase for Blinky/Pinky/Clyde).
        let target = match self.mode {
            GhostMode::Scatter => self.scatter_target,
            GhostMode::Chase => self.chase_target(player_pos, player_facing),
            GhostMode::Frightened => unreachable!("handled above"),
        };

        let candidates = self.neighbors_filtered(maze, false);
        if candidates.is_empty() {
            return;
        }
        let next = candidates
            .into_iter()
            .min_by_key(|&p| manhattan(p, target))
            .unwrap();
        self.prev_pos = Some(self.pos);
        self.pos = next;
    }

    /// Reset to home position, clear frightened mode, set is_in_house = true.
    pub fn send_home(&mut self) {
        self.pos = self.home_pos;
        self.mode = GhostMode::Scatter;
        self.is_in_house = true;
        self.prev_pos = None;
    }

    /// Set mode; when transitioning FROM Frightened, prev_pos is cleared.
    pub fn set_mode(&mut self, mode: GhostMode) {
        if self.mode == GhostMode::Frightened && mode != GhostMode::Frightened {
            self.prev_pos = None;
        }
        self.mode = mode;
    }

    /// True when in frightened mode AND frightened_ticks_remaining <= 30.
    pub fn is_frightened_blinking(&self, frightened_ticks_remaining: u32) -> bool {
        self.mode == GhostMode::Frightened && frightened_ticks_remaining <= 30
    }

    // ---- private helpers ----

    fn dot_threshold(&self) -> u32 {
        match self.personality {
            Personality::Blinky => 0,
            Personality::Pinky => 0,
            Personality::Inky => 30,
            Personality::Clyde => 60,
        }
    }

    fn chase_target(&self, player_pos: Position, player_facing: Option<Direction>) -> Position {
        match self.personality {
            Personality::Blinky => player_pos,
            Personality::Pinky => {
                let facing = player_facing.unwrap_or(Direction::Up);
                let (dr, dc): (i64, i64) = match facing {
                    Direction::Up => (-1, 0),
                    Direction::Down => (1, 0),
                    Direction::Left => (0, -1),
                    Direction::Right => (0, 1),
                };
                let row = (player_pos.row as i64 + 4 * dr)
                    .clamp(0, MAZE_MAX_ROW as i64) as usize;
                let col = (player_pos.col as i64 + 4 * dc)
                    .clamp(0, MAZE_MAX_COL as i64) as usize;
                Position { row, col }
            }
            Personality::Inky => unreachable!("Inky uses random movement in step()"),
            Personality::Clyde => {
                if chebyshev(self.pos, player_pos) > 8 {
                    player_pos
                } else {
                    self.scatter_target
                }
            }
        }
    }

    /// Returns adjacent open tiles, preferring to exclude prev_pos.
    /// `allow_ghost_house`: Frightened mode allows entering ghost house tiles.
    fn neighbors_filtered(&self, maze: &Maze, allow_ghost_house: bool) -> Vec<Position> {
        let all: Vec<Position> = self
            .adjacent(maze)
            .into_iter()
            .filter(|&p| !maze.is_wall(p))
            .filter(|&p| allow_ghost_house || !maze.is_ghost_house(p))
            .collect();

        let without_prev: Vec<Position> = all
            .iter()
            .copied()
            .filter(|&p| Some(p) != self.prev_pos)
            .collect();

        // Fall back to all candidates if prev_pos filter leaves nothing (dead end).
        if without_prev.is_empty() {
            all
        } else {
            without_prev
        }
    }

    /// The four cardinal neighbors with tunnel-row wrapping at the maze edges.
    fn adjacent(&self, maze: &Maze) -> Vec<Position> {
        let row = self.pos.row;
        let col = self.pos.col;
        let tunnel = maze.tunnel_row(row);
        let mut out = Vec::with_capacity(4);

        // Up
        if row > 0 {
            out.push(Position { row: row - 1, col });
        }
        // Down
        if row < MAZE_MAX_ROW {
            out.push(Position { row: row + 1, col });
        }
        // Left — wrap at col 0 in a tunnel row
        if tunnel && col == 0 {
            out.push(Position { row, col: MAZE_MAX_COL });
        } else if col > 0 {
            out.push(Position { row, col: col - 1 });
        }
        // Right — wrap at col 20 in a tunnel row
        if tunnel && col == MAZE_MAX_COL {
            out.push(Position { row, col: 0 });
        } else if col < MAZE_MAX_COL {
            out.push(Position { row, col: col + 1 });
        }

        out
    }
}

fn manhattan(a: Position, b: Position) -> usize {
    a.row.abs_diff(b.row) + a.col.abs_diff(b.col)
}

fn chebyshev(a: Position, b: Position) -> usize {
    a.row.abs_diff(b.row).max(a.col.abs_diff(b.col))
}

fn pseudo_rand(tick: u64, pos: Position, len: usize) -> usize {
    ((tick as usize).wrapping_add(pos.row).wrapping_add(pos.col)) % len
}
