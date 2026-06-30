---
feature: pacman-terminal-game
subtask: C
title: "Pac-Man and Ghost entities, movement, collision, and ghost AI"
write_set: ["src/entities.rs", "src/ghosts.rs"]
depends_on: ["A"]
complexity: simple
planner_depth: 1
---

## Context

Sub-task C defines Pac-Man and Ghost structs, their movement logic with wall collision, ghost AI targeting personalities, and collision detection between entities. Depends on sub-task A for `Tile`, `Direction`, `MazeGrid`, `TileEffect` types.

## Inputs from spec

### Data types (from spec)

- `PacMan`: `pos: (usize, usize)`, `dir: Direction`, `next_dir: Direction`, `state: PacManState`
- `PacManState` enum: `Alive`, `Dying(u8)`, `Respawning`
- `Ghost`: `pos: (usize, usize)`, `dir: Direction`, `personality: GhostPersonality`, `mode: GhostMode`, `color: Color`
- `GhostPersonality` enum: `Blinky`, `Pinky`, `Inky`, `Clyde` — distinct target-tile selection
- `GhostMode` enum: `Chase`, `Scatter`, `Frightened(u32)`, `Eaten` — `Frightened` carries remaining-duration ticks
- `CollisionEvent` enum: `GhostContact`, `GhostEaten(u8)`, `None`
- `MoveResult` enum (Pac-Man move result) and `GhostEvent` enum (ghost move side-effects)

### Public API (from spec)

**`src/entities.rs`:**
- `fn move_pacman(state: &mut GameState, maze: &MazeGrid, dir: Direction) -> MoveResult` — but wait: `GameState` is in E. To keep write-sets disjoint, define `PacMan` struct and movement functions that operate on `PacMan` directly, not `GameState`. Re-scope to:
  - `fn move_pacman(pacman: &mut PacMan, maze: &MazeGrid) -> MoveResult` (uses pacman's buffered `next_dir`)

- `fn check_collisions(pacman: &PacMan, ghosts: &[Ghost]) -> Vec<CollisionEvent>`

**`src/ghosts.rs`:**
- `fn move_ghosts(ghosts: &mut [Ghost], maze: &MazeGrid, pacman_pos: (usize, usize), blinky_pos: (usize, usize)) -> Vec<GhostEvent>`
  Ghost AI per personality:
  - Blinky: targets Pac-Man's current position directly
  - Pinky: targets 4 cells ahead of Pac-Man's current direction
  - Inky: uses double-vector from Blinky (mirror Blinky's position relative to 2 cells ahead of Pac-Man)
  - Clyde: if distance to Pac-Man > 8 cells, chase like Blinky; otherwise target scatter corner (bottom-left)

  For Frightened mode: ghosts reverse direction when entering Frightened, move at half speed (every other tick), random direction at intersections. For Eaten mode: ghosts move toward ghost lair at fast speed, respawn in Chase mode upon arrival.

### Movement rules
- Ghosts move one cell per tick (fast speed) or one cell per 2 ticks (half speed = frightened)
- Movement checks wall collision via `is_wall()` from maze
- At intersections, ghosts select the direction that minimizes distance to their target tile
- Ghosts cannot reverse direction (except when entering Frightened mode)
- Ghosts do not interact with portal tiles (treat as empty)

### Pac-Man movement
- Uses `next_dir` (buffered input): if `next_dir` is set and the adjacent cell in that direction is not a wall, Pac-Man turns to that direction on next tick
- Pac-Man moves one cell per tick in `dir`
- Stops at walls
- Portals: stepping onto a portal tile teleports to paired portal (logic in E, but C needs a helper `fn get_portal_destination(maze: &MazeGrid, pos: (usize, usize)) -> Option<(usize, usize)>` that returns the other end of the portal)

## Acceptance criteria (subset)

- `move_pacman` respects wall collisions and buffered direction changes
- Ghosts select correct target tiles per personality
- Blinky targets Pac-Man directly
- Pinky targets 4 cells ahead of Pac-Man
- Inky uses double-vector from Blinky
- Clyde: chase >8 cells away, flee otherwise
- Frightened ghosts reverse direction and move at half speed
- Eaten ghosts move toward ghost lair
- `check_collisions` returns correct events: `GhostContact` when Pac-Man touches ghost in Chase/Scatter, `GhostEaten(chain_index)` when Pac-Man touches ghost in Frightened
- Movement functions handle edge cases: no wall-wrapping, no reversal except on Frightened entry

## Interface contracts

Sub-task C exports:
- `PacMan`, `PacManState`, `Ghost`, `GhostPersonality`, `GhostMode`, `CollisionEvent`, `MoveResult`, `GhostEvent`
- Functions: `move_pacman`, `check_collisions`, `move_ghosts`, `get_portal_destination`

Color: use a simple `Color` type — either from ratatui's `ratatui::style::Color` re-exported locally, or define a simple enum `GhostColor { Red, Pink, Cyan, Orange }` to avoid ratatui dependency in this module. Since ratatui is already a dependency, use `pub use ratatui::style::Color;` or re-export in entities. Actually, simplest: define ghost colors as an enum in entities.rs and let render.rs (D) map them to ratatui colors.

## Must not touch

- Any other `src/` file
- `Cargo.toml`
- `docs/`
- Maze data, rendering, input, scoring, or app loop
- `GameState` definition (that's E's job)
