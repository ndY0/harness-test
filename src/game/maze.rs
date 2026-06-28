use crate::game::{Cell, Position};

pub struct Maze {
    pub cells: Vec<Vec<Cell>>,
}

impl Maze {
    pub fn parse(rows: &[&str; 21]) -> Self {
        let cells = rows
            .iter()
            .map(|row| {
                let mut row_cells: Vec<Cell> = row
                    .chars()
                    .take(21)
                    .map(|c| match c {
                        '#' => Cell::Wall,
                        '.' => Cell::Dot,
                        'o' => Cell::PowerPellet,
                        'P' => Cell::PlayerStart,
                        'G' => Cell::GhostHouse,
                        _ => Cell::Empty,
                    })
                    .collect();
                while row_cells.len() < 21 {
                    row_cells.push(Cell::Empty);
                }
                row_cells
            })
            .collect();
        Maze { cells }
    }

    pub fn cols(&self) -> usize {
        21
    }

    pub fn rows_count(&self) -> usize {
        21
    }

    pub fn is_wall(&self, pos: Position) -> bool {
        self.cells[pos.row][pos.col] == Cell::Wall
    }

    pub fn is_ghost_house(&self, pos: Position) -> bool {
        self.cells[pos.row][pos.col] == Cell::GhostHouse
    }

    /// Returns true when col 0 of the given row is non-wall (open tunnel side).
    pub fn tunnel_row(&self, row: usize) -> bool {
        self.cells[row][0] != Cell::Wall
    }

    pub fn player_start(&self) -> Position {
        for (r, row) in self.cells.iter().enumerate() {
            for (c, &cell) in row.iter().enumerate() {
                if cell == Cell::PlayerStart {
                    return Position { row: r, col: c };
                }
            }
        }
        panic!("no PlayerStart tile in maze")
    }

    pub fn ghost_house_center(&self) -> Position {
        for (r, row) in self.cells.iter().enumerate() {
            for (c, &cell) in row.iter().enumerate() {
                if cell == Cell::GhostHouse {
                    return Position { row: r, col: c };
                }
            }
        }
        panic!("no GhostHouse tile in maze")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::stages::STAGES;

    #[test]
    fn stages_has_ten_entries() {
        assert_eq!(STAGES.len(), 10);
    }

    #[test]
    fn each_stage_has_21_rows() {
        for stage in &STAGES {
            assert_eq!(stage.len(), 21);
        }
    }

    #[test]
    fn parse_stage_0_produces_21x21_maze() {
        let maze = Maze::parse(STAGES[0]);
        assert_eq!(maze.rows_count(), 21);
        assert_eq!(maze.cols(), 21);
        assert_eq!(maze.cells.len(), 21);
        for row in &maze.cells {
            assert_eq!(row.len(), 21);
        }
    }

    #[test]
    fn is_wall_true_for_corner() {
        let maze = Maze::parse(STAGES[0]);
        assert!(maze.is_wall(Position { row: 0, col: 0 }));
    }

    #[test]
    fn tunnel_row_for_stages_3_and_7() {
        let maze3 = Maze::parse(STAGES[2]);
        let maze7 = Maze::parse(STAGES[6]);
        let maze1 = Maze::parse(STAGES[0]);
        assert!(maze3.tunnel_row(10), "stage 3 row 10 should be a tunnel");
        assert!(maze7.tunnel_row(10), "stage 7 row 10 should be a tunnel");
        assert!(!maze1.tunnel_row(10), "stage 1 row 10 is not a tunnel");
    }

    #[test]
    fn player_start_stage_0() {
        let maze = Maze::parse(STAGES[0]);
        let pos = maze.player_start();
        assert_eq!(maze.cells[pos.row][pos.col], Cell::PlayerStart);
    }

    #[test]
    fn ghost_house_center_is_ghost_house() {
        let maze = Maze::parse(STAGES[0]);
        let pos = maze.ghost_house_center();
        assert_eq!(maze.cells[pos.row][pos.col], Cell::GhostHouse);
    }
}
