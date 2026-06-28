use crate::game::{Cell, Position};
use crate::game::maze::Maze;

pub struct PelletMap {
    data: Vec<bool>,
    cols: usize,
    cell_types: Vec<Cell>,
}

impl PelletMap {
    pub fn new(maze: &Maze) -> Self {
        let cols = maze.cols();
        let rows = maze.rows_count();
        let size = rows * cols;
        let mut data = vec![false; size];
        let mut cell_types = vec![Cell::Empty; size];
        for (r, row) in maze.cells.iter().enumerate() {
            for (c, &cell) in row.iter().enumerate() {
                let idx = r * cols + c;
                if matches!(cell, Cell::Dot | Cell::PowerPellet) {
                    data[idx] = true;
                    cell_types[idx] = cell;
                }
            }
        }
        Self { data, cols, cell_types }
    }

    pub fn eat(&mut self, pos: Position) -> Cell {
        let idx = pos.row * self.cols + pos.col;
        if self.data[idx] {
            self.data[idx] = false;
            self.cell_types[idx]
        } else {
            Cell::Empty
        }
    }

    pub fn all_eaten(&self) -> bool {
        self.data.iter().all(|&active| !active)
    }

    /// Returns true when the pellet at pos is still active (uneaten).
    pub fn is_active(&self, pos: Position) -> bool {
        let idx = pos.row * self.cols + pos.col;
        idx < self.data.len() && self.data[idx]
    }

    pub fn active_positions(&self) -> impl Iterator<Item = Position> + '_ {
        self.data.iter().enumerate().filter_map(|(idx, &active)| {
            if active {
                Some(Position {
                    row: idx / self.cols,
                    col: idx % self.cols,
                })
            } else {
                None
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::maze::Maze;

    fn simple_maze() -> Maze {
        let rows: [&str; 21] = [
            "#####################",
            "#o..........#......o#",
            "#.##.#####.#.#####.##",
            "#.#..........#......#",
            "#.#.##.###.###.###..#",
            "#....#...........#..#",
            "#.##.#.#######.#.##.#",
            "#....#....P....#....#",
            "#.##.#.#######.#.##.#",
            "#....#...GG....#....#",
            "#.##.#..GG...#.#.##.#",
            "#....#.........#....#",
            "#.##.#.#######.#.##.#",
            "#....#.........#....#",
            "#.##.#.#######.#.##.#",
            "#o..............o...#",
            "#.##.#####.#####.##.#",
            "#..#..........#..#..#",
            "#.#.##.###.###.##.#.#",
            "#...........#.......#",
            "#####################",
        ];
        Maze::parse(&rows)
    }

    #[test]
    fn new_initialises_pellets_from_maze() {
        let maze = simple_maze();
        let pm = PelletMap::new(&maze);
        assert!(!pm.all_eaten());
        let count = pm.active_positions().count();
        assert!(count > 0);
    }

    #[test]
    fn eat_dot_returns_dot_and_marks_eaten() {
        let maze = simple_maze();
        let mut pm = PelletMap::new(&maze);
        // row 1, col 2 is '.' in stage 1
        let pos = Position { row: 1, col: 2 };
        let result = pm.eat(pos);
        assert_eq!(result, Cell::Dot);
        assert_eq!(pm.eat(pos), Cell::Empty);
    }

    #[test]
    fn eat_power_pellet_returns_power_pellet() {
        let maze = simple_maze();
        let mut pm = PelletMap::new(&maze);
        // row 1, col 1 is 'o' (PowerPellet) in stage 1
        let pos = Position { row: 1, col: 1 };
        let result = pm.eat(pos);
        assert_eq!(result, Cell::PowerPellet);
    }

    #[test]
    fn eat_non_pellet_returns_empty() {
        let maze = simple_maze();
        let mut pm = PelletMap::new(&maze);
        // row 0, col 0 is '#' (Wall) — never a pellet
        let pos = Position { row: 0, col: 0 };
        assert_eq!(pm.eat(pos), Cell::Empty);
    }

    #[test]
    fn all_eaten_true_when_all_pellets_consumed() {
        let maze = simple_maze();
        let mut pm = PelletMap::new(&maze);
        let positions: Vec<Position> = pm.active_positions().collect();
        for pos in positions {
            pm.eat(pos);
        }
        assert!(pm.all_eaten());
    }
}
