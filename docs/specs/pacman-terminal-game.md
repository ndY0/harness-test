---
type: spec
domain: game
feature: pacman-terminal-game
status: active
date: 2026-06-27
superseded_by: none
superseded_date: none
complexity: complex
complexity_rationale: "Full Pacman game spanning 14+ source files across multiple modules: game logic (maze, stages, ghost AI, player, physics, pellets, scoring, timers), UI (renderer, HUD), and input — well over 200 lines of new code with clearly separable sub-problems and disjoint write sets."
---

# Spec: pacman-terminal-game

## Summary

This feature delivers a fully playable Pacman game that runs entirely in a POSIX
terminal with no audio, no external windowing system, and no runtime file I/O.
The player navigates a character-cell maze, eats dots and power pellets, avoids
or consumes ghosts, and progresses through 10 unique stages. The game is
distributed as a single Rust binary and is the sole feature of this project.
It replaces nothing and is the foundational deliverable of the pipeline.

## Acceptance criteria

1. **Compilation.** Running `cargo build --release` in the repository root
   succeeds with zero errors and produces a runnable binary. The binary accepts
   no required arguments and starts the game when invoked with no flags.

2. **Ten unique stages.** The game contains exactly 10 stages. Each stage is
   defined by a distinct 21-column × 21-row ASCII maze layout. No two stages
   share an identical layout. Stages are numbered 1–10 in the HUD.

3. **Player movement — arrows.** While the game is in the Playing state, pressing
   any of the four arrow keys (Up, Down, Left, Right) buffers the corresponding
   direction. On the next physics tick, the player moves one tile in the buffered
   direction if the target tile is not a wall; otherwise the player stays in place
   and the buffer is cleared.

4. **Player movement — WASD.** While the game is in the Playing state, pressing
   W (up), A (left), S (down), or D (right) behaves identically to the
   corresponding arrow key (criterion 3). Both input schemes must work
   simultaneously without any configuration change.

5. **Dot eating and stage clear.** When the player occupies a tile that contains
   an uneaten dot, that dot is marked eaten and the score increases by 10 points.
   When every dot and power pellet in the current stage has been eaten, the game
   transitions to the next stage (stage index increments by 1, player and ghost
   positions reset to their per-stage start coordinates, score and lives are
   preserved).

6. **Power pellet — frightened mode.** When the player occupies a tile that
   contains an uneaten power pellet, that pellet is marked eaten, the score
   increases by 50 points, and all four ghosts enter Frightened mode for exactly
   100 ticks (10 s at the 100 ms tick rate). During Frightened mode ghosts are
   rendered in blue (or fallback ASCII; see Technical Notes §rendering). During
   the last 30 ticks of Frightened mode ghosts blink (alternate blue/white
   foreground). When Frightened mode expires all ghosts return to their previous
   Chase or Scatter phase.

7. **Eating a frightened ghost.** When the player occupies the same tile as a
   ghost that is in Frightened mode, the ghost is eaten: it is sent back to the
   ghost house center, it is no longer frightened, and the score increases by
   200 × 2^(chain−1) points where chain is the count of ghosts eaten during the
   current Frightened mode activation (200, 400, 800, or 1 600 for the 1st, 2nd,
   3rd, and 4th ghost respectively). The ghost-chain counter resets when
   Frightened mode expires or when a new power pellet is eaten.

8. **Ghost collision — life lost.** When the player occupies the same tile as a
   ghost that is NOT in Frightened mode, the player loses one life, all positions
   reset to the per-stage start coordinates after a 1-second (10-tick) freeze,
   and the HUD displays "READY!" during the freeze. The ghost-chain counter
   resets on death.

9. **Lives system.** The player begins each game with exactly 3 lives. The HUD
   displays remaining lives as `♥` symbols (fallback: `*`). If the terminal
   cannot render U+2665, `*` is used instead. One extra life is awarded the
   first time the player's score reaches or exceeds 10 000 points; the HUD
   displays "+ EXTRA LIFE!" for 20 ticks (2 s) when this occurs. The extra life
   is awarded at most once per game.

10. **Game over screen.** When the player's life count reaches 0, the game
    transitions to the GameOver state. The terminal area displays a game-over
    screen containing the text "GAME OVER" and the final score. Pressing Q, Esc,
    or any arrow/WASD key while in GameOver state exits the process.

