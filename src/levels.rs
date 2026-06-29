#![allow(clippy::needless_range_loop)]

use crate::maze::Tile;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LevelConfig {
    pub width: usize,
    pub height: usize,
    pub tiles: [[Tile; 31]; 31],
    pub pacman_spawn: (usize, usize),
    pub ghost_spawns: [(usize, usize); 4],
    pub ghost_lair: (usize, usize),
    pub portal_pair_a: ((usize, usize), (usize, usize)),
    pub portal_pair_b: ((usize, usize), (usize, usize)),
    pub ghost_speed_factor: f32,
    pub fright_duration_ms: u32,
}

const W: Tile = Tile::Wall;
const D: Tile = Tile::Dot;
const P: Tile = Tile::PowerPellet;
const E: Tile = Tile::Empty;

fn setup_lair(tiles: &mut [[Tile; 31]; 31]) {
    for x in 13..18 { tiles[14][x] = W; }
    for x in 13..18 { tiles[16][x] = W; }
    for y in 14..17 { tiles[y][13] = W; }
    for y in 14..17 { tiles[y][17] = W; }
    for y in 14..17 {
        for x in 14..17 {
            tiles[y][x] = E;
        }
    }
}

fn make_level_1() -> LevelConfig {
    let mut tiles = [[W; 31]; 31];
    for y in 1..30 {
        for x in 1..30 {
            tiles[y][x] = D;
        }
    }
    // Inner walls
    for x in 5..26 { tiles[5][x] = W; }
    for x in 5..26 { tiles[25][x] = W; }
    for y in 5..26 { tiles[y][5] = W; }
    for y in 5..26 { tiles[y][25] = W; }
    // Gaps in inner walls
    tiles[5][15] = D;
    tiles[25][15] = D;
    tiles[15][5] = D;
    tiles[15][25] = D;
    // Power pellets in corners of inner box
    tiles[6][6] = P;
    tiles[6][24] = P;
    tiles[24][6] = P;
    tiles[24][24] = P;
    // Ghost lair
    setup_lair(&mut tiles);
    // Portal pairs
    tiles[1][1] = Tile::Portal(0);
    tiles[29][29] = Tile::Portal(0);
    tiles[1][29] = Tile::Portal(1);
    tiles[29][1] = Tile::Portal(1);

    LevelConfig {
        width: 31,
        height: 31,
        tiles,
        pacman_spawn: (15, 23),
        ghost_spawns: [(14, 15), (16, 15), (15, 14), (15, 16)],
        ghost_lair: (15, 15),
        portal_pair_a: ((1, 1), (29, 29)),
        portal_pair_b: ((1, 29), (29, 1)),
        ghost_speed_factor: 0.75,
        fright_duration_ms: 6000,
    }
}

fn make_level_2() -> LevelConfig {
    let mut tiles = [[W; 31]; 31];
    for y in 1..30 {
        for x in 1..30 {
            tiles[y][x] = D;
        }
    }
    // Horizontal walls
    for x in 3..28 { tiles[8][x] = W; }
    for x in 3..28 { tiles[22][x] = W; }
    tiles[8][15] = D;
    tiles[22][15] = D;
    // Vertical walls
    for y in 8..23 { tiles[y][8] = W; }
    for y in 8..23 { tiles[y][22] = W; }
    tiles[15][8] = D;
    tiles[15][22] = D;
    // Power pellets
    tiles[3][3] = P;
    tiles[3][27] = P;
    tiles[27][3] = P;
    tiles[27][27] = P;
    // Ghost lair
    setup_lair(&mut tiles);
    // Portals
    tiles[1][15] = Tile::Portal(0);
    tiles[29][15] = Tile::Portal(0);
    tiles[15][1] = Tile::Portal(1);
    tiles[15][29] = Tile::Portal(1);

    LevelConfig {
        width: 31,
        height: 31,
        tiles,
        pacman_spawn: (15, 23),
        ghost_spawns: [(14, 15), (16, 15), (15, 14), (15, 16)],
        ghost_lair: (15, 15),
        portal_pair_a: ((1, 15), (29, 15)),
        portal_pair_b: ((15, 1), (15, 29)),
        ghost_speed_factor: 0.80,
        fright_duration_ms: 5444,
    }
}

