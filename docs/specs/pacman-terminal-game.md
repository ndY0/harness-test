---
complexity: complex
complexity_rationale: "9+ modules across the entire codebase — App, Maze, Entities, GhostAI, Input, Render, Scoring, Scoreboard, Levels — with real-time game loop, independent concurrent components (ghost AI, input polling, rendering), and >200 new lines of Rust."
---

# Spec: Pac-Man Terminal Game

## Decomposition Hint

Sub-tasks with disjoint write-sets for parallel development:

| Sub-task ID | Modules | Depends on | Description |
|-------------|---------|------------|-------------|
| A | `src/maze.rs`, `src/levels.rs` | none | Grid tile types, maze data structure, 10 level configurations (inline Rust structs), portal pair coordinates, spawn points, per-level difficulty params. |
| B | `src/scoring.rs`, `src/scoreboard.rs` | A (types only) | Score accumulator, ghost-eat chain multiplier, fruit values per level, lives tracking, XDG-compliant JSON persistence, atomic write, top-10 ranking. |
| C | `src/entities.rs`, `src/ghosts.rs` | A (types only) | Pac-Man struct (position, direction, state), Ghost struct (position, direction, personality, mode), movement with wall collision, ghost AI targeting (Blinky/Pinky/Inky/Clyde). |
| D | `src/input.rs`, `src/render.rs` | A, C (types only) | crossterm event polling → action mapping, ratatui frame rendering (maze grid, entities with Unicode glyphs and color, HUD, menus, game-over screen). |
| E | `src/app.rs`, `src/main.rs` | A, B, C, D | Tick-based game loop (100–200ms), state machine (Menu/Playing/Paused/GameOver), per-tick orchestration of all components, terminal safety (RAII raw-mode guard), entrypoint. |

Sub-tasks A and B have zero code dependencies on each other and can be built in parallel first. C depends on A's type definitions. D depends on A and C's type definitions. E integrates everything.

## Summary

A single-binary terminal application that implements the classic Pac-Man arcade game using ratatui and crossterm. The player controls Pac-Man through 10 unique maze levels, eating dots while avoiding four ghosts. Power pellets temporarily invert the predator-prey relationship. Two teleportation portals per level provide tactical escape routes. A persistent top-10 high score board survives application restarts. The application replaces all existing `src/main.rs` placeholder code and uses only the crates declared in `Cargo.toml`.

## Acceptance criteria

1. Launching the binary via `cargo run` or direct invocation displays a main menu with three selectable items: "Start Game", "High Scores", and "Quit". Arrow keys navigate between items; Enter activates the selection; Escape quits. The game refuses to launch if the terminal is smaller than 80 columns × 24 rows, displaying an error message and exiting with code 1.

2. During gameplay, holding an arrow key moves Pac-Man one grid cell per tick in the held direction. Pac-Man stops only when colliding with a wall tile. Direction changes are buffered — pressing a new arrow key while Pac-Man is between cells causes the next move to use the new direction if the adjacent cell in that direction is not a wall. Release of all arrow keys stops movement at the next cell boundary.

3. Consuming a dot tile (10 points) removes it from the maze. Consuming all dot tiles on the current level immediately advances the game to the next level. Consuming all dots on level 10 triggers a victory screen with final score and the option to enter a high-score name.

4. Contact between Pac-Man and any ghost in Chase or Scatter mode decrements lives by 1 and resets Pac-Man and all ghosts to their spawn positions. If lives reach 0, the game transitions to the Game Over state. Pac-Man starts each session with 3 lives.

5. Eating a power pellet (50 points) toggles all ghosts to Frightened mode for a per-level duration (decreasing from 6000ms at level 1 to 1000ms at level 10 on a linear scale). During Frightened mode, ghosts reverse their current direction and move at half speed. Eating a Frightened ghost awards sequential bonus points: 200 for the first ghost eaten on the current power-pellet activation, then 400, 800, and 1600 for subsequent ghosts. The multiplier resets when the power pellet expires. Ghosts eaten in Frightened mode respawn at the ghost lair in Chase mode. Frightened mode expiration is preceded by a 2000ms flashing period during which ghosts alternate between Frightened and Chase appearance each 200ms tick.

