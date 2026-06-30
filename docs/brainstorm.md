# Brainstorm: Pac-Man Terminal Game

## Problem statement
The user wants a terminal-based reimplementation of the classic Pac-Man arcade game in Rust. The core tension is that Pac-Man is fundamentally a real-time action game with fluid movement and split-second dodging, but the terminal is a grid of character cells with discrete input events — the design challenge is bridging that gap without sacrificing either the feel of the original or the advantages of the terminal medium. The existing codebase is a stub (a Greeter trait, Person, and Team) that must be replaced entirely, leaving only the Cargo project skeleton, its dependency list (ratatui, crossterm, serde, rand), and the binary target name `pacman-terminal`.

## User personas
1. **The Retro Gamer**: A player who grew up with arcade cabinets and wants an authentic Pac-Man experience — same ghost behaviors, same scoring rhythms, same tension. They judge the game by how close it "feels" to the original, not by how many features it adds.
2. **The Terminal Tinkerer**: A developer who lives in the terminal and wants a quick, satisfying break that runs inside their existing tmux/terminal setup. They value low resource usage, keyboard-only controls, and the ability to launch and quit in under a second.
3. **The Casual Player**: Someone who knows Pac-Man but doesn't care about arcade accuracy. They want clear visuals, forgiving difficulty, and a sense of progression across the 10 levels. They'll bounce off if the first level is frustratingly hard or the controls feel unresponsive.
4. **The Score Chaser**: A competitive player motivated entirely by the leaderboard. They'll replay levels to optimize routes, memorize ghost patterns, and exploit portal shortcuts. They care deeply about scoring transparency, fairness, and leaderboard integrity.

## Core use cases
1. Launch the game from the terminal and see the main menu with options to start, view scores, or quit.
2. Control Pac-Man through a maze in real time using arrow keys, eating dots as they traverse the corridors.
3. Avoid four ghosts that patrol the maze; lose a life on contact unless a power pellet is active.
4. Step onto a portal tile to instantly teleport to the paired portal elsewhere on the same level.
5. Eat a power pellet to temporarily turn ghosts vulnerable, then chase and eat them for bonus points before they recover.
6. Clear a level by eating every dot, then advance to the next level with a harder maze and faster ghosts.
7. Lose all lives and see a game-over screen showing the final score and whether it qualifies for the leaderboard.
8. View the persistent top-10 high score board, with names and scores remembered across application restarts.

## Directions

1. **Arcade-faithful**: Reproduce the original Pac-Man ghost AI (scatter/chase modes, individual personality vectors, frightened reversal), the exact dot-and-fruit scoring table, and the 240-tick game clock. Maze layouts would be approximations of the original arcade board scaled to terminal dimensions. Distinctive for authenticity; trades off complexity of implementation and may feel sluggish in a terminal without careful frame timing; serves the Retro Gamer and Score Chaser best.

2. **Terminal-native roguelike**: Make the game turn-based — Pac-Man moves one cell per keypress, ghosts move simultaneously. Emphasises tactical positioning over reflexes. Distinctive for being deeply strategic rather than twitchy; trades off the adrenaline of real-time play; serves the Terminal Tinkerer who prefers deliberate play, but alienates Retro Gamers.

3. **Fluid real-time with smart terminal rendering**: Use ratatui's tick-based event loop with a stable 100–200ms frame rate, delta-time movement smoothing, and queue-buffered input so holding a key produces continuous motion. Characters are rendered as Unicode glyphs (● for Pac-Man, ♥ for lives, ◆ for pellets) with colour. Distinctive for feeling responsive despite terminal constraints; trades off needing careful timing code and may introduce visual jitter on slow terminals; serves Casual Players and Terminal Tinkerers equally well.

4. **Procedural mazes with seeded generation**: Abandon hand-crafted levels in favour of algorithmically generated mazes per level, controlled by a seed so leaderboard runs are reproducible. Distinctive for infinite replayability and smaller binary size; trades off the curated difficulty curve and aesthetically pleasing layouts of hand-designed mazes; serves Score Chasers who want fresh challenges.

5. **Multiplayer via terminal multiplexing**: Support two players on the same screen (split or shared maze) where the second player controls a ghost. Distinctive for social play in a terminal; trades off significant implementation complexity, potential input collision issues, and the difficulty of making ghost-control fun; serves a niche of Terminal Tinkerers who enjoy local co-op.

6. **ASCII-art cinematic**: Lean into the terminal's text nature with elaborate ASCII title screens, animated level transitions, "cutscenes" between levels, and character-based particle effects for eating ghosts. Distinctive for personality and charm; trades off development time on non-gameplay elements; serves Casual Players who value polish and atmosphere.

