---
type: architecture
domain: game
feature: pacman-terminal-game
status: active
date: 2026-06-27
superseded_by: none
superseded_date: none
complexity: complex
complexity_rationale: "Ten unique maze layouts, per-ghost AI personalities, scatter/chase/frightened timing state machine, tunnel wrapping in two stages, ghost-chain scoring, terminal lifecycle guard, and a stateless Ratatui renderer all interact through a single tick loop — too many coupled subsystems for a single Implementer context window."
---

# Game Domain Architecture — Pacman Terminal Game

## 1. Scope

This document is the authoritative design reference for the `game` domain. It
resolves all open questions from the Game Domain Charter and the System Topology,
and provides the 10 maze layouts implementers must embed verbatim in `stages.rs`.

Constraints inherited from the charter and system topology are **not repeated
here in full**; this document only records decisions and additions within the
Domain Architect's authority.

---

## 2. Resolved Design Decisions

### 2.1 Ghost AI

Four ghosts per stage, each with a distinct targeting personality:

| Ghost  | Name   | Colour  | Targeting rule |
|--------|--------|---------|----------------|
| Ghost 0 | Blinky | Red    | Targets the player's current tile directly. |
| Ghost 1 | Pinky  | Pink   | Targets the tile 4 steps ahead of the player's facing direction. |
| Ghost 2 | Inky   | Cyan   | Moves to a random adjacent open tile each step (simplified from the classic mirror offset). |
| Ghost 3 | Clyde  | Orange | Chases like Blinky when distance > 8 tiles (Chebyshev); flees to its scatter corner when distance ≤ 8 tiles. |

All ghosts use integer tile coordinates. Pathfinding is greedy (choose the
adjacent tile that minimises Manhattan distance to target, excluding the tile
the ghost just came from to prevent oscillation). In **Frightened** mode all
ghosts move to a random adjacent open tile each step regardless of personality.
In **Scatter** mode each ghost targets a fixed corner of the maze:

| Ghost  | Scatter corner |
|--------|---------------|
| Blinky | top-right     |
| Pinky  | top-left      |
| Inky   | bottom-right  |
| Clyde  | bottom-left   |

**Ghost house exit rule:** Ghosts exit the house on a dot-count threshold.
Pinky exits after 0 dots eaten (immediately). Inky exits after 30 dots eaten.
Clyde exits after 60 dots eaten. Blinky starts outside the house on every stage.
After death, a ghost returns to the house center and re-applies its threshold
relative to dots eaten at the time of death.

### 2.2 Stage Layout Format

Hardcoded ASCII string slices in `stages.rs` via a `const` array of `&str`
slices. No runtime file I/O, no `serde`, no embedded asset loading. Each stage
is a `const &[&str; 21]` of 21-character rows.

**Cell encoding:**

| Character | Meaning |
|-----------|---------|
| `#` | Wall |
| `.` | Dot |
| `o` | Power pellet |
| `P` | Player start position |
| `G` | Ghost house tile |
| ` ` (space) | Empty path (no dot) |

Mazes are 21 columns × 21 rows. The player start `P` is always treated as an
open path; it is the initial position only and does not persist in the cell map.
Ghost house tiles (`G`) form a 2×2 block near the center; ghosts inside the
house do not interact with the player.

### 2.3 Timing Parameters

| Parameter | Value | Notes |
|-----------|-------|-------|
| Tick interval | 100 ms | One `app.tick()` call per tick; also the render cadence |
| Ghost move interval | 200 ms (every 2 ticks) | Ghosts advance one tile per 2 ticks |
| Frightened duration | 10 s (100 ticks) | Same across all stages |
| Scatter phase | 7 s (70 ticks) | First and third scatter phases |
| Chase phase | 20 s (200 ticks) | Repeating; no difficulty scaling across stages |
| Scatter/chase cycle | Scatter → Chase → Scatter → Chase → Chase∞ | After two full cycles, ghosts stay in chase mode permanently |
| Ghost house exit check | Per-tick dot count comparison | See §2.1 |
| Death pause | 1 s (10 ticks) | Game frozen; player and ghosts held at positions |

Render and game logic share the same 100 ms tick. Rendering inside the Ratatui
`draw` closure is stateless and cheap; 100 ms is well within budget.

### 2.4 Score Values