6. Stepping onto a portal tile teleports Pac-Man to the paired portal tile on the same level, preserving his current direction. Teleportation occurs on the same tick as the move that entered the portal tile. If the destination portal's exit tile is occupied by a ghost in Chase mode, a life is lost and Pac-Man respawns at spawn. Ghosts do not interact with portal tiles — they treat portal tiles as empty passable paths.

7. A fruit bonus item spawns at the maze center at two dot-consumption thresholds per level: when 70 dots remain and when 170 dots remain. The fruit persists until collected or until the level is cleared. Fruit point values per level: 100, 300, 500, 700, 1000, 2000, 3000, 5000, 5000, 5000. Completing a level awards a flat bonus of `100 * current_level_number`.

8. When the game ends and the player's score ranks in the top 10, the game prompts for a 3-character uppercase alphanumeric name using keyboard input (A-Z, 0-9). The player confirms with Enter. The high score list is persisted as a JSON array of `{"name": "AAA", "score": 12345}` objects in `~/.local/share/pacman-terminal/scores.json`. The file is written atomically (write to temp file in same directory, then rename). On startup, if the file is missing or unparseable, the scoreboard initializes as empty. If the directory `~/.local/share/pacman-terminal/` does not exist, it is created with permission `0o700`.

9. If the terminal is resized below 80×24 during gameplay, the game loop pauses and renders a centered message: "Terminal too small — resize to at least 80×24 to continue." Gameplay resumes automatically when the terminal returns to ≥80×24. While paused, input events are discarded except for Ctrl+C and Escape, both of which quit the application.

10. Pressing Escape at any time terminates the application. Ctrl+C (SIGINT) terminates the application. In all exit paths — normal quit, error, or panic — raw mode is disabled and the alternate screen is restored via a drop guard or RAII wrapper, ensuring the user's terminal is left in a usable state. Normal exit returns code 0; fatal I/O errors print a message to stderr and return code 1.

## API contracts

None — internal only. All component interactions are direct Rust function calls within the `game` domain. The only external interface is filesystem I/O for high-score persistence (`~/.local/share/pacman-terminal/scores.json`).

### Internal function signatures (key contracts)

**Maze — `maze.rs`**
```
fn is_wall(grid: &MazeGrid, pos: (usize, usize)) -> bool
fn is_portal(grid: &MazeGrid, pos: (usize, usize)) -> Option<usize>  // returns portal_pair_index
fn consume_dot(grid: &mut MazeGrid, pos: (usize, usize)) -> TileEffect
fn dots_remaining(grid: &MazeGrid) -> u32
```
`TileEffect` enum: `Dot(10)`, `PowerPellet(50)`, `Portal(usize)`, `Fruit(u32)`, `Empty`, `Wall`.

**Entities — `entities.rs`**
```
fn move_pacman(state: &mut GameState, maze: &MazeGrid, dir: Direction) -> MoveResult
fn move_ghosts(state: &mut GameState, maze: &MazeGrid, ai: &GhostAI) -> Vec<GhostEvent>
fn check_collisions(pacman: &PacMan, ghosts: &[Ghost]) -> Vec<CollisionEvent>
```
`CollisionEvent` enum: `GhostContact`, `GhostEaten(u8)`, `None`.

**Scoring — `scoring.rs`**
```
fn add_points(score: &mut Score, event: ScoreEvent)
fn lose_life(state: &mut GameState) -> bool  // returns true if game over
fn level_bonus(level: u8) -> u32  // returns 100 * level
fn fruit_value(level: u8) -> u32
```
`ScoreEvent` enum: `Dot`, `PowerPellet`, `Ghost(u8)`, `Fruit(u32)`, `LevelComplete(u8)`.

