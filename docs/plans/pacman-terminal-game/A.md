---
feature: pacman-terminal-game
subtask: A
title: "Maze grid, tile types, and 10 level configurations"
write_set: ["src/maze.rs", "src/levels.rs"]
depends_on: []
complexity: simple
planner_depth: 1
---

## Context

Sub-task A defines the foundational data types and level data that all other sub-tasks depend on. It produces `src/maze.rs` and `src/levels.rs`.

## Inputs from spec

- `Tile` enum: `Wall`, `Dot`, `PowerPellet`, `Portal(usize)`, `Empty`
- `MazeGrid`: `tiles: [[Tile; 31]; 31]`, `consumed: Bitmap` (dot consumption tracked via bitmap, not tile mutation)
- `Direction` enum: `Up`, `Down`, `Left`, `Right`, `None`
- `TileEffect` enum: `Dot(10)`, `PowerPellet(50)`, `Portal(usize)`, `Fruit(u32)`, `Empty`, `Wall`
- `LevelConfig`: `width: usize, height: usize, tiles: [[Tile; 31]; 31], pacman_spawn: (usize, usize), ghost_spawns: [(usize, usize); 4], ghost_lair: (usize, usize), portal_pair_a: ((usize, usize), (usize, usize)), portal_pair_b: ((usize, usize), (usize, usize)), ghost_speed_factor: f32, fright_duration_ms: u32`
- 10 level configs defined as `static`/`const` inline Rust data
- Portal pair validation: both tiles must be path tiles (not walls)

### Public API (from spec)

**`src/maze.rs`:**
- `fn is_wall(grid: &MazeGrid, pos: (usize, usize)) -> bool`
- `fn is_portal(grid: &MazeGrid, pos: (usize, usize)) -> Option<usize>` — returns portal_pair_index (0 or 1)
- `fn consume_dot(grid: &mut MazeGrid, pos: (usize, usize)) -> TileEffect`
- `fn dots_remaining(grid: &MazeGrid) -> u32`

**`src/levels.rs`:**
- `fn get_level_config(level: u8) -> &'static LevelConfig` — panics if level > 10 (programmer bug)

## Acceptance criteria (subset)

- `MazeGrid` is 31×31 with odd dimensions
- `consume_dot` removes a dot via the bitmap, returns `Dot(10)` for Dot, `PowerPellet(50)` for PowerPellet, `Portal(usize)` for Portal, `Fruit(u32)` if fruit present, `Empty` if already consumed or empty, `Wall` for walls
- `dots_remaining` returns total unconsumed dot + power-pellet count
- `is_wall` correctly identifies Wall tiles
- `is_portal` returns the pair index for Portal tiles
- 10 `LevelConfig` instances exist: `get_level_config(1)` through `get_level_config(10)` all return valid configs
- `get_level_config(0)` or `get_level_config(11)` panics
- Each level's portal pairs are on path tiles (not walls)
- Each level has at least 4 power pellets
- Spawn positions are on path tiles

## Interface contracts

Sub-task A exports these types/functions for other sub-tasks:
- `Tile`, `MazeGrid`, `Direction`, `TileEffect` (from `maze.rs`)
- `LevelConfig`, `get_level_config` (from `levels.rs`)

These types must be `pub` and used as specified in the spec's internal function signatures.

## Must not touch

- Any other `src/` file
- `Cargo.toml`
- `docs/`
- Ghost AI, rendering, input, scoring, or app logic
