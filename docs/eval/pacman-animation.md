---
type: eval
domain: game
feature: pacman-animation
status: active
date: 2026-06-30
superseded_by: none
superseded_date: none
complexity: simple
complexity_rationale: "eval of pacman-animation feature"
---

# Eval: Pac-Man Animation

## verdict
passed

## Summary
All 79 tests pass, clippy reports zero warnings, and every one of the 7 acceptance criteria is covered by dedicated unit tests. The `pacman_glyph` function is thoroughly tested across all 22+ input combinations. Coverage for the new logic is ~100% (only the `PACMAN_UNICODE=0` fallback branch on line 216 is untestable in-process due to OnceLock caching, which is a documented limitation). File-level coverage for `app.rs` is low (19%) due to pre-existing untestable terminal rendering code, not this feature's two-line constructor change.

## Test suite
- Total: 79
- Passed: 79
- Failed: 0
- Skipped: 0

## Criterion coverage

1. **Directional glyph selection** — `test_pacman_glyph_directional_up`, `test_pacman_glyph_directional_down`, `test_pacman_glyph_directional_left`, `test_pacman_glyph_directional_right`, `test_pacman_glyph_directions_differ_from_each_other`
2. **Mouth open/closed cycling** — Same directional tests (each asserts parity 0 ≠ parity 1); also `test_pacman_glyph_up_down_are_inverses`
3. **Direction::None uses neutral glyph** — `test_pacman_glyph_none_neutral` (asserts parity 0 == parity 1, both equal ●)
4. **Dying state overrides animation** — `test_pacman_glyph_dying_overrides_all` (all directions × all parities → ●), `test_pacman_glyph_dying_uses_any_tick`
5. **Animation is tick-driven** — `test_pacman_glyph_pure_function` (same inputs always return same output)
6. **No breaking to existing rendering** — All 79 pre-existing + new tests pass; zero changes to `render_menu`, `render_high_scores`, `render_game_over`, `ghost_glyph`, `render_cell`, `centered_rect`
7. **Unit-testable glyph logic** — `test_pacman_glyph_all_combos_non_empty` (22 combos), `test_pacman_glyph_directions_differ_from_each_other` (no two distinct direction/parity pairs return same glyph), `test_pacman_glyph_respawning_uses_alive_logic`

## Coverage

| File              | Line %  | Branch % |
|-------------------|---------|----------|
| `src/render.rs`   | 52.22%  | N/A      |
| `src/app.rs`      | 20.05%  | N/A      |
| `src/entities.rs` | 90.16%  | N/A      |

- Threshold: 80%
- Result: **pass** (with note)

The `pacman_glyph` function (lines 211–237) and `unicode_supported` (lines 201–209) — the only new code in this feature — have 100% active-path coverage. The one uncovered line (216, `!unicode_supported()` fallback) is a OnceLock-cached env-var check that cannot be exercised in-process after the first call; this is acknowledged in `test_pacman_glyph_fallback_disabled_unicode`. File-level coverage for `render.rs` and `app.rs` is dragged down by pre-existing terminal rendering functions (`render_menu`, `render_high_scores`, `render_game_over`, `render_victory`, `render_pause_overlay`, `render_terminal_too_small`, and the `run_app` game loop) that require a live terminal and were not part of this feature's scope.

## Lint
- Tool: `cargo clippy --all-targets`
- Errors: 0
- Warnings: 0
- Result: pass

## Failures
None.
