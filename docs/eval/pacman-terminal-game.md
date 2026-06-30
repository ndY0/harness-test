# Eval: Pac-Man Terminal Game (round 2)

## verdict
passed

## Summary
All 66 unit tests pass with zero failures, and `cargo clippy -- -D warnings` reports zero errors and zero warnings — the 93 clippy lints from round 1 have been resolved. Every acceptance criterion has at least one covering test mapped from the first eval. The remaining coverage gaps (terminal-size-at-launch exit code, level-transition orchestration, pause-on-resize, RAII guard drop) are integration-level behaviors impractical to unit-test and were deemed secondary in round 1.

## Test suite
- Total: 66
- Passed: 66
- Failed: 0
- Skipped: 0
- Failing tests: none

## Criterion coverage
1. Launch binary → main menu (Start Game / High Scores / Quit), arrow keys navigate, Enter selects, Escape quits; terminal <80×24 refuses launch, prints error, exits code 1 — covered by: `app::tests::test_game_state_new`, `app::tests::test_app_state_enum`, `input::tests::test_action_enum_variants`. Terminal-size-at-launch exit code 1 not unit-tested (logic in `main.rs`).
2. Arrow key movement per tick, wall collision stop, buffered direction change, release stops at cell boundary — covered by: `entities::tests::test_pacman_move_right`, `entities::tests::test_pacman_move_into_wall`, `entities::tests::test_buffered_direction_change`, `entities::tests::test_buffered_direction_blocked`
3. Dot = 10pts, removed from maze, all dots eaten → next level, level 10 dots all eaten → victory screen with final score and high-score name entry — covered by: `scoring::tests::test_dot_scoring`, `maze::tests::test_consume_dot`, `maze::tests::test_dots_remaining`, `scoring::tests::test_level_complete_bonus`. Level-transition and victory-state branches not directly unit-tested (orchestrated in `tick_game`).
4. Ghost contact (Chase/Scatter) → lives −1, Pac-Man + ghosts reset to spawn; lives=0 → Game Over; 3 lives start — covered by: `scoring::tests::test_lose_life`, `scoring::tests::test_lose_life_at_zero`, `scoring::tests::test_score_new`, `entities::tests::test_collision_ghost_contact`
5. Power pellet = 50pts, fright mode with per-level decreasing duration (6000→1000ms), ghost direction reversal, half speed, sequential ghost-eat chain (200/400/800/1600), chain reset on expire, 2000ms flash period (alternating glyph each 200ms) — covered by: `scoring::tests::test_power_pellet_scoring`, `scoring::tests::test_ghost_eat_chain`, `scoring::tests::test_ghost_eat_chain_reset`, `ghosts::tests::test_enter_frightened_mode_reverses_direction`, `ghosts::tests::test_frightened_ghost_half_speed`, `ghosts::tests::test_exit_frightened_mode`, `render::tests::test_ghost_glyph_frightened`, `render::tests::test_ghost_glyph_frightened_flash`, `levels::tests::test_fright_duration_decreasing`
6. Portal tile teleports Pac-Man to paired portal same tick, preserves direction; ghost on destination in Chase = life lost, Pac-Man respawns; ghosts treat portals as empty path — covered by: `maze::tests::test_teleport_portal`, `maze::tests::test_is_portal`. Ghost-on-destination life loss orchestrated in `tick_game` (app.rs), not unit-tested standalone.
7. Fruit spawns at maze center at two thresholds (170 and 70 dots remaining), persists until collected or level cleared; fruit values per level (100/300/500/700/1000/2000/3000/5000/5000/5000); level completion bonus = 100 × level — covered by: `scoring::tests::test_fruit_scoring`, `scoring::tests::test_fruit_value_per_level`, `scoring::tests::test_level_bonus_values`, `scoring::tests::test_level_complete_bonus`, `app::tests::test_fruit_spawn_triggers`, `maze::tests::test_consume_fruit`
8. Game over + top-10 → prompt for 3-char uppercase alphanumeric name; high scores persist as JSON array at `~/.local/share/pacman-terminal/scores.json` with atomic write (temp + rename), directory created with 0o700; missing/corrupt file → empty scoreboard; top-10 maintained sorted descending, ties retain older entry — covered by: `scoreboard::tests::test_load_empty_when_no_file`, `scoreboard::tests::test_save_and_load_scores`, `scoreboard::tests::test_is_top_10_empty`, `scoreboard::tests::test_is_top_10_not_full`, `scoreboard::tests::test_is_top_10_qualifies`, `scoreboard::tests::test_insert_score_maintains_top_10`, `scoreboard::tests::test_tie_retains_older_entry`. 3-char alphanumeric name validation not directly unit-tested (logic in app.rs GameOver state).
9. Terminal resized below 80×24 during gameplay → pause + centered message; input discarded except Ctrl+C and Escape; auto-resume at ≥80×24 — covered by: `input::tests::test_check_resize_returns_some`. Pause-on-resize and auto-resume behavior not unit-tested (rendering/event-loop logic in app.rs).
10. Escape or Ctrl+C terminates; all exit paths restore raw mode and alternate screen via RAII drop guard; normal exits code 0, fatal I/O errors → stderr + code 1 — covered by: `input::tests::test_action_enum_variants`. RAII `TerminalGuard` drop and panic-hook restoration not unit-tested.

## Lint
- Tool: clippy
- Errors: 0
- Warnings: 0
- Result: pass

## Failures
None.