| Event | Points |
|-------|--------|
| Eat dot | 10 |
| Eat power pellet | 50 |
| Eat 1st ghost (frightened) | 200 |
| Eat 2nd ghost (chain) | 400 |
| Eat 3rd ghost (chain) | 800 |
| Eat 4th ghost (chain) | 1 600 |
| Bonus items | None |

The ghost-chain multiplier resets when frightened mode ends or when a new power
pellet is eaten. The chain counter is stored in `scoring.rs` as a `u8` (0–4).

### 2.5 Death Sequence

Instant respawn. On contact with a non-frightened ghost:

1. `player.lose_life()` decrements the life counter.
2. If lives > 0: game freezes for 10 ticks (1 s), then all positions reset to
   their per-stage start coordinates. No visual animation. The HUD flashes
   "READY!" during the pause.
3. If lives == 0: transition to `GameOver` state immediately.

No sprite-frame death animation. The terminal character-cell grid makes
multi-frame sprite cycles impractical without significant complexity.

### 2.6 Tunnel Wrapping

Stages **3** and **7** include left-right tunnels. A tunnel is represented by
open path cells (`.`) on the leftmost and rightmost column of the same row.
`physics.rs` implements modular column arithmetic:

```rust
// Wrapping move in physics.rs
let new_col = (player.col as isize + delta_col)
    .rem_euclid(maze.cols() as isize) as usize;
```

This wrapping is applied **only** when the player or ghost occupies a cell in
column 0 or column 20 and attempts to move further in that direction. No other
physics uses modular arithmetic.

### 2.7 Lives

3 starting lives. The HUD renders lives as `♥` symbols (Unicode U+2665). If the
terminal cannot render Unicode, a fallback of `*` is acceptable; the renderer
should attempt Unicode first.

### 2.8 Terminal Minimum Size

```rust
pub const MIN_COLS: u16 = 80;
pub const MIN_ROWS: u16 = 30;
```

Defined in `ui/mod.rs`. On startup and on every `Resize` event, the renderer
checks `terminal.size()`. If either dimension is below the minimum, the entire
terminal area is cleared and a single error widget is rendered:

```
Terminal too small.
Need at least 80×30. Current: <cols>×<rows>.
Please resize and the game will resume automatically.
```

The game loop continues running (no exit); when the terminal is resized to
meet the minimum, normal rendering resumes on the next tick.

### 2.9 Ghost Count Per Stage

