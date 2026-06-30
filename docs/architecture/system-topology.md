---
type: architecture
domain: global
feature: none
status: active
date: 2026-06-29
superseded_by: none
superseded_date: none
complexity: simple
complexity_rationale: "Single-domain project with no cross-domain interfaces"
---

# System Topology: Pac-Man Terminal Game

## Bounded context map

| Domain | Responsibility | Owner |
|--------|---------------|-------|
| `game` | All game logic, rendering, input, persistence, scoring, level progression | Domain Architect (game) |

This is a single-domain system. There are no cross-domain interfaces.

## Deployment topology

- **Runtime**: Single binary (`pacman-terminal`), compiled from `src/main.rs`
- **Execution model**: Direct terminal process. No server, no daemon, no background service.
- **Platform targets**: Linux, macOS (primary). Windows support is not guaranteed in v1 but nothing in the stack precludes it.
- **Persistence**: Single file on local filesystem (high-score board). Exact path TBD by Domain Architect (XDG or simple relative path).
- **No network boundary**: All I/O is local terminal I/O and local filesystem I/O.

## Inter-domain communication

None. Single domain. All modules within the `game` domain communicate via direct Rust function calls and shared types.

## External dependencies (fixed)

| Dependency | Version | Purpose |
|-----------|---------|---------|
| `ratatui` | 0.28 | Terminal UI rendering (frame-based, widget model) |
| `crossterm` | 0.28 | Terminal raw mode, input events, cursor control |
| `serde` + `serde_json` | 1.x | High-score serialization/deserialization |
| `rand` | 0.8 | Random number generation (ghost AI, procedural elements if used) |

No additional dependencies may be added without an architecture review.

## Cross-cutting concerns

| Concern | Owner | Decision |
|---------|-------|----------|
| Error handling | Domain Architect | No panics in game loop; graceful degradation on I/O errors |
| Input handling | Domain Architect | crossterm event polling; keyboard-only |
| Rendering strategy | Domain Architect | ratatui tick-based frame loop |
| Data persistence | Domain Architect | serde JSON, local file, atomic writes |
| Game clock / timing | Domain Architect | Tick rate and delta-time approach |

## Infrastructure

- No containerization, no orchestration, no cloud resources.
- Build: `cargo build --release` produces the distribution artifact.
- Testing: `cargo test` with Rust's built-in test harness.

## Open architecture questions

1. **Real-time vs turn-based game loop**: The brainstorm lists this as the single most consequential decision. This architecture does not force either approach — it is the Domain Architect's responsibility to choose and justify.
2. **High-score file location**: XDG (`~/.local/share/pacman-terminal/`) vs simple relative path vs configurable. Domain Architect decides.
3. **Terminal size handling**: Minimum size threshold, resize behavior (pause/refuse/re-render). Domain Architect decides.
4. **Level data storage format**: Inline Rust, external files (TOML/JSON), or procedural generation. Domain Architect decides.
5. **Ghost-portal interaction**: Can ghosts use portals? Domain Architect decides.
