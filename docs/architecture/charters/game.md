---
type: charter
domain: game
feature: none
status: active
date: 2026-06-29
superseded_by: none
superseded_date: none
complexity: simple
complexity_rationale: "Single bounded context with no cross-domain contracts"
---

# Domain Charter: game

## Responsibility boundary
The `game` domain owns all behaviour of the Pac-Man terminal application: game loop, state machine (menu/playing/game-over), maze rendering with ratatui, keyboard input via crossterm, entity movement and collision, ghost AI, scoring, health/lives, level progression (10 levels), portal teleportation, power pellets, and high-score persistence via serde JSON. This domain owns the entire binary; there are no other domains.

## Allowed technology stack

| Layer | Technology |
|-------|-----------|
| Language | Rust (edition 2021) |
| Rendering | ratatui 0.28 |
| Terminal I/O | crossterm 0.28 |
| Serialization | serde 1.x + serde_json 1.x |
| Random numbers | rand 0.8 |
| Build system | Cargo |

No additional crates without architecture review.

## Interfaces this domain must expose
None. This is a standalone binary with no external API surface. All interfaces are internal module boundaries within the `src/` tree.

## Interfaces this domain must consume
None. The application has no network dependencies. The only external interface is the local filesystem for high-score persistence (read/write a single JSON file).

## Non-negotiable constraints

1. **Terminal safety**: Raw mode and alternate screen must be restored on any exit path (normal, error, panic).
2. **Minimum terminal size**: 80x24. Behavior when smaller is a Domain Architect decision.
3. **No panics from recoverable errors**: I/O failures, corrupted save files, and invalid input must be handled gracefully.
4. **Clean exit**: On quit, restore terminal state and exit with code 0. On fatal error, restore terminal state, print error to stderr, exit non-zero.
5. **Keyboard-only input**: No mouse. Arrow keys for movement, Enter/Escape for menus.
6. **Exactly 10 levels**: Each with a unique maze layout and two portals.
7. **Scoring system**: Must include dots, power pellets, ghosts (with escalating sequential-eat bonus), and level-completion bonus. Exact point values are a Domain Architect decision.
8. **Health/lives system**: Start with 3 lives. Lose a life on ghost contact (unless power pellet active). Game over at 0 lives.
9. **Top-10 high-score board**: Persistent across restarts. Atomic writes. Corrupted file treated as absent.
10. **Power pellets**: At least 4 per level. Temporarily make ghosts vulnerable. Ghosts flash before reverting. Duration may decrease across levels.
11. **Portals**: Two per level. Stepping onto a portal tile teleports the player to its pair. Ghost-portal interaction is a Domain Architect decision.
12. **Existing `src/main.rs` must be replaced entirely** (the Greeter/Person/Team placeholder code).

## Open questions for Domain Architect

1. **Real-time or turn-based game loop?** See brainstorm §1. Real-time requires tick-based timing, input buffering, and frame-rate management. Turn-based simplifies everything but changes genre. This cascades into rendering strategy, ghost AI timing, and input handling.
2. **Ghost AI fidelity**: Full arcade replication (Blinky/Pinky/Inky/Clyde personalities, scatter/chase mode cycling, frightened reversal) or a simplified model?
3. **Level data storage**: Inline Rust data structures, external files (TOML/JSON), or procedural generation with seeds?
4. **Ghost-portal interaction**: Can ghosts teleport through portals? If so, intentionally or randomly?
5. **Scoring model**: Exact point values per action? Match arcade (10/dot, 50/pellet, 200/400/800/1600 ghost chain, fruit bonuses) or simplified?
6. **Difficulty progression across 10 levels**: What changes? Ghost speed, ghost count, AI aggressiveness (shorter scatter phases), maze complexity, portal placement, power pellet count, or power pellet duration?
7. **Power pellet duration**: Fixed duration or decreasing per level? If decreasing, arcade curve (~6s down to ~1s) or custom?
8. **Fruits/bonus items**: Include them? If so, spawn timing, point values, and whether they persist per level.
9. **High-score file path**: XDG-compliant (`~/.local/share/pacman-terminal/`), relative to binary, or configurable?
10. **Terminal resize behavior**: Refuse to launch below minimum? Pause with message? Re-render gracefully?
11. **Rendering style**: Unicode glyphs (● for Pac-Man, ♥ for lives, ◆ for pellets) or ASCII-only? Colour palette?
12. **Module layout**: How to partition `src/` (e.g. `main.rs`, `game.rs`, `render.rs`, `input.rs`, `levels.rs`, `score.rs`, `ghosts.rs`, `maze.rs`)? Exact structure is the Domain Architect's call.
13. **Menu system**: Start screen, scoreboard view, pause screen? Layout and navigation details.