11. **Victory screen.** When the player clears stage 10 (all dots eaten in stage
    10 with lives > 0), the game transitions to the Victory state. The terminal
    area displays a congratulatory message and the final score. Pressing Q, Esc,
    or any arrow/WASD key while in Victory state exits the process.

12. **Score display in HUD.** At all times during Playing and Paused states, the
    HUD displays the current score as a zero-padded 5-digit integer (e.g.
    "SCORE: 00000"). The display updates each tick.

13. **Stage number display in HUD.** At all times during Playing and Paused
    states, the HUD displays the current stage number as "STAGE: N" where N is
    1–10. The display updates on each stage transition.

14. **Quit: Esc or Q.** Pressing Esc or Q at any time (including during Playing,
    Paused, GameOver, and Victory states) causes the process to exit and the
    terminal to be restored to cooked mode. The terminal must not be left in raw
    mode after the process exits.

15. **Pause: P or Space.** Pressing P or Space while in Playing state transitions
    to Paused state; pressing P or Space again while in Paused state resumes
    Playing state. While paused, the HUD displays "PAUSED". The game loop
    continues to poll for input; the tick step that advances ghost positions,
    timers, and physics is skipped while paused.

16. **Terminal too-small message.** On startup and on every terminal Resize event,
    the renderer checks terminal dimensions. If `cols < 80` or `rows < 30`, the
    entire terminal area is cleared and the following error widget is displayed:

    ```
    Terminal too small.
    Need at least 80×30. Current: <cols>×<rows>.
    Please resize and the game will resume automatically.
    ```

    The game loop continues running; when the terminal is resized to meet the
    minimum, normal rendering resumes on the next tick without user action.

17. **No audio, no external window.** The binary does not link any audio crate
    (rodio, cpal, or equivalent), SDL, OpenGL, Vulkan, Winit, or any windowing
    system. Running `cargo tree` must not show any such dependency. The game
    renders exclusively in the terminal's alternate screen buffer.

18. **Terminal lifecycle safety.** The terminal is restored to cooked mode on all
    exit paths: normal quit, panic, and SIGTERM/SIGINT (where supported by
    Crossterm). This is implemented via a `TerminalGuard` struct with a `Drop`
    implementation in `main.rs`. A Reviewer will treat any test or manual
    scenario in which the terminal is left in raw mode as a BLOCKING finding.

## API contracts

None — internal only. The binary exposes no HTTP endpoints, no IPC sockets, no
library API, and no event schemas. The only public interface is the compiled
binary invoked with no required arguments.

Optional CLI flags (`--stage N` for debug entry) are at the Domain Architect's
discretion; their presence or absence does not affect the acceptance criteria
above.

## Data

No persistent data. No save files. No high-score storage. All game state is
in-process and discarded when the binary exits.

**In-process state entities (informational — defined in architecture, not here):**

- `App` — top-level state machine (Menu | Playing | Paused | GameOver | Victory)
- `GameState` — live game state composed during the Playing/Paused states
- `Maze` — immutable 21×21 grid of `Cell` values parsed from stage ASCII
- `PelletMap` — mutable eaten-state for dots and power pellets (flat `Vec<bool>`)
- `Player` — position, facing direction, buffered input direction, lives count
- `Ghost` (×4 per stage) — position, AI mode, personality, scatter target
- `ScoreState` — score accumulator, ghost-chain counter, extra-life flag
- `GameTimers` — frightened ticks, scatter ticks, chase ticks, cycle count, death-pause ticks, extra-life display ticks

These structures are defined in the architecture document at
`docs/architecture/game-domain.md`. The Implementer must not introduce new
persistent entities without Domain Architect approval.

## Edge cases

- **Wall buffer rejection.** If the buffered direction would move the player into
  a wall, the move is silently skipped for that tick. The buffer is cleared. The
  player does not move.

