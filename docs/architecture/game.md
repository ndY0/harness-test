---
type: architecture
domain: game
feature: pacman-animation
status: active
date: 2026-06-30
superseded_by: none
superseded_date: none
complexity: simple
complexity_rationale: "Single-domain feature; animation state derived from existing tick counter and direction; no new external interfaces"
---

## Domain: game

### Component map

| Component | Responsibility |
|-----------|---------------|
| `App` | Top-level state machine (Menu, Playing, Paused, GameOver). Owns the ratatui tick-based game loop. Delegates to sub-components per frame. |
| `Maze` | Grid-based maze: wall/path topology, dot/pellet/portal positions. Tracks dot consumption per level via a consumed bitmap. |
| `Entities` | Pac-Man position, direction, movement state. Ghost positions, directions, AI modes. Collision detection between entities and maze tiles. |
| `GhostAI` | Per-ghost target selection using distinct targeting personalities (Blinky/Pinky/Inky/Clyde). Frightened-mode reversal. |
| `Input` | crossterm event polling. Maps key events to actions (movement, menu nav, pause/quit). Buffered input for continuous movement. |
| `Render` | ratatui Terminal frame drawing: maze, entities, HUD (score, lives, level), menus, game-over screen. Unicode glyphs with color. **Pac-Man animation**: selects directional mouth-open/mouth-closed glyph based on tick parity and Pac-Man direction. |
| `Scoring` | Point tallying: dots, power pellets, sequential ghost-eat multiplier, fruits, level-completion bonus. Owns score and lives count. |
| `Scoreboard` | Top-10 high-score persistence. serde JSON, atomic writes. Corrupted file treated absent. XDG-compliant path. |
| `Levels` | Owns 10 level definitions: maze layout, portal pairs, ghost/Pac-Man spawns, difficulty parameters. Provides level-transition logic. |

### Data model

Key entities:
- **MazeGrid**: 2D array of tiles (Wall, Dot, PowerPellet, Portal, Empty). Immutable except dot consumption tracked via bitmap.
- **PacMan**: grid position, direction enum, state (alive/dying/respawning). No animation fields — animation is derived at render time from direction and tick parity.
- **Ghost**: grid position, direction, personality type, mode (chase/scatter/frightened/eaten), color.
- **LevelConfig**: maze layout, spawn points, portal pairs, difficulty params (ghost speed factor, fright duration).
- **ScoreEntry**: player name + score. JSON-serialized for persistence.
- **GameState**: current level index, lives, score, power-pellet timer, ghost-eat chain counter, **tick_count** (u64, wrapping increment each tick — provides animation frame source).
- **RenderContext**: snapshot passed to Render each frame. Includes `tick_count` so Render can compute animation frame from tick parity without coupling to App state.

Animation state placement: **No animation state on PacMan**. The `Render` component computes the animation frame at draw time from two inputs: (a) Pac-Man's current `dir`, and (b) `tick_count % 2` parity. This keeps PacMan a pure data struct (`Clone + Copy`), unchanged from its current definition.

Persistence: High scores only (single JSON file). All game state is ephemeral (in-memory per session).

### Internal communication

All components communicate via direct Rust function calls, orchestrated by `App`.

Per-tick data flow:
1. `Input` → action → `App`
2. `App` → updates `Entities` positions, queries `Maze` for collisions and tile consumption
3. `App` → notifies `Scoring` of events (dot eaten, ghost eaten, fruit collected)
4. `App` → queries `GhostAI` for updated target vectors
5. `App` → checks portal teleportation (player only; ghosts do not use portals)
6. `App` → evaluates win/lose conditions, transitions state if needed
7. `App` → passes world snapshot to `Render` for frame draw (including `tick_count`)

