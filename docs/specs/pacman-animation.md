---
type: spec
domain: game
feature: pacman-animation
status: active
date: 2026-06-30
superseded_by: none
superseded_date: none
complexity: simple
complexity_rationale: "3 files changed (app.rs, render.rs, entities.rs — RenderContext struct), no concurrent paths, estimated <80 new lines"
---

# Spec: Pac-Man Animation

## Summary
Replace the static filled-circle Pac-Man glyph with a directional, animated glyph that cycles between mouth-open and mouth-closed frames each tick. The glyph faces Pac-Man's current movement direction. Animation frame selection is a pure function of `(direction, tick_count % 2)`, computed in the render layer at draw time. No animation state is added to the `PacMan` struct.

## Acceptance criteria

1. **Directional glyph selection** — Given Pac-Man facing `Up`, `Down`, `Left`, or `Right`, when `render_game` draws that cell, then the glyph displayed is direction-specific (wedge-opening points in the movement direction) and differs from the glyph for any other direction.

2. **Mouth open/closed cycling** — Given consecutive game ticks with no direction change, when Pac-Man is rendered on tick N (even) vs tick N+1 (odd), then the two glyphs differ (mouth-open frame vs mouth-closed frame) and the pattern repeats every 2 ticks.

3. **Direction::None uses neutral glyph** — Given Pac-Man's `dir` is `Direction::None`, when rendered, then a neutral glyph (wedge-less or symmetric) is displayed, and it does not change with `tick_count` parity.

4. **Dying state overrides animation** — Given Pac-Man's `state` is `Dying(_)`, when rendered, then the glyph is a fixed filled circle (●, U+25CF) regardless of `dir` or `tick_count`.

5. **Animation is tick-driven** — Given `tick_count` in `RenderContext`, when `render_game` selects a glyph for Pac-Man, then the glyph value depends only on `(pacman.dir, pacman.state, tick_count % 2)` and not on any mutable state within the render function.

6. **No breaking to existing rendering** — Given the base game is fully functional (maze, ghosts, scoring, HUD, menus), when the animation changes are applied, then all non-Pac-Man rendering (walls, dots, power pellets, ghosts, HUD, menus, fruit) is unchanged from the previous release.

7. **Unit-testable glyph logic** — Given the glyph-lookup function, when called with each `(Direction variant, parity 0 or 1, PacManState variant)`, then all 22 combinations `(5 directions × 2 parities × 2 states + 2 Dying overrides)` return non-empty `&'static str` values, and no two distinct `(Up/Down/Left/Right, parity)` pairs return the same glyph for the same state.

## API contracts

### Modified: `RenderContext` (in `src/render.rs` or `src/entities.rs`)

Add field:

```rust
pub tick_count: u64,
```

### Modified: `render_game` call site (in `src/app.rs`)

All `RenderContext` constructors (Playing, Paused) must include `tick_count: state.tick_count`.

### New: `fn pacman_glyph(dir: Direction, state: PacManState, parity: u8) -> &'static str` (in `src/render.rs`, private or `pub(crate)`)

Returns the glyph string for Pac-Man.

| `dir`      | `state == Alive, parity 0 (even)` | `state == Alive, parity 1 (odd)` |
|------------|-----------------------------------|----------------------------------|
| `Up`       | ◒ (U+25D2)                        | ◓ (U+25D3)                       |
| `Down`     | ◓                                  | ◒                                |
| `Left`     | ◐ (U+25D0)                        | ◑ (U+25D1)                       |
| `Right`    | ◑                                  | ◐                                |
| `None`     | ● (U+25CF)                        | ●                                |
| `Dying(_)` | ●                                  | ●                                |

**Fallback**: If the terminal or ratatui rendering path cannot render these Unicode Geometric Shapes characters, the implementation must degrade to a static `●` (U+25CF) or `C` for all states. The fallback selection mechanism is an implementation detail (e.g., check a compile-time feature flag or environment variable `PACMAN_UNICODE=0`), but must not panic or render garbage.

### Modified: `render_game` Pac-Man rendering (in `src/render.rs`)

Replace the existing static `"●"` for Pac-Man with a call to `pacman_glyph` using `ctx.pacman.dir`, `ctx.pacman.state`, and `(ctx.tick_count % 2) as u8`.

Also render Pac-Man when `PacManState::Dying(_)` (currently only `Alive` state is rendered), using the fixed closed-circle from the glyph function.

## Data

No new persisted data. No new fields on `PacMan`, `Ghost`, `MazeGrid`, `Score`, or any persisted struct.

- `RenderContext` gains one field: `tick_count: u64` (already exists on `GameState`, now threaded through).
- `GameState.tick_count` is already present; no change to its definition or increment logic.

## Edge cases

- **Pac-Man not moving (`Direction::None`)**: Renders neutral circle; no frame alternation needed.
- **Pac-Man stuck against a wall**: Direction remains set even if position doesn't change; animation continues cycling normally.
- **`tick_count` wrapping**: `u64` wraps at 2^64; `tick_count % 2` continues to produce correct parity across wrap.
- **Dying state**: Overrides all directional animation; always shows closed circle regardless of parity.
- **Respawning state**: Brief (1 tick). If `Respawning` appears in render — use Alive glyph logic since the spec doesn't define a special glyph for it, or match the Alive branch.
- **Power pellet active**: No glyph change per architecture decision. Animation continues with normal mouth cycling.
- **Terminal without Unicode support**: Fallback to `●` for all states. Must not crash, render tofu, or produce misaligned output.
- **`RenderContext` constructed without `tick_count`**: Compile-time error prevents this (Rust struct literal requires all fields). No runtime edge case.

## Out of scope

- Multi-frame animation beyond 2 frames (architecture decision: 2-frame parity sufficient for terminal)
- Ghost animation or ghost mouth cycling (covered by F-003 Ghost Visuals)
- Death animation sequence (architecture decision: fixed glyph during Dying)
- Power-pellet visual changes to Pac-Man (architecture decision: no change)
- Animation speed or tick-rate changes (tick rate is 150ms, owned by app loop, unchanged)
- Unicode fallback mechanism as a user-facing setting (deferred; ADR not required)
