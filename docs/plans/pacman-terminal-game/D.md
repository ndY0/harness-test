---
feature: pacman-terminal-game
subtask: D
title: "Input handling and terminal rendering"
write_set: ["src/input.rs", "src/render.rs"]
depends_on: ["A", "C"]
complexity: simple
planner_depth: 1
---

## Context

Sub-task D implements crossterm event polling for keyboard input and ratatui frame-based rendering of the entire game. Depends on A for `Direction`, `MazeGrid`, `Tile`, `LevelConfig`; and C for `PacMan`, `Ghost`, `GhostPersonality`, `GhostMode`, `CollisionEvent`.

## Inputs from spec

### Input handling (`src/input.rs`)

crossterm event polling loop. Maps key events to an `Action` enum:
- `Action` enum: `Move(Direction)`, `Select`, `Up`, `Down`, `Quit`, `Pause`, `None`
- Arrow keys → `Move(Direction)`
- Enter → `Select`
- Escape → `Quit`
- `q` → `Quit` (alternate)
- `p` → `Pause`
- Ctrl+C → `Quit`

Input is polled non-blocking in the game loop tick. During gameplay, holding a key (repeated Key events) maps to continuous movement. Direction buffering: only the most recent direction is stored.

Public API:
- `fn poll_input() -> Action` — polls crossterm events, non-blocking, returns next action
- Must handle terminal resize events: expose `fn check_resize() -> Option<(u16, u16)>` or return resize info via Action

### Rendering (`src/render.rs`)

ratatui frame rendering for all game states. Functions:
- `fn render_menu(frame: &mut Frame, area: Rect, selected: usize)` — main menu with "Start Game", "High Scores", "Quit"
- `fn render_game(frame: &mut Frame, area: Rect, state: &GameState)` — the game screen: maze grid, entities, HUD
- `fn render_high_scores(frame: &mut Frame, area: Rect, entries: &[ScoreEntry])`
- `fn render_game_over(frame: &mut Frame, area: Rect, score: u32, is_top10: bool)`
- `fn render_victory(frame: &mut Frame, area: Rect, score: u32)`
- `fn render_pause_overlay(frame: &mut Frame, area: Rect)` — pause message
- `fn render_terminal_too_small(frame: &mut Frame, area: Rect)` — centered message

### Unicode glyphs and colors
- Pac-Man: `●` (U+25CF) — yellow color
- Ghosts: `♛` or similar character — each ghost distinct color: Blinky=Red, Pinky=Pink, Inky=Cyan, Clyde=Orange
- Frightened ghost: blue `♛`
- Wall: `█` (U+2588) or block character — blue/dark color
- Dot: `·` (U+00B7) — white
- Power Pellet: `●` (larger/bold) or `◎` — white blinking
- Portal: `◎` — cyan/green color
- Fruit: `♠` or `F` — red
- Empty: space
- HUD: score, lives (using `♥`), level number — top bar above maze

### HUD layout
- Top line: `SCORE: <score>  LIVES: <hearts>  LEVEL: <level>`
- Maze grid rendered below
- Game area centered in terminal

### Menu rendering
- Title "PAC-MAN" at top
- Navigable menu items highlighted when selected
- "High Scores" view shows scoreboard list
- Game Over shows final score, prompt for name entry if top-10

Note: Since `GameState` and `ScoreEntry` are defined in E and B respectively, use references to their fields. Keep rendering decoupled: pass primitive values or slices. For `render_game`, accept a struct or individual parameters for what to render.

Define a `RenderState` struct or just pass individual parameters:
- `fn render_game(frame: &mut Frame, area: Rect, maze: &MazeGrid, pacman: &PacMan, ghosts: &[Ghost], score: u32, lives: u8, level: u8, fruit_active: bool, fruit_pos: Option<(usize, usize)>)`

## Acceptance criteria (subset)

- `poll_input` correctly maps arrow keys to `Move(Direction)` actions
- `poll_input` maps Escape to `Quit`, Enter to `Select`
- `poll_input` is non-blocking (returns `Action::None` if no event)
- All render functions draw to the terminal frame without panicking
- Maze renders walls as filled blocks, paths as spaces, dots as `·`, power pellets as `◎`
- Entities render with correct Unicode glyphs and colors
- HUD displays score, lives (hearts), level
- Menu items highlight on selection
- Game-over screen shows final score
- Victory screen renders correctly
- Pause overlay renders over game screen
- Terminal-too-small message centered
- Rendering handles all edge cases (empty board, all dots eaten)

## Interface contracts

Sub-task D exports:
- `Action` enum
- `poll_input()`, `check_resize()` functions
- All render functions: `render_menu`, `render_game`, `render_high_scores`, `render_game_over`, `render_victory`, `render_pause_overlay`, `render_terminal_too_small`
- Any rendering helper types (e.g., color mappings)

D does NOT define `GameState` or `ScoreEntry` — receives data as parameters.

## Must not touch

- Any other `src/` file
- `Cargo.toml`
- `docs/`
- Maze data, entities, scoring, or app logic
- `GameState` or `ScoreEntry` definitions (that's E and B's job)
