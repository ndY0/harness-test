---
type: architecture
domain: global
feature: none
status: active
date: 2026-06-27
superseded_by: none
superseded_date: none
complexity: complex
complexity_rationale: "Full system topology covering crate layout, module structure, game-loop data flows, and all cross-cutting standards for a 10-stage terminal Pacman game."
---

# System Topology — Pacman Terminal Game

## 1. Overview

This system is a single-binary, single-process terminal Pacman game written in
Rust. There are no network interfaces, no databases, no inter-process
communication, and no external services. The runtime environment is a POSIX
terminal capable of raw mode (xterm, alacritty, tmux, etc.).

There is exactly **one domain**: `game`. The domain boundary is the binary
itself. All design decisions below that boundary are delegated to the Game
Domain Architect via the charter at
`docs/architecture/charters/game-domain.md`.

---

## 2. Bounded Context Map

| Domain | Responsibility | Owner |
|--------|---------------|-------|
| `game` | All game logic, rendering, input handling, stage management, and terminal lifecycle | Game Domain Architect |

No inter-domain interfaces exist. There are no cross-domain contracts to
version or review.

---

## 3. Crate Layout

The project is a **single Cargo crate** (no workspace). A workspace would add
coordination overhead without benefit at this scale.

```
Cargo.toml
src/
  main.rs          — entry point; wires terminal setup, event loop, and teardown
  app.rs           — App struct; top-level state machine (menu, playing, game-over)
  game/
    mod.rs         — re-exports; GameState; core value types (Direction, Cell, etc.)
    maze.rs        — Maze type; 2-D grid of Cell; wall/path/ghost-house queries
    stages.rs      — 10 stage definitions; stage loader; maps stage index to Maze
    ghost.rs       — Ghost entity; AI mode enum (Chase, Scatter, Frightened)
    player.rs      — Player entity; position, lives, buffered-input direction
    physics.rs     — Movement resolution; collision detection; tunnelling support
    pellets.rs     — Dot and power-pellet placement; eaten-state tracking
    scoring.rs     — Score accumulation; ghost-chain multiplier during Frightened
  ui/
    mod.rs         — Ratatui UI entry point; layout split (game area + HUD)
    renderer.rs    — Renders GameState into Ratatui widgets each frame
    hud.rs         — Score, lives remaining, stage number, and status messages
  input.rs         — Crossterm KeyEvent → GameAction mapping (arrows + WASD)
```

The Domain Architect may add, merge, or rename modules within `src/` as long
as the public boundary of `app.rs` (the App struct and its `tick` / `handle_input`
methods) remains stable from `main.rs`'s perspective.

---

## 4. Key Data Flows

### 4.1 Game Loop (primary flow)

```
┌──────────────────────────────────────────────┐
│                  main.rs                     │
│                                              │
│  setup terminal raw mode                     │
│  load stage 1 → App::new(stage)              │
│                                              │
│  loop {                                      │
│    terminal.draw(|f| ui::render(f, &app)) ◄──┼── frame render (every iteration)
│                                              │
│    if crossterm::event::poll(tick_rate) {    │
│      match event::read() {                   │
│        Key(k) → app.handle_input(k)  ◄───────┼── keyboard input
│        Resize → (handled by Ratatui)         │
│      }                                       │
│    }                                         │
│                                              │
│    app.tick()  ◄─────────────────────────────┼── time step: ghosts, timers, state
│                                              │
│    if app.should_quit { break }              │
│  }                                           │
│                                              │
│  restore terminal                            │
└──────────────────────────────────────────────┘
```

The tick rate is a fixed duration (target: ~16 ms / 60 fps for rendering;
ghost movement is gated by an internal step counter so effective ghost speed
is independent of render rate). The exact tick rate is an open question
delegated to the Domain Architect.

### 4.2 Input Flow

```
KeyEvent (crossterm)
  → input::map_key(event) → Option<GameAction>
  → app.handle_input(action)
      → player.buffer_direction(dir)   [movement actions]
      → app.request_quit()             [Esc / Q]
      → app.pause_toggle()             [P / Space]
```

Input is decoupled from physics: `handle_input` only buffers a requested
direction. The direction is consumed by `physics.rs` during `app.tick()` so
that the player never moves diagonally even if keys arrive faster than ticks.

### 4.3 Tick Flow

