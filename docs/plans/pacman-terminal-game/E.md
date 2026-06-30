---
feature: pacman-terminal-game
subtask: E
title: "App state machine, game loop, terminal safety, and entrypoint"
write_set: ["src/app.rs", "src/main.rs"]
depends_on: ["A", "B", "C", "D"]
complexity: simple
planner_depth: 1
---

## Context

Sub-task E integrates all other modules into a working application. It defines `GameState`, the `AppState` machine, the tick-based game loop, terminal safety (RAII raw-mode guard), and `main.rs` entrypoint. This replaces all existing `src/main.rs` content.

## Inputs from spec

### Data types

- `AppState` enum: `Menu`, `Playing`, `Paused`, `GameOver`, `Victory`
- `GameState`: `level: u8`, `lives: u8`, `score: u32`, `ghost_eat_chain: u8`, `power_pellet_timer: u32`, `state: AppState`, `maze: MazeGrid`, `pacman: PacMan`, `ghosts: [Ghost; 4]`, `fruit_spawned: [bool; 2]`, `fruit_active: bool`, `fruit_pos: Option<(usize,usize)>`
- `AppError` enum: `Io(std::io::Error)`, `Serde(serde_json::Error)`, `TerminalSize { width: u16, height: u16 }`

### Game loop (tick-based, 100–200ms)

Each tick:
1. Poll input via `poll_input()` → `Action`
2. Process action based on `AppState`:
   - **Menu**: Up/Down navigate menu items; Enter selects; Escape/Quit exits
   - **Playing**: Move action → buffer direction; Pause → transition to Paused; Quit → exit
   - **Paused**: Only process Quit or Escape (quit); ignore others
   - **GameOver**: Enter to submit name (if top-10) or return to menu; Escape to menu
   - **Victory**: Enter or Escape → transition to GameOver or name entry
3. Update entity positions: `move_pacman()`, `move_ghosts()`
4. Check collisions: `check_collisions()`
5. Process tile consumption: `consume_dot()`, portal teleportation
6. Process fruit spawning (170 and 70 dots remaining thresholds)
7. Update scoring via `add_points()`
8. Check win/lose conditions, level transitions
9. Render current state via render functions

### Terminal safety

- RAII guard struct `TerminalGuard` that:
  - Enables raw mode on creation
  - Enters alternate screen on creation
  - Disables raw mode on drop
  - Leaves alternate screen on drop
- Installed panic hook that restores terminal before exit
- All exit paths (normal quit, error, panic) restore terminal

### Entrypoint (`main.rs`)

- Check terminal size: if <80x24, print error to stderr, exit code 1
- Create `TerminalGuard`
- Initialize ratatui `Terminal`
- Enter game loop
- On Ctrl+C: terminal guard drop restores terminal, exit code 0
- On fatal I/O error: print to stderr, exit code 1

### Level transitions

- When `dots_remaining() == 0`:
  - Award level-completion bonus `100 * level`
  - If `level == 10`: transition to `Victory`
  - Else: increment level, rebuild `GameState` with new `LevelConfig`
- During transition: spawn positions may need ghost displacement if occupied

### Portal teleportation

- When Pac-Man steps onto a portal tile, teleport to paired portal same tick
- If destination occupied by ghost in Chase mode → lose a life, respawn at spawn

### Game Over flow

- When lives reach 0: transition to GameOver
- Check `is_top_10()`:
  - If true: prompt for 3-character name, on Enter call `insert_score()`, `save_scores()`
  - If false: show "Press Enter to continue" → return to Menu

### Resize handling

- Check terminal size each tick
- If <80x24: pause game, render "Terminal too small" message
- If ≥80x24 again: resume from Paused
- During resize-pause: only Quit/Escape/Ctrl+C processed

### Ghost AI integration

- Ghost AI called each tick for target selection
- Power pellet consumption: set Frightened mode on all ghosts, set timer
- Frightened timer decrements each tick
- Flashing period: last 2000ms, ghosts alternate appearance each 200ms tick
- When timer expires: all ghosts return to Chase mode, `ghost_eat_chain` resets to 0

## Acceptance criteria (subset)

1. Launch: main menu with 3 items, arrow key nav, Enter selects, Escape quits
2. Terminal size check: <80x24 → error message, exit code 1
3. Gameplay: arrow keys move Pac-Man one cell per tick, buffered direction changes
4. Dot consumption: dots removed, score incremented by 10 each
5. All dots on level → next level; level 10 all dots → Victory
6. Ghost contact in Chase/Scatter → lose life, reset positions; 0 lives → GameOver
7. Power pellet: ghosts enter Frightened, timer counts down, flashing period, ghost-eat chain
8. Portal: teleport to paired portal, ghost-on-destination → lose life
9. Fruit: spawn at 170 and 70 dots remaining, collectable, per-level point values
10. Escape/Ctrl+C at any time terminates, terminal restored
11. Resize below 80×24 → pause with message; resize back → resume
12. Top-10 score → name entry (3 uppercase alphanumeric), persistent save
13. All exit paths restore terminal (RAII drop guard)

## Interface contracts

Sub-task E imports and wires together:
- `maze::{MazeGrid, Direction, Tile, TileEffect, is_wall, is_portal, consume_dot, dots_remaining}`
- `levels::{LevelConfig, get_level_config}`
- `scoring::{add_points, lose_life, level_bonus, fruit_value, ghost_eat_chain bonus}`
- `scoreboard::{ScoreEntry, load_scores, save_scores, is_top_10, insert_score}`
- `entities::{PacMan, PacManState, Ghost, GhostPersonality, GhostMode, CollisionEvent, move_pacman, check_collisions}`
- `ghosts::{move_ghosts, GhostEvent}`
- `input::{Action, poll_input, check_resize}`
- `render::{render_menu, render_game, render_high_scores, render_game_over, render_victory, render_pause_overlay, render_terminal_too_small}`

E defines: `AppState`, `GameState`, `AppError`, `TerminalGuard`, `main()`.

## Must not touch

- Any other `src/` file (only `src/app.rs` and `src/main.rs`)
- `Cargo.toml` (unless adding needed `libc` or similar for signal handling — avoid if possible, use crossterm's built-in)
- `docs/`
- Implementation details of maze, scoring, entities, ghosts, input, or render
