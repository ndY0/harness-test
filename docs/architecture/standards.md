---
type: architecture
domain: global
feature: none
status: active
date: 2026-06-29
superseded_by: none
superseded_date: none
complexity: simple
complexity_rationale: "Cross-cutting standards for a single-domain project"
---

# Architecture Standards

## ADR-001: Error handling

**Decision**: The application must never panic from a recoverable error. Use `Result` for all fallible operations. On unrecoverable errors (terminal I/O failure, corrupted save file), the application cleanly restores the terminal and prints an error message to stderr before exiting. Panics are acceptable only for invariant violations that indicate a programming bug.

**Rationale**: This is a terminal application. A panic leaves the user with a broken terminal (raw mode not restored). Clean exit with terminal restoration is mandatory.

## ADR-002: Terminal safety

**Decision**: Use crossterm's `terminal::enable_raw_mode()` and `terminal::disable_raw_mode()` inside a drop guard or RAII wrapper. On any exit path (normal, error, panic via `std::panic::set_hook`), raw mode must be disabled and the alternate screen must be restored.

**Rationale**: A terminal stuck in raw mode after the application exits is a degraded user experience. The cleanup must be unconditional.

## ADR-003: Code structure

**Decision**: The `src/` tree shall be organised as a Cargo project with a single binary target. Modules shall separate concerns: `render`, `input`, `game` (state/logic), `levels`, `score`. Exact module layout is the Domain Architect's decision.

**Rationale**: Separation of rendering from game logic from persistence simplifies testing and future extension.

## ADR-004: Testing

**Decision**: Game logic (scoring, ghost AI, level progression, collision detection) must be unit-testable without a terminal. Rendering and input code may require integration tests. Use `cargo test` as the test runner.

**Rationale**: Terminal-dependent tests are fragile in CI. Pure logic tests are fast and reliable.

## ADR-005: File format

**Decision**: High scores persist as a JSON file. The file must be written atomically (write to temp file, rename). Corrupted files are treated as if absent (scores reset).

**Rationale**: JSON via serde is already in the dependency tree. Atomic writes prevent truncation on crash. A corrupted file should not block the game from launching.

## ADR-006: Rendering target

**Decision**: The game targets a standard 80x24 terminal as the minimum viable dimensions. The Domain Architect must define the exact minimum size, the behavior when below that threshold, and the resize strategy.

**Rationale**: 80x24 is the de facto terminal standard. A game that doesn't fit is unplayable.

## ADR-007: Input model

**Decision**: Keyboard-only input via crossterm `Event::Key`. Arrow keys for movement. Additional keys for menu navigation (Enter, Escape). No mouse support required.

**Rationale**: Pac-Man is fundamentally a keyboard/joystick game. Mouse support adds complexity with no gameplay benefit.

## Open questions

None. All deferred decisions are listed in the game charter.
