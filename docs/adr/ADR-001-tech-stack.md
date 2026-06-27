---
type: adr
domain: global
feature: pacman-terminal-game
status: active
date: 2026-06-27
superseded_by: none
superseded_date: none
complexity: simple
complexity_rationale: "Records technology choices; no design trade-off analysis spans multiple domains."
---

# ADR-001 — Technology Stack for Pacman Terminal Game

## Status

Accepted

## Context

Feature F-001 requires a Pacman terminal game with the following hard
constraints from the feature specification:

- Written in Rust
- Uses Ratatui as the TUI library
- Terminal UI only (no external window, no SDL, no OpenGL, no audio)
- Keyboard-controlled (arrow keys or WASD)
- Ten stages with unique maze layouts

This ADR captures why these choices were made and the consequences that flow
from them. It also records the game-loop architecture decision, which is the
one meaningful architectural choice the Master Architect makes above the domain
level.

---

## Decisions

### D1 — Language: Rust (stable toolchain)

**Decision:** Use Rust with the latest stable toolchain at implementation time.

**Rationale:**
- Mandated by the feature specification.
- Rust's ownership model eliminates entire classes of bugs (use-after-free,
  data races) that are common in interactive game loops.
- Rust's zero-cost abstractions and lack of GC pauses make it well-suited for
  a consistent-frame-rate game loop where latency spikes would cause visible
  stuttering.
- The Rust ecosystem for TUI development (Ratatui + Crossterm) is mature and
  actively maintained.

**Consequences:**
- No other language may be used for any part of the build (no C/C++ via FFI
  for game logic; C may only appear transitively through Crossterm's system
  calls, which is acceptable).
- Developers must use `cargo` for build and test. No Makefile-only workflows.

---

### D2 — TUI Framework: Ratatui

**Decision:** Use Ratatui as the sole terminal UI library.

**Rationale:**
- Mandated by the feature specification.
- Ratatui's immediate-mode rendering model (redraw the full UI each frame from
  a `Frame` reference) maps cleanly onto a game-loop architecture: no retained
  widget state to synchronise with game state.
- Ratatui's `Canvas` widget supports arbitrary character-cell drawing, which
  is required for maze rendering.
- Ratatui is the most actively maintained Rust TUI library as of 2026.

**Consequences:**
- Ratatui's default backend is Crossterm. The Domain Architect must not swap
  this for termion or another backend without a new ADR, because backend
  swaps affect raw-mode behaviour and event handling in non-obvious ways.
- The renderer must remain stateless (see system topology §6.3). Ratatui's
  `Frame` is borrowed for the duration of `terminal.draw()`; no mutable
  game-state access is possible during rendering, which enforces the
  stateless-renderer constraint mechanically.

---

### D3 — Terminal Backend: Crossterm

**Decision:** Use Crossterm as the terminal backend for Ratatui.

**Rationale:**
- Crossterm is Ratatui's default and most widely tested backend.
- It is cross-platform (Linux, macOS, Windows) without requiring a platform-
  specific terminal library.
- It provides both raw-mode control and the event-polling (`event::poll`) API
  that the game loop requires.

**Consequences:**
- Terminal setup and teardown must use Crossterm's `enable_raw_mode` /
  `disable_raw_mode` APIs.
- A `Drop`-based guard wrapping these calls is required (see system topology
  §6.1 and domain charter constraint 1).
- `event::poll(Duration)` is the prescribed way to implement the tick
  deadline; busy-polling with `event::read()` is not acceptable.

---

### D4 — Game Loop Architecture: Single-threaded tick-and-render loop

**Decision:** Implement the game loop as a single-threaded loop that:
1. Renders the current state via `terminal.draw()`
2. Polls for input with a bounded timeout (the tick duration)
3. Applies any input to game state
4. Advances game state by one tick (ghost movement, timer decrements,
   collision checks)
5. Repeats

**Rationale:**
- A single-threaded loop is sufficient for a game of this scale and eliminates
  synchronisation complexity.
- Ratatui's `terminal.draw()` borrows the terminal mutably for the duration of
  the call; multi-threaded rendering would require a `Mutex<Terminal>` with no
  benefit.
- Separating physics ticks from render frames (by using a step counter inside
  the game state rather than relying on wall-clock timing for game speed) makes
  the game behaviour reproducible and testable without a real timer.
- This architecture is the established idiomatic pattern for Ratatui
  applications (see Ratatui's own "Counter App" tutorial and community
  templates).

**Consequences:**
- The Domain Architect must ensure that `app.tick()` completes well within the
  tick budget. Expensive operations (e.g. O(n²) ghost pathfinding where n is
  maze size) must be avoided or bounded.
- The game must not use `std::thread::sleep` inside the tick body. The timing
  is controlled entirely by the `event::poll` timeout.
- Testing game logic does not require a running terminal. `app.tick()` and
  `app.handle_input()` operate on plain data structures and can be unit-tested
  without Ratatui or Crossterm.

---

### D5 — Stage/Level Structure: Fixed 10-stage progression

**Decision:** The game has exactly 10 stages, loaded in order. Each stage is a
distinct maze layout. Stage progression is linear (stage N → stage N+1 on
clearing all dots). There is no random stage selection and no stage repetition
within a single run.

**Rationale:**
- Mandated by the feature specification ("10 stages, each with a unique
  maze/level layout").
- A fixed linear progression is the simplest correct implementation.
- Unique layouts per stage prevent the game from feeling repetitive and reward
  the player's memory of each maze.

**Consequences:**
- Stage layouts must be defined at compile time (no runtime file I/O).
- Each stage must be verifiably distinct — at minimum, the set of wall cells
  must differ between any two stages. This is an acceptance criterion.
- The Domain Architect must design all 10 maze layouts before implementation.
  Placeholder "same maze repeated" stages are not acceptable.

---

### D6 — No Persistence, No Configuration

**Decision:** The game has no save files, no high-score persistence, no
configuration files, and no command-line arguments (beyond optional debug
flags).

**Rationale:**
- The feature specification does not require persistence or configuration.
- Adding persistence would require a decision on storage location
  (`~/.local/share/`, XDG dirs, etc.) that is not in scope.
- Keeping the binary stateless eliminates an entire class of bugs and
  simplifies the domain architecture.

**Consequences:**
- Score is lost when the process exits. This is expected and by design.
- If the Domain Architect later determines that a `--stage N` debug flag is
  useful, it may be added without a new ADR as it does not affect default
  behaviour.

---

## Alternatives Considered

| Alternative | Rejected because |
|-------------|-----------------|
| Python + curses | Feature specification mandates Rust |
| Rust + ncurses (via `ncurses-rs`) | Ratatui mandated by spec; ncurses has a C FFI dependency and is harder to make cross-platform |
| Rust + `termion` backend | Termion does not support Windows; Crossterm is the more robust default |
| Async game loop (Tokio) | No I/O-bound concurrency needed; Tokio adds complexity and compile-time cost with no benefit |
| Multi-threaded render + logic | Ratatui's `terminal.draw()` is inherently single-thread; added complexity with no gain |
| Runtime-loaded stage files | Adds file I/O complexity; `include_str!` achieves the same result with compile-time guarantees |

---

## References

- System topology: `docs/architecture/system-topology.md`
- Game domain charter: `docs/architecture/charters/game-domain.md`
- Ratatui documentation: https://ratatui.rs
- Crossterm crate: https://crates.io/crates/crossterm