**Scoreboard — `scoreboard.rs`**
```
fn load_scores() -> io::Result<Vec<ScoreEntry>>
fn save_scores(entries: &[ScoreEntry]) -> io::Result<()>
fn is_top_10(entries: &[ScoreEntry], score: u32) -> bool
fn insert_score(entries: &mut Vec<ScoreEntry>, entry: ScoreEntry)  // maintains top-10, sorted descending
```
`ScoreEntry`: `{ name: String, score: u32 }`. Max 10 entries, sorted by score descending.

**Levels — `levels.rs`**
```
fn get_level_config(level: u8) -> &'static LevelConfig  // panics if level > 10 (programmer bug)
```
`LevelConfig`: maze layout (2D tile array), Pac-Man spawn (x,y), ghost spawns [(x,y); 4], portal_pair_1 ((x1,y1),(x2,y2)), portal_pair_2 ((x1,y1),(x2,y2)), ghost_speed_factor (f32), fright_duration_ms (u32).

### Error handling contract

All fallible operations return `Result<T, AppError>`. `AppError` is an enum with variants: `Io(std::io::Error)`, `Serde(serde_json::Error)`, `TerminalSize { width: u16, height: u16 }`. The main loop converts errors to clean exit with terminal restoration. Panics are reserved for invariant violations only (e.g., level index out of bounds).

## Data

**New types (all in-memory, no persistence beyond scoreboard):**

| Type | Fields | Notes |
|------|--------|-------|
| `Tile` enum | `Wall`, `Dot`, `PowerPellet`, `Portal(usize)`, `Empty` | `usize` on Portal is pair index (0 or 1) |
| `MazeGrid` | `tiles: [[Tile; 31]; 31]`, `consumed: Bitmap` | 31×31 grid. Dot consumption tracked via bitmap, not mutation of `tiles`. |
| `Direction` enum | `Up`, `Down`, `Left`, `Right`, `None` | |
| `PacMan` | `pos: (usize, usize)`, `dir: Direction`, `next_dir: Direction`, `state: PacManState` | `next_dir` is buffered input direction |
| `PacManState` enum | `Alive`, `Dying(u8)`, `Respawning` | `Dying` counter for death animation ticks |
| `Ghost` | `pos: (usize, usize)`, `dir: Direction`, `personality: GhostPersonality`, `mode: GhostMode`, `color: Color` | |
| `GhostPersonality` enum | `Blinky`, `Pinky`, `Inky`, `Clyde` | Determines target-tile selection algorithm |
| `GhostMode` enum | `Chase`, `Scatter`, `Frightened(u32)`, `Eaten` | `Frightened` carries remaining-duration ticks |
| `GameState` | `level: u8`, `lives: u8`, `score: u32`, `ghost_eat_chain: u8`, `power_pellet_timer: u32`, `state: AppState`, `maze: MazeGrid`, `pacman: PacMan`, `ghosts: [Ghost; 4]`, `fruit_spawned: [bool; 2]`, `fruit_active: bool`, `fruit_pos: Option<(usize,usize)>` | Ephemeral — rebuilt on level transition |
| `AppState` enum | `Menu`, `Playing`, `Paused`, `GameOver`, `Victory` | Top-level state machine |
| `ScoreEntry` | `name: String`, `score: u32` | Persisted to JSON |
| `LevelConfig` | `width: usize, height: usize, tiles: [[Tile; 31]; 31], pacman_spawn: (usize, usize), ghost_spawns: [(usize,usize); 4], ghost_lair: (usize, usize), portal_pair_a: ((usize,usize),(usize,usize)), portal_pair_b: ((usize,usize),(usize,usize)), ghost_speed_factor: f32, fright_duration_ms: u32` | Compile-time constant. 10 instances defined inline. |
| `AppError` enum | `Io(std::io::Error)`, `Serde(serde_json::Error)`, `TerminalSize { width: u16, height: u16 }` | |

