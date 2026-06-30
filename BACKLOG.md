# Backlog

## Pac-Man Terminal Game
- status: done
- description: A terminal-based Pac-Man game with 10 unique levels, portals, scoring, health system, and scoreboard
- complexity: complex
- complexity_rationale: "9+ modules across the entire codebase — App, Maze, Entities, GhostAI, Input, Render, Scoring, Scoreboard, Levels — with real-time game loop, independent concurrent components (ghost AI, input polling, rendering), and >200 new lines of Rust."
- depends_on: none
- spec: docs/specs/pacman-terminal-game.md
- review: approved
- eval: passed
- iterations: 2
- blocked_reason: none

## Pac-Man Animation
- status: done
- description: Animate Pac-Man with mouth opening/closing across ticks, using a proper Pac-Man-like glyph (circle with wedge cut out) that rotates to face the current movement direction
- complexity: simple
- complexity_rationale: "3 files changed (app.rs, render.rs, entities.rs), no concurrent paths, <80 new lines"
- depends_on: pacman-terminal-game
- spec: docs/specs/pacman-animation.md
- review: approved
- eval: passed
- iterations: 0
- blocked_reason: none

## Ghost Visuals
- status: draft
- description: Slow down ghosts relative to Pac-Man speed and give them proper ghost-like textures with distinct colors and visual identity for each personality (Blinky/Pinky/Inky/Clyde)
- complexity: TBD
- complexity_rationale: TBD
- depends_on: pacman-terminal-game
- spec: none
- review: none
- eval: none
- iterations: 0
- blocked_reason: none

## Wall and Floor Textures
- status: draft
- description: Add colored textures to wall tiles (solid block appearance) and floor tiles (contrasting background), replacing plain Unicode characters with styled ratatui spans for visual depth
- complexity: TBD
- complexity_rationale: TBD
- depends_on: pacman-terminal-game
- spec: none
- review: none
- eval: none
- iterations: 0
- blocked_reason: none