fn make_level_3() -> LevelConfig {
    let mut tiles = [[W; 31]; 31];
    for y in 1..30 {
        for x in 1..30 {
            tiles[y][x] = D;
        }
    }
    // T-shaped walls
    for x in 10..21 { tiles[10][x] = W; }
    for y in 10..21 { tiles[y][15] = W; }
    tiles[10][15] = D;
    tiles[20][15] = D;
    tiles[15][10] = D;
    // Additional walls
    for x in 4..12 { tiles[4][x] = W; }
    for x in 19..27 { tiles[26][x] = W; }
    tiles[4][8] = D;
    tiles[26][23] = D;
    // Power pellets
    tiles[2][2] = P;
    tiles[2][28] = P;
    tiles[28][2] = P;
    tiles[28][28] = P;
    tiles[10][10] = P;
    // Ghost lair
    setup_lair(&mut tiles);
    // Portals
    tiles[1][14] = Tile::Portal(0);
    tiles[29][16] = Tile::Portal(0);
    tiles[14][1] = Tile::Portal(1);
    tiles[16][29] = Tile::Portal(1);

    LevelConfig {
        width: 31,
        height: 31,
        tiles,
        pacman_spawn: (15, 23),
        ghost_spawns: [(14, 15), (16, 15), (15, 14), (15, 16)],
        ghost_lair: (15, 15),
        portal_pair_a: ((1, 14), (29, 16)),
        portal_pair_b: ((14, 1), (16, 29)),
        ghost_speed_factor: 0.85,
        fright_duration_ms: 4888,
    }
}

fn make_level_4() -> LevelConfig {
    let mut tiles = [[W; 31]; 31];
    for y in 1..30 {
        for x in 1..30 {
            tiles[y][x] = D;
        }
    }
    // Diamond-shaped walls
    for i in 0..8 {
        for x in (15 - i)..=(15 + i) {
            if 8 + i < 23 && (8..=22).contains(&x) {
                tiles[8 + i][x] = W;
                tiles[22 - i][x] = W;
            }
        }
    }
    tiles[15][15] = D;
    // Power pellets
    tiles[1][1] = P;
    tiles[1][29] = P;
    tiles[29][1] = P;
    tiles[29][29] = P;
    tiles[1][15] = P;
    tiles[29][15] = P;
    // Ghost lair
    setup_lair(&mut tiles);
    // Portals
    tiles[1][7] = Tile::Portal(0);
    tiles[29][23] = Tile::Portal(0);
    tiles[7][1] = Tile::Portal(1);
    tiles[23][29] = Tile::Portal(1);

    LevelConfig {
        width: 31,
        height: 31,
        tiles,
        pacman_spawn: (15, 23),
        ghost_spawns: [(14, 15), (16, 15), (15, 14), (15, 16)],
        ghost_lair: (15, 15),
        portal_pair_a: ((1, 7), (29, 23)),
        portal_pair_b: ((7, 1), (23, 29)),
        ghost_speed_factor: 0.90,
        fright_duration_ms: 4333,
    }
}

fn make_level_5() -> LevelConfig {
    let mut tiles = [[W; 31]; 31];
    for y in 1..30 {
        for x in 1..30 {
            tiles[y][x] = D;
        }
    }
    // Grid pattern wall segments
    for y in (3..28).step_by(6) {
        for x in 3..28 { tiles[y][x] = W; }
        tiles[y][15] = D;
    }
    for x in (3..28).step_by(6) {
        for y in 3..28 { tiles[y][x] = W; }
        tiles[15][x] = D;
    }
    // Power pellets
    tiles[2][2] = P;
    tiles[2][28] = P;
    tiles[28][2] = P;
    tiles[28][28] = P;
    // Ghost lair
    setup_lair(&mut tiles);
    // Portals
    tiles[1][1] = Tile::Portal(0);
    tiles[29][1] = Tile::Portal(0);
    tiles[1][29] = Tile::Portal(1);
    tiles[29][29] = Tile::Portal(1);

    LevelConfig {
        width: 31,
        height: 31,
        tiles,
        pacman_spawn: (15, 1),
        ghost_spawns: [(14, 15), (16, 15), (15, 14), (15, 16)],
        ghost_lair: (15, 15),
        portal_pair_a: ((1, 1), (29, 1)),
        portal_pair_b: ((1, 29), (29, 29)),
        ghost_speed_factor: 0.95,
        fright_duration_ms: 3777,
    }
}