**Constraints on data:**
- `MazeGrid` is always 31×31 (odd dimensions so corridors are 1-cell wide with walls on even rows/cols).
- `name` in `ScoreEntry` is exactly 3 uppercase alphanumeric characters.
- `score` in `ScoreEntry` is a `u32` (max ~4.29 billion — effectively unbounded).
- `lives` starts at 3, cannot exceed 99 (no extra-life mechanic specified).
- `level` ranges 1–10.
- `ghost_eat_chain` ranges 0–4, reset to 0 when power pellet expires.
- Portal pairs are validated at definition time: both tiles of a pair must be path tiles (not walls).

## Edge cases

- Pac-Man and a ghost occupy the same tile on the same tick — resolve collision before rendering, ghost contact takes priority over dot consumption.
- Power pellet is eaten on the same tick as ghost contact — power pellet effect applies first, ghost is eaten (not Pac-Man), `ghost_eat_chain` starts at 200 points.
- Frightened mode expires on the same tick Pac-Man occupies a ghost's tile — ghost reverts to Chase, Pac-Man loses a life.
- Level transition occurs simultaneously with ghost contact (last dot eaten moves Pac-Man onto a ghost) — level transition takes priority, no life lost.
- Holding an arrow key into a wall for many ticks produces no movement and no error.
- Rapid direction changes (e.g., alternating Up/Left every tick at a corridor junction) — only the most recent buffered direction applies; no input queue overflow.
- Scoreboard file directory `~/.local/share/pacman-terminal/` is not writable — game launches but scores are not persisted; a warning is printed to stderr but the game continues.
- Scoreboard file exists but contains invalid JSON — treated as absent, scoreboard initializes empty, no error shown to user beyond the stderr warning.
- Inserting a new score when the scoreboard already has 10 entries — if the new score is higher than the lowest, the lowest is dropped; ties are broken by retaining the older entry.
- Player enters a blank or fewer-than-3-character name — input is rejected; prompt re-displays until 3 characters are entered.
- All four ghosts are eaten on a single power pellet activation — chain progresses 200→400→800→1600; fifth ghost eat is impossible since only 4 ghosts exist.
- Fruit spawns while Pac-Man is occupying the maze center tile — fruit is collected immediately on spawn tick, no visual rendered.
- Terminal resize during paused state — pause message re-centers; if new size ≥80×24, gameplay resumes.
- Spawning at the start of a level when a ghost occupies the spawn tile — ghost is displaced to the nearest empty tile (or ghost lair).

## Out of scope

- Scatter/chase mode cycling — simplified ghost AI uses continuous Chase mode with distinct targeting personalities per the architecture decision (see `docs/architecture/game.md` §Technology decisions, Ghost AI).
- Ghost use of portals — portals are player-only per architecture decision (`docs/architecture/game.md` §Technology decisions, Ghost-portal).
- Ghost eye-return animation — eaten ghosts respawn instantly at the ghost lair; open architecture question left to Implementer discretion.
- Pac-Man mouth open/close animation — single Unicode glyph (●) used for all Pac-Man rendering.
- Mouse input — keyboard-only per ADR-007 (`docs/architecture/standards.md`).
- Multiplayer or co-op modes — listed as Direction 5 in brainstorm but not selected.
- Level editor — listed as Direction 8 in brainstorm but not selected.
- Procedural maze generation — levels are hand-crafted inline Rust structs per architecture decision (`docs/architecture/game.md` §Technology decisions, Level storage).
- Extra-life awards at score thresholds — no mechanism defined; lives are fixed at 3 per session.
- Adjustable difficulty or game-speed settings — no configuration surface beyond what levels define internally.
- Sound or audio output of any kind.
- Windows-specific path handling — only Linux/macOS XDG paths are guaranteed; the architecture notes Windows is not guaranteed in v1 (`docs/architecture/system-topology.md` §Deployment topology).
- Maze grid dimensions other than 31×31.