- **Ghost house passthrough.** Ghost house tiles (`G`) are impassable to the
  player. A ghost occupying a ghost house tile does not collide with the player
  even if they share the same tile (ghosts leaving the house pass through the
  player's start region briefly).

- **Tunnel wrapping edge.** Wrapping is only applied when the entity is in
  column 0 moving left or column 20 moving right, and the target wrap cell is an
  open path. If the wrap destination is a wall, movement is blocked. Wrapping is
  active only in stages 3 and 7.

- **Power pellet during frightened mode.** If the player eats a power pellet
  while ghosts are already frightened, the frightened timer resets to 100 ticks
  and the ghost-chain counter resets to 0 (the new pellet starts a fresh chain).

- **All ghosts already eaten during frightened mode.** If all four ghosts are
  sent home before frightened mode expires, the chain counter is not incremented
  further. Frightened mode still expires on its timer.

- **Resize during death freeze.** If the terminal is resized below minimum during
  the 10-tick death freeze, the too-small widget is shown and the freeze timer
  continues counting. Normal rendering resumes when the terminal is large enough.

- **Score overflow.** `score` is stored as `u32`. The theoretical maximum score
  for a 10-stage game is well under 2^32; no overflow handling is required, but
  the `u32` type must be used (not `i32`).

- **Extra life awarded exactly once.** The `extra_life_awarded` flag in
  `ScoreState` is set on the tick the threshold is crossed and never reset during
  a single game session. Crossing 10 000 points a second time (impossible in a
  normal game but possible in testing) must not award a second extra life.

- **Simultaneous ghost collision.** If the player occupies the same tile as two
  ghosts in the same tick, and one is frightened and one is not, the non-frightened
  ghost takes priority: the player loses a life, and the frightened ghost is
  not consumed.

- **Ghost movement during death freeze.** During the 10-tick death pause, ghost
  positions are held; no ghost AI step is executed.

- **Quit during death freeze.** Pressing Q or Esc during the death-pause freeze
  exits the process immediately; the freeze does not block the quit path.

## Out of scope

- **Music or sound effects.** Explicitly excluded. No audio of any kind. (Charter
  constraint §4, system topology §6.4, and human request "no music".)

- **High-score persistence.** No scores are written to disk or read from disk.
  (System topology §5, infrastructure topology row "Persistence: None".)

- **Online or networked multiplayer.** Single-player only.

- **AI-controlled player.** No bot mode or attract mode. The player is always
  human-controlled.

- **Difficulty levels or settings UI.** All timing parameters are compile-time
  constants. No in-game options menu.

- **Bonus fruit items.** Score values for bonus items are documented as "None" in
  the domain architecture (§2.4). Bonus items are not implemented.

- **Classic Pacman death animation.** Death sequence is instant-freeze + respawn.
  No sprite-frame animation. (Domain architecture §2.5.)

- **Saving or loading game state.** No checkpointing, no continue-from-stage-N
  without the optional `--stage` flag (if the Domain Architect adds it for
  debugging).

- **Window icon, taskbar entry, or desktop integration.** Terminal-only.

- **Colour themes or accessibility modes.** A single colour scheme is used as
  defined in the domain architecture (§7). No configuration.

- **Joystick or gamepad input.** Keyboard only (arrows and WASD).

---

## Technical notes

### Reference documents

- System topology: `docs/architecture/system-topology.md`
- Domain architecture (authoritative for all design decisions):
  `docs/architecture/game-domain.md`
- Domain charter: `docs/architecture/charters/game-domain.md`

### Required files (from domain architecture §4)

```
Cargo.toml
src/
  main.rs          — TerminalGuard (Drop impl), event loop, graceful teardown
  app.rs           — App struct; state machine Menu|Playing|Paused|GameOver|Victory;
                     tick() and handle_input() methods
  game/
    mod.rs         — Re-exports; value types: Direction, Cell, GhostMode,
                     GameAction, Position
    maze.rs        — Maze struct (Vec<Vec<Cell>>); is_wall, is_ghost_house,
                     tunnel_row; ASCII→Maze parser
    stages.rs      — STAGES: [&[&str; 21]; 10] const array; pure data, no logic
    ghost.rs       — Ghost struct (pos, mode, personality, home_pos, scatter_target);
                     step(&maze, &player, &blinky_pos)
    player.rs      — Player struct (pos, facing, buffered_dir, lives);
                     buffer_direction, consume_direction
    physics.rs     — apply_move(entity, dir, &maze) -> Option<Position>;
                     tunnel wrapping; check_player_ghost collision
    pellets.rs     — PelletMap (Vec<bool> indexed by row*cols+col);
                     eat(pos)->CellType, all_eaten()->bool, active_positions()
    scoring.rs     — ScoreState (score: u32, ghost_chain: u8,
                     extra_life_awarded: bool);
                     add_dot, add_power_pellet, eat_ghost->u32, check_extra_life
    timers.rs      — GameTimers (frightened_ticks, scatter_ticks, chase_ticks,
                     cycle_count, death_pause_ticks, extra_life_display_ticks);
                     tick() -> Vec<TimerEvent>
  ui/
    mod.rs         — render(frame, &App); MIN_COLS=80, MIN_ROWS=30; size-check
                     logic; error widget
    renderer.rs    — maze_widget(&GameState) -> impl Widget; coloured character
                     cells; ghost and player rendering with Ratatui Span
    hud.rs         — hud_widget(&ScoreState, lives, stage, status_msg) -> impl Widget
  input.rs         — map_key(KeyEvent) -> Option<GameAction>; pure mapping, no state
```

### Cargo.toml dependencies

```toml
[package]
name = "pacman-terminal"
version = "0.1.0"
edition = "2021"

[dependencies]
ratatui   = "0.27"   # TUI framework — widgets, layout, terminal abstraction
crossterm = "0.27"   # Terminal backend — raw mode, event polling, ANSI output
anyhow    = "1"      # Error propagation — anyhow::Result<T> throughout call stack
```

No other runtime dependencies are permitted without a new ADR entry.

### Maze cell encoding (verbatim from domain architecture §2.2)

| Char | Meaning |
|------|---------|
| `#`  | Wall |
| `.`  | Dot |
| `o`  | Power pellet |
| `P`  | Player start position (treated as open path at runtime) |
| `G`  | Ghost house tile |
| ` `  | Empty path (no dot) |

### Ghost personality summary (domain architecture §2.1)

| Ghost | Colour | Chase target |
|-------|--------|-------------|
| Blinky (0) | Red | Player's current tile |
| Pinky (1) | Pink | Tile 4 steps ahead of player's facing |
| Inky (2) | Cyan | Random adjacent open tile |
| Clyde (3) | Orange | Player's tile if distance > 8; scatter corner if ≤ 8 |

### Timing constants (domain architecture §2.3)

| Parameter | Value |
|-----------|-------|
| Tick interval | 100 ms |
| Ghost move interval | 200 ms (every 2 ticks) |
| Frightened duration | 100 ticks (10 s) |
| Frightened blink onset | Last 30 ticks |
| Scatter phase | 70 ticks (7 s) |
| Chase phase | 200 ticks (20 s) |
| Scatter/chase cycle | Scatter→Chase→Scatter→Chase→Chase∞ |
| Death freeze | 10 ticks (1 s) |
| Extra life display | 20 ticks (2 s) |

### Rendering notes

- Ghost normal character: `ᗣ` (U+15E3) with per-personality foreground colour;
  ASCII fallback: `B`/`P`/`I`/`C` respectively.
- Ghost frightened: `ᗣ` blue foreground; blink = alternate blue/white during
  last 30 ticks.
- Player character: `C` yellow foreground.
- HUD layout: 1 row at top; maze centered in remaining rows and columns.
- Score format: `SCORE: 00000` (zero-padded to 5 digits).
- Lives format: `LIVES: ♥ ♥ ♥` (U+2665, fallback `*`).
- Stage format: `STAGE: N`.

### Decomposition Hint

Suggested sub-tasks for the Planner (non-binding):

```
S0: Scaffolding and types
    Write sets: Cargo.toml, src/main.rs (skeleton), src/app.rs (state machine
    enum only), src/game/mod.rs (value types), src/input.rs
    Dependencies: none

S1: Maze and stage data
    Write sets: src/game/maze.rs, src/game/stages.rs (all 10 layouts)
    Dependencies: S0

S2: Player, physics, and pellets
    Write sets: src/game/player.rs, src/game/physics.rs, src/game/pellets.rs
    Dependencies: S0, S1

S3: Ghost AI and timers
    Write sets: src/game/ghost.rs, src/game/timers.rs
    Dependencies: S0, S1

S4: Scoring
    Write sets: src/game/scoring.rs
    Dependencies: S0

S5: Rendering and HUD
    Write sets: src/ui/mod.rs, src/ui/renderer.rs, src/ui/hud.rs
    Dependencies: S0, S1, S2, S3, S4

S6: App tick loop wiring, terminal guard, integration tests
    Write sets: src/main.rs (complete), src/app.rs (complete), tests/
    Dependencies: S0–S5
```
