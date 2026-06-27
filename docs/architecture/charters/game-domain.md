---
type: charter
domain: game
feature: none
status: active
date: 2026-06-27
superseded_by: none
superseded_date: none
complexity: simple
complexity_rationale: "Single-domain charter for a self-contained terminal game; no cross-domain interfaces to constrain."
---

# Domain Charter: game

## Domain: game

### Responsibility boundary

The `game` domain owns the complete Pacman terminal game: all game logic (player
movement, ghost AI, collision detection, dot/pellet mechanics, scoring, stage
management), the terminal UI (Ratatui rendering, HUD), input handling, and the
process lifecycle (terminal setup and teardown). It is the only domain in this
system. There are no external services, APIs, or peers it must coordinate with.
Everything the binary does lives inside this domain.

---

### Allowed technology stack

| Category | Permitted |
|----------|-----------|
| Language | Rust (stable toolchain, latest stable at implementation time) |
| TUI framework | Ratatui (latest stable) |
| Terminal backend | Crossterm (Ratatui's default backend; must not be swapped for termion or other backends without a new ADR) |
| Error handling | `anyhow` **or** `color-eyre` — Domain Architect picks one and applies it consistently |
| Data serialisation | Only if needed for stage definitions: `serde` + RON or TOML via `include_str!` (no runtime file I/O) |
| Test utilities | Rust standard test harness (`#[test]`, `#[cfg(test)]`) |
| **Explicitly forbidden** | SDL, OpenGL, Vulkan, Winit, `rodio`, `cpal`, any audio crate, any windowing crate, Tokio (unless the Domain Architect produces a compelling justification — see open question §8 in system topology) |

No new crate may be added to `Cargo.toml` that belongs to a forbidden category.
All other crates require no approval but must be documented in the domain
architecture doc with a one-line rationale.

---

### Interfaces this domain must expose

This domain exposes no external interfaces. The only public surface is the
compiled binary itself, invoked with no required arguments.

Optional CLI flags are at the Domain Architect's discretion (e.g. `--stage N`
for debugging). They must not change the default behavior when absent.

---

### Interfaces this domain must consume

This domain consumes:
- The host terminal via Crossterm (raw mode, event polling, ANSI output)
- The keyboard (arrow keys, WASD, Esc, Q, P, Space — see system topology §6.5)

No network interfaces, file system reads at runtime, or IPC of any kind.

---

### Non-negotiable constraints

1. **Terminal restore on all exit paths.** Raw mode must be restored before
   process exit under all conditions: normal quit, panic, and signal. A `Drop`-
   based terminal guard is mandatory. Violation is a BLOCKING finding in any
   review.

2. **No audio, no external window.** See system topology §6.4. This is a hard
   requirement traceable to the feature specification. No exceptions.

3. **Ten stages, each unique.** The game must have exactly 10 stages. Each stage
   must have a distinct maze layout (a layout may not be reused verbatim across
   stages). Stage identity is enforced in acceptance tests.

4. **Classic Pacman mechanics must be present.** The following mechanics are
   required and non-negotiable:
   - Player navigates a maze and eats dots
   - Eating all dots clears the stage and advances to the next
   - Ghosts chase the player; contact with a non-frightened ghost costs a life
   - Power pellets temporarily make ghosts frightened (vulnerable/edible)
   - Eating a frightened ghost sends it back to the ghost house
   - Player starts with a fixed number of lives (minimum 3)

5. **Keyboard controls: arrows + WASD.** Both must work. Esc or Q must quit.
   P or Space must pause/unpause. These bindings must not be behind a feature
   flag or configuration option.

6. **No `unwrap()` / `expect()` in non-test production code** without an
   explanatory comment justifying why the condition is a logical impossibility.
   Reviewers will treat undocumented panics as BLOCKING findings.

7. **Stateless renderer.** `ui::render` (or its equivalent) must not mutate
   `GameState`. All mutations happen in the tick path.

8. **Minimum terminal size check.** The game must detect when the terminal is
   too small to render the largest maze and display a clear error message
   (e.g. "Terminal too small — please resize"). It must not silently render
   garbage.

---

### Open questions for Domain Architect

The following design decisions are entirely within your authority. Resolve them
in `docs/architecture/game-domain.md` before implementation begins.

1. **Ghost AI algorithm** — Classic per-ghost targeting (Blinky targets current
   player tile; Pinky targets 4 tiles ahead; Inky uses a Blinky-relative
   offset; Clyde flees when close) vs. a simplified unified chase algorithm.
   Either is acceptable. Document your choice and rationale.

2. **Stage layout format** — Hardcoded string arrays in `stages.rs` (zero
   runtime dependencies, simpler) vs. embedded data files via `include_str!`
   with `serde` (easier to edit). Pick one and apply it to all 10 stages.

3. **Timing parameters** — Exact values for:
   - Tick interval (render frame rate)
   - Ghost movement step rate (may differ from render rate)
   - Frightened mode duration
   - Scatter/chase cycle lengths (may increase difficulty across stages)
   - Ghost house exit timer or dot-count threshold

4. **Score values** — Dot worth, power-pellet worth, ghost-chain values during
   frightened mode, bonus items (if any). A simple scoring scheme is fine.

5. **Death sequence** — Brief animation vs. instant respawn. If animation,
   define the visual representation (cell-by-cell wipe, player sprite cycle).

6. **Tunnel wrapping** — Whether any of the 10 mazes include left-right
   tunnels. If yes, `physics.rs` must support modular column arithmetic.

7. **Lives count** — Exact starting life count (must be ≥ 3 per constraint 4).

8. **Terminal size minimum** — Minimum columns and rows required to render the
   largest maze, including the HUD. Define these as named constants.

9. **Ghost count per stage** — The system topology assumes ≤ 4 ghosts (classic
   configuration). You may reduce this for early stages if desired.

10. **Extra-life threshold** — Whether the player earns an extra life at a score
    milestone (classic: 10 000 points). Optional feature; document the decision
    either way.