```
app.tick()
  → if paused: return
  → player.apply_buffered_move(&maze)     (physics.rs: wall check)
  → for each ghost: ghost.step(&maze, &player_pos, mode)
  → pellets.check_eaten(player_pos)
      → if dot:          scoring.add_dot()
      → if power_pellet: scoring.add_power_pellet(); ghosts.set_frightened(timer)
  → collision_check(player_pos, ghost_positions)
      → if ghost frightened: scoring.eat_ghost(chain); ghost.send_home()
      → if ghost not frightened: player.lose_life(); stage.reset_positions()
  → if pellets.all_eaten(): app.advance_stage()
  → if player.lives == 0:  app.game_over()
  → ghosts.tick_frightened_timer()
  → ghosts.tick_scatter_chase_cycle()
```

### 4.4 Stage Transition Flow

```
app.advance_stage()
  → stage_index += 1
  → if stage_index > 10: app.victory()
  → else: load stages::STAGES[stage_index]
           reset player position
           reset ghost positions
           preserve score and lives
```

---

## 5. Infrastructure Topology

| Concern | Decision |
|---------|---------|
| Runtime | Single OS process, single thread (no Tokio, no Rayon required) |
| Terminal backend | Crossterm (cross-platform raw mode, event polling) |
| UI framework | Ratatui 0.x (latest stable at implementation time) |
| Build system | Cargo (standard) |
| Distribution | Single statically-linked binary (preferred) or dynamic with glibc |
| Persistence | None — no save files, no high-score storage |
| Configuration | None at launch — all parameters are compile-time constants or hardcoded defaults |

---

## 6. Cross-Cutting Concerns

### 6.1 Terminal Lifecycle (CRITICAL)

Terminal raw mode **must** be restored on all exit paths:
- Normal exit (player quits)
- Panic
- Signal (SIGTERM, SIGINT where supported)

The Domain Architect must implement a terminal guard struct using the `Drop`
trait that unconditionally restores the terminal to cooked mode. Failure to do
so leaves the user's terminal in an unusable state.

### 6.2 Error Handling

- Use `anyhow::Result` or `color-eyre::Result` (Domain Architect's choice)
  throughout the call stack for fallible operations.
- No `unwrap()` or `expect()` in non-test code except where a condition is
  a genuine logical impossibility (must be documented with a comment).
- Terminal-restore errors during panic must be silently swallowed (we cannot
  propagate errors inside a panic handler).

### 6.3 Rendering Correctness

- All rendering is **stateless from the renderer's perspective**: `ui::render`
  takes `&GameState` and produces terminal output. It must not mutate state.
- Frame rendering must complete within the tick budget (~16 ms). No I/O, no
  allocation-heavy operations inside the render path.

### 6.4 No Audio, No External Windows

The game must not link any audio library, SDL, OpenGL, Vulkan, or windowing
system. Terminal-only. This is a hard constraint, not a preference.

### 6.5 Input Compatibility

The game must accept both arrow keys and WASD for movement. Esc or Q must quit
at any time. P or Space must pause/unpause. No other input bindings are
mandated at this tier; the Domain Architect may add UI-navigation keys as
needed.

---

## 7. Open Architecture Questions

The following decisions are intentionally left to the Game Domain Architect.
They must be resolved in the domain architecture document
(`docs/architecture/game-domain.md`) before implementation begins:

1. **Ghost AI fidelity** — Classic Pac-Man ghost personalities (Blinky/Pinky/
   Inky/Clyde targeting algorithms) vs. a simplified unified AI. Either is
   acceptable; the choice must be documented.

2. **Stage layout representation** — Hardcoded ASCII-art string arrays in
   `stages.rs` vs. embedded RON/TOML data files loaded with `include_str!`.
   Trade-off: strings are simpler; data files are easier to edit without
   recompiling.

3. **Timing parameters** — Tick rate, ghost step interval, frightened duration,
   scatter/chase cycle lengths. These determine game feel and difficulty
   progression across 10 stages.

4. **Ghost house exit rule** — The condition under which waiting ghosts leave
   the ghost house (dot count, timer, or hybrid). This is a game-feel decision.

5. **Score values** — Dot worth, power-pellet worth, ghost-chain values
   (200/400/800/1600 or simplified flat value), bonus items if any.

6. **Death animation** — Whether a brief death animation is shown before
   respawning, or if respawn is instant. If animation: cell-by-cell wipe vs.
   sprite-frame cycle.

7. **Tunnel wrapping** — Whether any maze has left-right tunnel wrapping (as in
   the original). If yes, physics.rs must support modular position arithmetic.

8. **Terminal minimum size** — Minimum terminal columns and rows required. The
   renderer should check on startup and on resize, and show an error widget
   if the terminal is too small.

---

## 8. References

- ADR-001: `docs/adr/ADR-001-tech-stack.md`
- Game Domain Charter: `docs/architecture/charters/game-domain.md`
