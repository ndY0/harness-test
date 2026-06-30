---
feature: pacman-terminal-game
subtask: B
title: "Scoring, lives, and persistent top-10 scoreboard"
write_set: ["src/scoring.rs", "src/scoreboard.rs"]
depends_on: ["A"]
complexity: simple
planner_depth: 1
---

## Context

Sub-task B implements scoring logic and the persistent high-score board. It depends on sub-task A for `Direction` and `TileEffect` types only.

## Inputs from spec

- Scoring model: dots (10), power pellets (50), ghost-eat chain (200→400→800→1600), fruit (per-level values), level-completion bonus (100 × level)
- `ScoreEvent` enum: `Dot`, `PowerPellet`, `Ghost(u8)`, `Fruit(u32)`, `LevelComplete(u8)`
- `ScoreEntry`: `{ name: String, score: u32 }`, max 10 entries, sorted by score descending
- Persistence path: `~/.local/share/pacman-terminal/scores.json`
- Atomic write: write to temp file in same directory, then rename
- Directory created with `0o700` permissions if missing
- On startup: missing or unparseable file → empty scoreboard
- If directory not writable: game launches, warning to stderr, no persistence
- Top-10 ranking: tie broken by retaining older entry; lowest dropped when full
- Name input: exactly 3 uppercase alphanumeric characters (A-Z, 0-9)
- Score is `u32`
- Lives: start at 3, cannot exceed 99
- `ghost_eat_chain`: 0–4, reset to 0 when power pellet expires

### Public API (from spec)

**`src/scoring.rs`:**
- `fn add_points(score: &mut Score, event: ScoreEvent)`
- `fn lose_life(state: &mut GameState) -> bool` — returns true if game over
- `fn level_bonus(level: u8) -> u32` — returns 100 * level
- `fn fruit_value(level: u8) -> u32` — per spec: 100, 300, 500, 700, 1000, 2000, 3000, 5000, 5000, 5000

**`src/scoreboard.rs`:**
- `fn load_scores() -> io::Result<Vec<ScoreEntry>>`
- `fn save_scores(entries: &[ScoreEntry]) -> io::Result<()>`
- `fn is_top_10(entries: &[ScoreEntry], score: u32) -> bool`
- `fn insert_score(entries: &mut Vec<ScoreEntry>, entry: ScoreEntry)` — maintains top-10 descending

## Acceptance criteria (subset)

- `add_points` correctly accumulates score for each `ScoreEvent` variant
- Ghost-eat chain progression: first ghost = 200, second = 400, third = 800, fourth = 1600
- Ghost-eat chain resets when returning 0 (power pellet expires)
- `lose_life` decrements lives, returns true when lives reach 0
- `level_bonus(1)` returns 100, `level_bonus(10)` returns 1000
- `fruit_value` returns correct per-level values
- `load_scores` on missing file returns empty Vec
- `load_scores` on corrupt file returns empty Vec (prints warning to stderr)
- `save_scores` writes atomically via temp+rename
- `insert_score` maintains sorted descending order, max 10 entries
- `is_top_10` returns true if score qualifies
- Scoreboard directory creation with 0o700 permissions

## Interface contracts

Sub-task B exports:
- `ScoreEvent`, `Score`, `ScoreEntry` types
- Functions: `add_points`, `lose_life`, `level_bonus`, `fruit_value`, `load_scores`, `save_scores`, `is_top_10`, `insert_score`

Note: `GameState` is defined in sub-task E (`app.rs`). For `lose_life`, you need to define a minimal `GameState` locally in `scoring.rs` or use a trait. The simplest approach: define a struct `ScoringState` in `scoring.rs` with `lives: u8` and change `lose_life` to take `&mut ScoringState` instead. The `GameState` in `app.rs` (E) will embed `ScoringState` or mirror its fields. Coordinate: `scoring.rs` exports `ScoringState { pub lives: u8 }` and `lose_life(state: &mut ScoringState) -> bool`. E wraps this.

The actual `GameState` lives in E. To keep write-sets disjoint, define a minimal `ScoringState` or just have `lose_life` take `&mut u8` (lives count) directly, and E calls it. Simplest approach: `fn lose_life(lives: &mut u8) -> bool`.

## Must not touch

- Any other `src/` file
- `Cargo.toml`
- `docs/`
- Maze, entities, rendering, input, or app logic
- `GameState` definition (that's E's job)