All 10 stages use exactly 4 ghosts. The ghost house `G` block always contains
exactly 4 ghost start positions (the 2×2 block plus Blinky's start outside it).
Blinky begins 1 tile above the ghost house center; the other three begin inside
the house.

### 2.10 Extra Life

One extra life is awarded at 10 000 points. The award is given once per game
(tracked by a `bool extra_life_awarded` in `scoring.rs`). The HUD briefly
displays "+ EXTRA LIFE!" for 2 s (20 ticks) when the threshold is crossed.

---

## 3. Cargo.toml Dependencies

```toml
[package]
name = "pacman-terminal"
version = "0.1.0"
edition = "2021"

[dependencies]
ratatui  = "0.27"   # TUI framework — widgets, layout, terminal abstraction
crossterm = "0.27"  # Terminal backend — raw mode, event polling, ANSI output
anyhow   = "1"      # Error propagation — anyhow::Result<T> throughout call stack
```

`color-eyre` is not used; `anyhow` is simpler and sufficient for a single-binary
game with no structured error reporting requirements.

No other runtime dependencies are permitted without a new ADR entry.

---

## 4. Module Structure

Refinement of the system topology proposal. File ownership is final for
implementers; module additions require Domain Architect approval.

```
src/
  main.rs               — Terminal guard setup, event loop, graceful teardown.
                          Owns: TerminalGuard (Drop impl restores raw mode).
  app.rs                — App struct. State machine: Menu | Playing | Paused |
                          GameOver | Victory. Owns tick() and handle_input().
  game/
    mod.rs              — Re-exports. Value types: Direction, Cell, GhostMode,
                          GameAction, Position. No logic.
    maze.rs             — Maze struct: Vec<Vec<Cell>>. Methods: is_wall(pos),
                          is_ghost_house(pos), tunnel_row(row) -> bool.
                          Stage ASCII → Maze parsing lives here.
    stages.rs           — STAGES: [&[&str; 21]; 10] const array. One entry per
                          stage. No logic — pure data.
    ghost.rs            — Ghost struct: pos, mode (Chase|Scatter|Frightened),
                          personality (Blinky|Pinky|Inky|Clyde), home_pos,
                          scatter_target. Methods: step(&maze, &player, &blinky_pos).
    player.rs           — Player struct: pos, facing, buffered_dir, lives.
                          Methods: buffer_direction, consume_direction.
    physics.rs          — Movement resolution. apply_move(entity, dir, &maze) ->
                          Option<Position>. Tunnel wrapping here. Collision:
                          check_player_ghost(player_pos, ghost_positions).
    pellets.rs          — PelletMap struct: BitVec of eaten state indexed by
                          (row * cols + col). Methods: eat(pos) -> CellType,
                          all_eaten() -> bool, active_positions() -> impl Iterator.
    scoring.rs          — ScoreState struct: score: u32, ghost_chain: u8,
                          extra_life_awarded: bool. Methods: add_dot,
                          add_power_pellet, eat_ghost -> u32, check_extra_life.
    timers.rs           — GameTimers struct: frightened_ticks, scatter_ticks,
                          chase_ticks, cycle_count, death_pause_ticks,
                          extra_life_display_ticks. Methods: tick() -> Vec<TimerEvent>.
  ui/
    mod.rs              — render(frame, &App). MIN_COLS, MIN_ROWS constants.
                          Size check + error widget or normal layout.
    renderer.rs         — maze_widget(&GameState) -> impl Widget. Renders maze
                          cells as colored characters. Ghost and player cells
                          rendered with fg colour via Ratatui Span.
    hud.rs              — hud_widget(&ScoreState, lives, stage, status_msg)
                          -> impl Widget. One-line bar above or below maze.
  input.rs              — map_key(KeyEvent) -> Option<GameAction>. Pure mapping,
                          no state.
```

`timers.rs` is added relative to the system topology proposal. Centralising all
timer state in one struct prevents drift between frightened/scatter/chase
accounting as the tick loop evolves.

`pellets.rs` uses a flat `Vec<bool>` (or `BitVec` if the `bitvec` crate is
added; otherwise `Vec<bool>` is fine) rather than mutating the Maze's cell
array. This keeps `Maze` immutable after parse and makes pellet reset on death
a single `pellets.clone_from(&stage_initial_pellets)` call.

---

## 5. Terminal Guard

```rust
// main.rs
struct TerminalGuard;

impl TerminalGuard {
    fn new() -> anyhow::Result<Self> {
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(
            std::io::stdout(),
            crossterm::terminal::EnterAlternateScreen
        )?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        // Errors swallowed intentionally — we are potentially inside a panic
        // handler and cannot propagate. Best-effort restore is the right policy.
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(
            std::io::stdout(),
            crossterm::terminal::LeaveAlternateScreen
        );
    }
}
```

`main()` creates `TerminalGuard` as the first stack-allocated value. It is
dropped last, after all other resources, on any exit path including panics.

---

## 6. Scatter/Chase State Machine

Implemented in `timers.rs`. Cycle sequence (ticks at 100 ms each):

```
Cycle 1:  Scatter  70 ticks  →  Chase 200 ticks
Cycle 2:  Scatter  70 ticks  →  Chase 200 ticks
Cycle 3+: Chase indefinitely
```

`cycle_count: u8` tracks how many full scatter→chase pairs have completed.
When `cycle_count >= 2`, the timer stays in Chase permanently. Frightened mode
**interrupts** the current phase; when frightened ends, the interrupted phase
resumes with its remaining ticks.

---

## 7. Ghost Rendering Characters

| Ghost     | Normal char | Frightened char |
|-----------|-------------|-----------------|
| Blinky    | `ᗣ` (U+15E3) fallback `B` | `ᗣ` blue fg |
| Pinky     | `ᗣ` pink fg  fallback `P` | `ᗣ` blue fg |
| Inky      | `ᗣ` cyan fg  fallback `I` | `ᗣ` blue fg |
| Clyde     | `ᗣ` yellow fg fallback `C` | `ᗣ` blue fg |
| Player    | `C` yellow fg | — |

The renderer attempts Unicode; if the terminal reports it cannot render the
character (detected by checking terminal capabilities via crossterm), it falls
back to the ASCII fallback. Frightened ghosts blink (alternating blue/white fg)
during the last 3 seconds (30 ticks) of frightened mode.

---

## 8. Maze Layouts

All mazes are 21 columns wide × 21 rows tall. Each uses the cell encoding
defined in §2.2. Ghost house is a 2×2 `G` block at rows 9–10, cols 9–10.
Blinky starts at row 8, col 10 (one tile above house center). Player starts
at row 15, col 10 for all stages unless otherwise noted.

Stages 3 and 7 have tunnel rows: open cells at col 0 and col 20 on the same
row (row 10) connect via wrap-around.

### Stage 1

```
#####################
#o..........#......o#
#.##.#####.#.#####.##
#.#..........#......#
#.#.##.###.###.###..#
#....#...........#..#
#.##.#.#######.#.##.#
#....#....P....#....#
#.##.#.#######.#.##.#
#....#...GG....#....#
#.##.#..GG...#.#.##.#
#....#.........#....#
#.##.#.#######.#.##.#
#....#.........#....#
#.##.#.#######.#.##.#
#o..............o...#
#.##.#####.#####.##.#
#..#..........#..#..#
#.#.##.###.###.##.#.#
#...........#.......#
#####################
```

### Stage 2

```
#####################
#o.#.........#....o.#
#..#.#######.#.####.#
#..............#....#
#.####.#####.####.#.#
#......#...#......#.#
#.####.#.#.####.###.#
#.#....#.#.#....#...#
#.#.####GG####.##.#.#
#.#.#...GG...#.#..#.#
#.#.####.#####.##.#.#
#.#....#.#.#...#....#
#.####.#.#.####.###.#
#......#...#...P..#.#
#.####.#####.####.#.#
#..............#....#
#..#.#######.#.####.#
#o.#.........#.....o#
#...................#.#
####################.#
#####################
```

### Stage 3 (tunnel on row 10)

```
#####################
#o................o.#
#.##.########.###.#.#
#.#..#......#..#..#.#
#.#.##.####.##.##.#.#
#.#..........#....#.#
#.###.######.#.###.##
#....#..........#...#
#.##.##.######.##.#.#
#....#...GG...#.....#
 ..#.#..GG....#.##. #
#....#.........#....#
#.##.##.######.##.#.#
#....#....P.....#...#
#.###.######.#.###.##
#.#..........#....#.#
#.#.##.####.##.##.#.#
#.#..#......#..#..#.#
#.##.########.###.#.#
#o................o.#
#####################
```

### Stage 4

```
#####################
#o.................o#
#.##.#####.#####.##.#
#..#.#.....#.....#..#
##.#.#.###.###.#.#.##
#..#.#.#.....#.#.#..#
#..#.#.#.###.#.#.#..#
#....#.#.#.#.#.#....#
#.##.#.#.#P#.#.#.##.#
#....#...GG..#......#
#.##.#..GG...#....##.#
#....#.......#......#
#.##.#.#.###.#.#.##.#
#....#.#.#.#.#.#....#
#..#.#.#.###.#.#.#..#
#..#.#.#.....#.#.#..#
##.#.#.###.###.#.#.##
#..#.#.....#.....#..#
#.##.#####.#####.##.#
#o.................o#
#####################
```

### Stage 5

```
#####################
#o.#...........#...o#
#..#.#########.#...##
#..#...........#....#
#..####.#####.####..#
#........#.#........#
#.######.#.#.######.#
#......#.....#......#
#.####.#.GG.#.####.#.#
#......#.GG.#.......#
#.####.#....#.####..#
#......#.P...#......#
#.######.#.#.######.#
#........#.#........#
#..####.#####.####..#
#..#...........#....#
#..#.#########.#...##
#o.#...........#...o#
#...................#.#
####################.#
#####################
```

### Stage 6

```
#####################
#o.................o#
##.###.#####.###.###.#
#..#...#...#...#....#
#..#.###.#.###.#.##.#
#..#.#...#.#...#.#..#
#....#.#####.#.#....#
#.##...#...#...#.##.#
#.#.###.#.#.###.#.#.#
#.#.#...GG..#...#.#.#
#.#.#..GG...#...#.#.#
#.#.###.#.#.###.#.#.#
#.##...#.P.#...#.##.#
#....#.#####.#.#....#
#..#.#...#.#...#.#..#
#..#.###.#.###.#.##.#
#..#...#...#...#....#
##.###.#####.###.###.#
#o.................o#
#...................#
#####################
```

### Stage 7 (tunnel on row 10)

```
#####################
#o.#.............#.o#
#..#.###########.#..#
#....#.........#....#
#.##.#.#######.#.##.#
#.#..#.#.....#.#..#.#
#.#..#.#.###.#.#..#.#
#.#....#.#.#.#....#.#
#.#.##.#.#P#.#.##.#.#
#.#.#..#.GG.#.#..#.#
 ..#.#..GG..#.#.. #
#.#.#..#....#.#..#.#
#.#.##.#.###.#.##.#.#
#.#....#.#.#.#....#.#
#.#..#.#.###.#.#..#.#
#.#..#.#.....#.#..#.#
#.##.#.#######.#.##.#
#....#.........#....#
#..#.###########.#..#
#o.#.............#.o#
#####################
```

### Stage 8

```
#####################
#o.................o#
#.#####.#####.#####.#
#.#...#.#...#.#...#.#
#.#.#.#.#.#.#.#.#.#.#
#...#...#.#...#...#.#
###.###.#.#.###.###.#
#...............#...#
#.##.#.#.GG.#.#.##..#
#....#..GG..#.......#
#.##.#......#.#.##..#
#....#..P...#.......#
###.###.#.#.###.###.#
#...#...#.#...#...#.#
#...#.#.#.#.#.#.#...#
#.#...#.#...#.#...#.#
#.#####.#####.#####.#
#...................#
#.#####.#####.#####.#
#o.................o#
#####################
```

### Stage 9

```
#####################
#o...#.........#...o#
#.##.#.#######.#.##.#
#..#...#.....#...#..#
##.#.###.###.###.#.##
#..#.#.......#.#.#..#
#..#.#.#####.#.#.#..#
#....#.#...#.#.#....#
#.##.#.#.GG#.#.#.##.#
#....#...GG..#......#
#.##.#.#.....#.#.##.#
#....#.#.P.#.#......#
#.##.#.#####.#.#.##.#
#....#.......#......#
##.#.###.###.###.#.##
#..#...#.....#...#..#
#..#.#.#######.#.#..#
#.##.#.........#.##.#
#...#...........#...#
#o.................o#
#####################
```

### Stage 10

```
#####################
#o.#.#.........#.#.o#
#..#.#.#######.#.#..#
#....#.#.....#.#....#
#.##.###.###.###.##.#
#.#..........#....#.#
#.#.######.######.#.#
#.#.#......#.....#.#.#
#.#.#.####GG####.#.#.#
#...#.#...GG...#.#..#
#.#.#.####.####.#.#.#
#.#.#..P...#....#.#.#
#.#.######.######.#.#
#.#..........#....#.#
#.##.###.###.###.##.#
#....#.#.....#.#....#
#..#.#.#######.#.#..#
#o.#.#.........#.#.o#
#...................#
#.##################.#
#####################
```

---

## 9. Rendering Layout

The terminal area (minimum 80×30) is split vertically:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  SCORE: 00000   LIVES: ♥ ♥ ♥   STAGE: 1                     [P] Pause      │  row 0 (HUD)
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│                      [ maze 21×21 chars, centered ]                        │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

The HUD occupies 1 row. The maze area takes the remaining rows. The maze is
centered horizontally and vertically within the remaining space using Ratatui's
`Layout::centered_rect` or equivalent padding blocks.

---

## 10. Key Invariants for Implementers

1. `Maze` is immutable after construction. Pellet state lives in `PelletMap`,
   not in the maze grid.
2. `ui::render` (and everything it calls) takes only `&` references. No `mut`.
3. Every public function returning `anyhow::Result` must propagate with `?`.
   No `.unwrap()` without an explanatory `// SAFETY:` comment.
4. `stages::STAGES` is indexed 0–9. Stage number displayed to the player is
   `stage_index + 1`.
5. The ghost chain counter resets on: (a) frightened mode expiry, (b) player
   death, (c) new power pellet eaten (chain resets, not increments).
6. Tunnel wrapping applies to both player and ghosts in stages 3 and 7.
7. `MIN_COLS` and `MIN_ROWS` are checked before any game state is rendered.