fn make_level_6() -> LevelConfig {
    let mut tiles = [[W; 31]; 31];
    for y in 1..30 {
        for x in 1..30 {
            tiles[y][x] = D;
        }
    }
    // Spiral-inspired walls
    for i in 0..4 {
        let s = 2 + i * 4;
        let e = 28 - i * 4;
        for x in s..=e { tiles[s][x] = W; }
        for y in s..=e { tiles[y][e] = W; }
        for x in (s..=e).rev() { tiles[e][x] = W; }
        for y in (s..=e).rev() { tiles[y][s] = W; }
        // Gaps
        tiles[s][s + 4] = D;
        tiles[s + 4][e] = D;
        tiles[e][e - 4] = D;
        tiles[e - 4][s] = D;
    }
    // Power pellets
    tiles[3][3] = P;
    tiles[3][27] = P;
    tiles[27][3] = P;
    tiles[27][27] = P;
    // Ghost lair
    setup_lair(&mut tiles);
    // Portals
    tiles[1][1] = Tile::Portal(0);
    tiles[29][29] = Tile::Portal(0);
    tiles[1][10] = Tile::Portal(1);
    tiles[29][20] = Tile::Portal(1);

    LevelConfig {
        width: 31,
        height: 31,
        tiles,
        pacman_spawn: (15, 23),
        ghost_spawns: [(14, 15), (16, 15), (15, 14), (15, 16)],
        ghost_lair: (15, 15),
        portal_pair_a: ((1, 1), (29, 29)),
        portal_pair_b: ((1, 10), (29, 20)),
        ghost_speed_factor: 1.00,
        fright_duration_ms: 3222,
    }
}

fn make_level_7() -> LevelConfig {
    let mut tiles = [[W; 31]; 31];
    for y in 1..30 {
        for x in 1..30 {
            tiles[y][x] = D;
        }
    }
    // Cross-shaped open areas with scattered wall blocks
    for y in 1..30 {
        for x in 1..30 {
            if (x % 4 == 0 && y % 4 == 0) && (x > 3 && x < 27 && y > 3 && y < 27) {
                tiles[y][x] = W;
            }
        }
    }
    // Open corridors
    tiles[15][4] = D;
    tiles[15][26] = D;
    tiles[4][15] = D;
    tiles[26][15] = D;
    // Power pellets
    tiles[2][2] = P;
    tiles[2][28] = P;
    tiles[28][2] = P;
    tiles[28][28] = P;
    tiles[15][15] = P;
    // Ghost lair
    setup_lair(&mut tiles);
    // Power pellets (all outside lair)
    tiles[1][15] = Tile::Portal(0);
    tiles[29][15] = Tile::Portal(0);
    tiles[15][1] = Tile::Portal(1);
    tiles[15][29] = Tile::Portal(1);

    LevelConfig {
        width: 31,
        height: 31,
        tiles,
        pacman_spawn: (15, 23),
        ghost_spawns: [(14, 15), (16, 15), (15, 14), (15, 16)],
        ghost_lair: (15, 15),
        portal_pair_a: ((1, 15), (29, 15)),
        portal_pair_b: ((15, 1), (15, 29)),
        ghost_speed_factor: 1.05,
        fright_duration_ms: 2666,
    }
}