Animation flow (within step 7):
- `Render` receives `PacMan.dir` and `tick_count` via `RenderContext`
- `Render` selects glyph from a direction-indexed lookup table using `(dir, tick_count % 2)` as key
- Direction → glyph mapping: 4 cardinal directions + a neutral/static variant for `Direction::None`
- Mouth-open frame on even ticks, mouth-closed on odd ticks (or vice versa; parity choice is cosmetic)

### External interfaces

None. Standalone terminal binary. No network APIs, no IPC, no shared contracts. The only external interaction is filesystem I/O for high-score persistence.

### Technology decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Game loop | Real-time, tick-based via ratatui event loop | Preserves arcade feel. ratatui's frame-rate model naturally supports this. 100–200ms tick interval provides responsive controls within terminal constraints. |
| Ghost AI | Simplified: distinct targeting personalities, chase/frightened modes, no scatter/chase cycling | Arcade-accurate mode cycling is complex to tune and hard to perceive in terminal rendering. Distinct personalities give recognizable behavior without cycling overhead. |
| Level storage | Inline Rust struct definitions | Compile-time validation, zero runtime I/O, no format parsing bugs. 10 mazes are small enough for inline data. |
| Ghost-portal | Ghosts do not use portals | Portals are a player tactical escape. Ghost teleportation would make portal placement a liability. |
| Scoring model | Arcade-matched values | Well-documented values; satisfies the Score Chaser persona. |
| Scoreboard persistence | XDG-compliant path, atomic write via temp+rename | Standard Unix convention. Atomic writes prevent corruption. |
| Terminal resize | Refuse launch below 80x24. Pause with message if resized during play. | 80x24 is the de facto standard. |
| Fruits | Included — spawn twice per level at dot-count thresholds | Adds scoring depth and risk/reward. |
| Power pellet duration | Decreasing per level (arcade-like curve) | Provides difficulty progression. |
| Module layout | `main.rs`, `app.rs`, `maze.rs`, `entities.rs`, `input.rs`, `render.rs`, `scoring.rs`, `levels.rs`, `scoreboard.rs` | Separation of concerns. Logic modules unit-testable without terminal. |
| **Pac-Man animation — state placement** | **Derived at render time from `tick_count` and `PacMan.dir`; no animation fields on PacMan struct** | Keeps PacMan a pure data struct (`Clone + Copy`). Render already owns presentation concerns. Tick-driven frame changes align with existing real-time loop. |
| **Pac-Man animation — frame cycling** | **2-frame parity: mouth-open on even ticks, mouth-closed on odd ticks** | Sufficient for single-cell terminal rendering. Multi-frame animation would add complexity with imperceptible benefit at 6–10 fps terminal refresh. |
| **Pac-Man animation — glyph selection** | **Directional Unicode glyph lookup: 4 cardinal directions × 2 mouth states, plus neutral glyph for `Direction::None`. Fallback to simpler glyph set for terminals lacking wide Unicode support.** | Directional mouth wedge communicates movement intent (classic Pac-Man look). Unicode Geometric Shapes block provides adequate wedge/circle variants. Fallback prevents rendering breakage on limited terminals. |
| **Pac-Man animation — death interaction** | **Fixed glyph (closed circle) during `Dying` state; normal animation during `Respawning`** | Visually distinct death indicator without complex death animation. Respawning is brief (1 tick). |
| **Pac-Man animation — power-pellet interaction** | **No change to Pac-Man glyph during power-pellet active state** | Arcade convention: Pac-Man appearance is unchanged by power pellets. Only ghosts change appearance. |

### Open questions

1. **Menu-screen rendering style**: Simple text list or elaborate TUI with borders/styling?
2. **Player name entry for high scores**: Text input in raw mode requires special handling via crossterm key events; widget design is unresolved.
3. **Ghost eaten-eye return animation**: Animate eyes returning to lair in real time, or instant respawn? Terminal animation fidelity constrains options.
4. **Level-completion bonus formula**: Flat bonus or scaled by remaining lives? Charter requires a bonus but not how it is computed.