7. **Minimalist pure-play**: Strip the game to its absolute essentials: one maze shape that mutates across levels (walls shift), no fruits, no cutscenes, instant restart. Focus entirely on tight controls and ghost AI. Distinctive for being a "pure" skill test with zero cruft; trades off charm, variety, and casual appeal; serves Score Chasers and purist Terminal Tinkerers.

8. **Level-editor-enabled**: Include a built-in TUI level editor that lets players design, save, and share custom mazes. Levels ship as external TOML files. Distinctive for community extensibility; trades off a significantly larger scope and the risk that the editor overshadows the base game; serves Terminal Tinkerers who love to mod.

## Constraints and non-negotiables
- Must be written in Rust, using the existing Cargo project at this repository root.
- Must use `ratatui` (0.28) for terminal rendering and `crossterm` (0.28) for terminal manipulation.
- Must produce a binary named `pacman-terminal` (already defined in `Cargo.toml`).
- Must include exactly 10 unique levels, each with a different maze layout.
- Every level must contain two portals (teleportation gates).
- Must have a scoring system (dots, power pellets, ghosts, level completion).
- Must have a health/lives system with game-over state.
- Must persist a top-10 high score board locally.
- The existing `src/main.rs` code (Greeter, Person, Team) must be replaced — it is a placeholder.
- `serde` and `serde_json` (already in Cargo.toml) must serve as the serialization layer for score persistence (inferred).
- `rand` (already in Cargo.toml) must serve random number generation for ghost AI or procedural elements (inferred).
- The game must be playable in a standard 80x24 terminal (inferred — not explicitly stated, but a terminal game that doesn't fit common dimensions is unusable).

## Open questions
1. **Real-time or turn-based?** This is the single most consequential decision. Real-time requires a game loop with timing, input buffering, and frame-rate management. Turn-based simplifies implementation but changes the game genre entirely. The answer cascades into rendering architecture, ghost AI design, and the entire player experience.
2. **What ghost AI fidelity is expected?** Original Pac-Man ghosts use distinct algorithms (Blinky targets Pac-Man directly; Pinky targets ahead; Inky uses a complex vector; Clyde chases then flees) with scatter/chase mode cycling. Should this be replicated, simplified, or replaced with a different model? The choice affects both implementation effort and authenticity.
3. **How should levels be authored and stored?** Inline Rust data structures? External files (TOML, JSON, or a custom text format)? Procedural generation with seeds? Each carries different trade-offs for modifiability, binary size, and development workflow.
4. **How do portals interact with ghosts?** Can ghosts also teleport through portals? If so, can they use them intentionally or randomly? If not, portals become a guaranteed escape for the player. This directly affects level design and difficulty.
5. **What is the exact scoring model?** Should it match the arcade original (10 points per dot, 50 per pellet, 200/400/800/1600 for sequential ghost eats, fruit bonuses at set dot thresholds)? Or a simplified model? The Score Chaser persona cares deeply about this.
6. **What changes across the 10 levels to increase difficulty?** Options include: ghost speed, ghost count, AI aggressiveness (shorter scatter phases), maze complexity (narrower corridors, fewer escape routes), portal placement, dot density, power pellet count, or time limits. The difficulty curve defines the progression arc.
7. **Where should the high-score file be stored?** XDG-compliant path (`~/.local/share/pacman-terminal/`), a simple file next to the binary, or a configurable location? This is a portability and user-expectation question — especially on Windows where XDG doesn't apply.
8. **What should the minimum terminal size be, and what happens on resize?** A Pac-Man maze needs at minimum roughly 20x15 cells for meaningful gameplay. Should the game refuse to launch below a threshold? Should it pause and display a "resize terminal" message? Should it re-render gracefully at any size?
9. **Power pellet duration and mechanics:** The original game has decreasing fright times per level (from ~6 seconds down to ~1 second) and ghosts flash white before reverting. Should this be replicated, simplified to a fixed duration, or made configurable? This significantly affects late-game difficulty and scoring potential.
10. **Should there be fruits or bonus items?** The original game spawns fruits (cherry, strawberry, orange, etc.) twice per level for bonus points. They add scoring depth and incentivise risk-taking. Omitting them simplifies implementation but flattens the scoring model.

## Research notes
No web research was conducted for this brainstorm. The analysis is based on domain knowledge of the original Pac-Man arcade game mechanics, the ratatui/crossterm Rust ecosystem, and terminal application design patterns.