fn make_level_8() -> LevelConfig {
    let mut tiles = [[W; 31]; 31];
    for y in 1..30 {
        for x in 1..30 {
            tiles[y][x] = D;
        }
    }
    // Maze-like walls
    for x in 2..29 { tiles[2][x] = W; }
    for x in 2..29 { tiles[28][x] = W; }
    for y in 2..29 { tiles[y][2] = W; }
    for y in 2..29 { tiles[y][28] = W; }
    for x in 4..27 { tiles[10][x] = W; }
    for x in 4..27 { tiles[20][x] = W; }
    for y in 4..27 { tiles[y][10] = W; }
    for y in 4..27 { tiles[y][20] = W; }
    // Gaps
    tiles[2][15] = D;
    tiles[28][15] = D;
    tiles[15][2] = D;
    tiles[15][28] = D;
    tiles[10][15] = D;
    tiles[20][15] = D;
    tiles[15][10] = D;
    tiles[15][20] = D;
    // Power pellets
    tiles[3][3] = P;
    tiles[3][27] = P;
    tiles[27][3] = P;
    tiles[27][27] = P;
    tiles[3][15] = P;
    tiles[27][15] = P;
    // Ghost lair
    setup_lair(&mut tiles);
    // Portals
    tiles[1][1] = Tile::Portal(0);
    tiles[29][29] = Tile::Portal(0);
    tiles[1][29] = Tile::Portal(1);
    tiles[29][1] = Tile::Portal(1);

    LevelConfig {
        width: 31,
        height: 31,
        tiles,
        pacman_spawn: (15, 23),
        ghost_spawns: [(14, 15), (16, 15), (15, 14), (15, 16)],
        ghost_lair: (15, 15),
        portal_pair_a: ((1, 1), (29, 29)),
        portal_pair_b: ((1, 29), (29, 1)),
        ghost_speed_factor: 1.10,
        fright_duration_ms: 2111,
    }
}

fn make_level_9() -> LevelConfig {
    let mut tiles = [[W; 31]; 31];
    for y in 1..30 {
        for x in 1..30 {
            tiles[y][x] = D;
        }
    }
    // Complex wall pattern
    for i in 0..5 {
        let off = 2 + i * 5;
        for x in off..(29 - off) { tiles[off][x] = W; }
        for x in off..(29 - off) { tiles[29 - off][x] = W; }
        tiles[off][15] = D;
        tiles[29 - off][15] = D;
    }
    for i in 0..5 {
        let off = 2 + i * 5;
        for y in off..(29 - off) { tiles[y][off] = W; }
        for y in off..(29 - off) { tiles[y][29 - off] = W; }
        tiles[15][off] = D;
        tiles[15][29 - off] = D;
    }
    // Power pellets
    tiles[1][1] = P;
    tiles[1][29] = P;
    tiles[29][1] = P;
    tiles[29][29] = P;
    // Ghost lair
    setup_lair(&mut tiles);
    // Portals
    tiles[1][14] = Tile::Portal(0);
    tiles[29][16] = Tile::Portal(0);
    tiles[14][1] = Tile::Portal(1);
    tiles[16][29] = Tile::Portal(1);

    LevelConfig {
        width: 31,
        height: 31,
        tiles,
        pacman_spawn: (15, 23),
        ghost_spawns: [(14, 15), (16, 15), (15, 14), (15, 16)],
        ghost_lair: (15, 15),
        portal_pair_a: ((1, 14), (29, 16)),
        portal_pair_b: ((14, 1), (16, 29)),
        ghost_speed_factor: 1.15,
        fright_duration_ms: 1555,
    }
}

fn make_level_10() -> LevelConfig {
    let mut tiles = [[W; 31]; 31];
    for y in 1..30 {
        for x in 1..30 {
            tiles[y][x] = D;
        }
    }
    // Tight maze - many wall segments
    for y in 1..30 {
        for x in 1..30 {
            if ((x % 3 == 0 && y % 3 == 0) || (x % 4 == 1 && y % 4 == 1))
                && (x, y) != (15, 15)
            {
                tiles[y][x] = W;
            }
        }
    }
    // Ensure corridors exist
    tiles[3][15] = D;
    tiles[27][15] = D;
    tiles[15][3] = D;
    tiles[15][27] = D;
    // Power pellets
    tiles[1][1] = P;
    tiles[1][29] = P;
    tiles[29][1] = P;
    tiles[29][29] = P;
    tiles[1][15] = P;
    tiles[29][15] = P;
    tiles[15][1] = P;
    tiles[15][29] = P;
    // Ghost lair
    setup_lair(&mut tiles);
    // Portals
    tiles[1][1] = Tile::Portal(0);
    tiles[29][29] = Tile::Portal(0);
    tiles[1][29] = Tile::Portal(1);
    tiles[29][1] = Tile::Portal(1);

    LevelConfig {
        width: 31,
        height: 31,
        tiles,
        pacman_spawn: (15, 23),
        ghost_spawns: [(14, 15), (16, 15), (15, 14), (15, 16)],
        ghost_lair: (15, 15),
        portal_pair_a: ((1, 1), (29, 29)),
        portal_pair_b: ((1, 29), (29, 1)),
        ghost_speed_factor: 1.20,
        fright_duration_ms: 1000,
    }
}

