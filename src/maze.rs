#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    Wall,
    Dot,
    PowerPellet,
    Portal(usize),
    Empty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    None,
}

impl Direction {
    pub fn opposite(&self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
            Direction::None => Direction::None,
        }
    }

    pub fn delta(&self) -> (isize, isize) {
        match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
            Direction::None => (0, 0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileEffect {
    Dot(u32),
    PowerPellet(u32),
    Portal(usize),
    Fruit(u32),
    Empty,
    Wall,
}

#[derive(Debug, Clone)]
pub struct MazeGrid {
    pub tiles: [[Tile; 31]; 31],
    pub consumed: [[bool; 31]; 31],
    pub fruit_pos: Option<(usize, usize)>,
    pub fruit_value: u32,
}

impl MazeGrid {
    pub fn new(tiles: [[Tile; 31]; 31]) -> Self {
        MazeGrid {
            tiles,
            consumed: [[false; 31]; 31],
            fruit_pos: None,
            fruit_value: 0,
        }
    }
}

pub fn is_wall(grid: &MazeGrid, pos: (usize, usize)) -> bool {
    matches!(grid.tiles[pos.1][pos.0], Tile::Wall)
}

pub fn is_portal(grid: &MazeGrid, pos: (usize, usize)) -> Option<usize> {
    match grid.tiles[pos.1][pos.0] {
        Tile::Portal(idx) => Some(idx),
        _ => None,
    }
}

pub fn dots_remaining(grid: &MazeGrid) -> u32 {
    let mut count: u32 = 0;
    for y in 0..31 {
        for x in 0..31 {
            if !grid.consumed[y][x] {
                match grid.tiles[y][x] {
                    Tile::Dot | Tile::PowerPellet => count += 1,
                    _ => {}
                }
            }
        }
    }
    count
}

pub fn consume_dot(grid: &mut MazeGrid, pos: (usize, usize)) -> TileEffect {
    if grid.consumed[pos.1][pos.0] {
        return TileEffect::Empty;
    }
    match grid.tiles[pos.1][pos.0] {
        Tile::Dot => {
            grid.consumed[pos.1][pos.0] = true;
            TileEffect::Dot(10)
        }
        Tile::PowerPellet => {
            grid.consumed[pos.1][pos.0] = true;
            TileEffect::PowerPellet(50)
        }
        Tile::Portal(idx) => TileEffect::Portal(idx),
        Tile::Empty => TileEffect::Empty,
        Tile::Wall => TileEffect::Wall,
    }
}

pub fn consume_fruit(grid: &mut MazeGrid) -> Option<u32> {
    if grid.fruit_pos.is_some() {
        let value = grid.fruit_value;
        grid.fruit_pos = None;
        grid.fruit_value = 0;
        Some(value)
    } else {
        None
    }
}

pub fn teleport_portal(grid: &MazeGrid, pos: (usize, usize)) -> Option<(usize, usize)> {
    let pair_idx = is_portal(grid, pos)?;
    for y in 0..31 {
        for x in 0..31 {
            if (x, y) != pos {
                if let Tile::Portal(idx) = grid.tiles[y][x] {
                    if idx == pair_idx {
                        return Some((x, y));
                    }
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_grid() -> MazeGrid {
        let mut tiles = [[Tile::Wall; 31]; 31];
        for y in 1..30 {
            for x in 1..30 {
                tiles[y][x] = Tile::Dot;
            }
        }
        tiles[2][2] = Tile::PowerPellet;
        tiles[5][5] = Tile::Portal(0);
        tiles[10][10] = Tile::Portal(0);
        tiles[15][15] = Tile::Portal(1);
        tiles[20][20] = Tile::Portal(1);
        tiles[3][3] = Tile::Empty;
        MazeGrid::new(tiles)
    }

    #[test]
    fn test_is_wall() {
        let grid = make_grid();
        assert!(is_wall(&grid, (0, 0)));
        assert!(!is_wall(&grid, (2, 2)));
    }

    #[test]
    fn test_is_portal() {
        let grid = make_grid();
        assert_eq!(is_portal(&grid, (5, 5)), Some(0));
        assert_eq!(is_portal(&grid, (10, 10)), Some(0));
        assert_eq!(is_portal(&grid, (15, 15)), Some(1));
        assert_eq!(is_portal(&grid, (2, 2)), None);
    }

    #[test]
    fn test_consume_dot() {
        let mut grid = make_grid();
        // (2,2) is a PowerPellet
        assert_eq!(consume_dot(&mut grid, (2, 2)), TileEffect::PowerPellet(50));
        // second consume returns Empty since already consumed
        assert_eq!(consume_dot(&mut grid, (2, 2)), TileEffect::Empty);
        // (3,3) is Empty
        assert_eq!(consume_dot(&mut grid, (3, 3)), TileEffect::Empty);
        // (5,5) is a Portal
        assert_eq!(consume_dot(&mut grid, (5, 5)), TileEffect::Portal(0));
        // 841 inner cells - 4 portals - 1 empty - 1 consumed pellet = 835 dots remaining
        assert_eq!(dots_remaining(&grid), 29 * 29 - 6);
    }

    #[test]
    fn test_dots_remaining() {
        let mut grid = make_grid();
        let initial = dots_remaining(&grid);
        consume_dot(&mut grid, (1, 1));
        assert_eq!(dots_remaining(&grid), initial - 1);
    }

    #[test]
    fn test_teleport_portal() {
        let grid = make_grid();
        assert_eq!(teleport_portal(&grid, (5, 5)), Some((10, 10)));
        assert_eq!(teleport_portal(&grid, (10, 10)), Some((5, 5)));
        assert_eq!(teleport_portal(&grid, (15, 15)), Some((20, 20)));
        assert_eq!(teleport_portal(&grid, (2, 2)), None);
    }

    #[test]
    fn test_direction_opposite() {
        assert_eq!(Direction::Up.opposite(), Direction::Down);
        assert_eq!(Direction::Left.opposite(), Direction::Right);
        assert_eq!(Direction::None.opposite(), Direction::None);
    }

    #[test]
    fn test_direction_delta() {
        assert_eq!(Direction::Up.delta(), (0, -1));
        assert_eq!(Direction::Down.delta(), (0, 1));
        assert_eq!(Direction::Left.delta(), (-1, 0));
        assert_eq!(Direction::Right.delta(), (1, 0));
    }

    #[test]
    fn test_consume_fruit() {
        let mut grid = make_grid();
        assert_eq!(consume_fruit(&mut grid), None);
        grid.fruit_pos = Some((3, 3));
        grid.fruit_value = 100;
        assert_eq!(consume_fruit(&mut grid), Some(100));
        assert_eq!(grid.fruit_pos, None);
    }
}