use std::sync::LazyLock;

static LEVELS: LazyLock<[LevelConfig; 10]> = LazyLock::new(|| [
    make_level_1(),
    make_level_2(),
    make_level_3(),
    make_level_4(),
    make_level_5(),
    make_level_6(),
    make_level_7(),
    make_level_8(),
    make_level_9(),
    make_level_10(),
]);

pub fn get_level_config(level: u8) -> &'static LevelConfig {
    assert!((1..=10).contains(&level), "level out of bounds");
    &LEVELS[(level - 1) as usize]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::maze::Tile;

    #[test]
    fn test_all_levels_exist() {
        for l in 1..=10 {
            let cfg = get_level_config(l);
            assert_eq!(cfg.width, 31);
            assert_eq!(cfg.height, 31);
        }
    }

    #[test]
    #[should_panic]
    fn test_level_0_panics() {
        get_level_config(0);
    }

    #[test]
    #[should_panic]
    fn test_level_11_panics() {
        get_level_config(11);
    }

    #[test]
    fn test_portal_pairs_on_path_tiles() {
        for l in 1..=10 {
            let cfg = get_level_config(l);
            let ((a_x, a_y), (b_x, b_y)) = cfg.portal_pair_a;
            assert!(!matches!(cfg.tiles[a_y][a_x], Tile::Wall),
                "level {} portal_a_a is wall at ({},{})", l, a_x, a_y);
            assert!(!matches!(cfg.tiles[b_y][b_x], Tile::Wall),
                "level {} portal_a_b is wall at ({},{})", l, b_x, b_y);
            let ((c_x, c_y), (d_x, d_y)) = cfg.portal_pair_b;
            assert!(!matches!(cfg.tiles[c_y][c_x], Tile::Wall),
                "level {} portal_b_a is wall at ({},{})", l, c_x, c_y);
            assert!(!matches!(cfg.tiles[d_y][d_x], Tile::Wall),
                "level {} portal_b_b is wall at ({},{})", l, d_x, d_y);
        }
    }

    #[test]
    fn test_power_pellets_per_level() {
        for l in 1..=10 {
            let cfg = get_level_config(l);
            let pellet_count = cfg.tiles.iter().flatten()
                .filter(|t| matches!(t, Tile::PowerPellet))
                .count();
            assert!(pellet_count >= 4, "level {} has only {} power pellets", l, pellet_count);
        }
    }

    #[test]
    fn test_spawn_positions_on_path() {
        for l in 1..=10 {
            let cfg = get_level_config(l);
            let (px, py) = cfg.pacman_spawn;
            assert!(!matches!(cfg.tiles[py][px], Tile::Wall),
                "level {} pacman spawn on wall at ({},{})", l, px, py);
            for (gx, gy) in &cfg.ghost_spawns {
                assert!(!matches!(cfg.tiles[*gy][*gx], Tile::Wall),
                    "level {} ghost spawn on wall at ({},{})", l, gx, gy);
            }
        }
    }

    #[test]
    fn test_fright_duration_decreasing() {
        let mut prev = u32::MAX;
        for l in 1..=10 {
            let dur = get_level_config(l).fright_duration_ms;
            assert!(dur <= prev, "level {} fright duration not decreasing", l);
            prev = dur;
        }
    }

    #[test]
    fn test_ghost_speed_increasing() {
        let mut prev = 0.0;
        for l in 1..=10 {
            let speed = get_level_config(l).ghost_speed_factor;
            assert!(speed >= prev, "level {} ghost speed not increasing", l);
            prev = speed;
        }
    }
}
