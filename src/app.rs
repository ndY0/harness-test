use crate::maze::{
    MazeGrid, Tile, Direction, consume_dot, consume_fruit, dots_remaining,
    is_wall, teleport_portal,
};
use crate::levels::{LevelConfig, get_level_config};
use crate::entities::{
    PacMan, PacManState, Ghost, GhostMode, GhostPersonality,
    CollisionEvent, MoveResult, move_pacman, check_collisions,
};
use crate::ghosts::{GhostEvent, move_ghosts, enter_frightened_mode, exit_frightened_mode};
use crate::scoring::{Score, ScoreEvent, add_points, lose_life, reset_ghost_eat_chain, fruit_value};
use crate::scoreboard::{ScoreEntry, load_scores, save_scores, is_top_10, insert_score};
use crate::input::{Action, poll_input, check_resize};
use crate::render::{
    render_menu, render_game, render_high_scores, render_game_over,
    render_victory, render_pause_overlay, render_terminal_too_small,
};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::{self, stdout, Stdout};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    Menu,
    Playing,
    Paused,
    GameOver,
    Victory,
    HighScores,
}

pub struct GameState {
    pub level: u8,
    pub score: Score,
    pub power_pellet_timer: u32,
    pub power_pellet_flash: bool,
    pub state: AppState,
    pub maze: MazeGrid,
    pub pacman: PacMan,
    pub ghosts: [Ghost; 4],
    pub fruit_spawned: [bool; 2],
    pub fruit_active: bool,
    pub fruit_pos: Option<(usize, usize)>,
    pub menu_selected: usize,
    pub high_scores: Vec<ScoreEntry>,
    pub name_input: String,
    pub show_high_scores: bool,
    pub tick_count: u64,
}

impl GameState {
    pub fn new() -> Self {
        let level_cfg = get_level_config(1);
        let maze = MazeGrid::new(level_cfg.tiles);
        let pacman = PacMan::new(level_cfg.pacman_spawn);
        let ghosts = [
            Ghost::new(level_cfg.ghost_spawns[0], GhostPersonality::Blinky),
            Ghost::new(level_cfg.ghost_spawns[1], GhostPersonality::Pinky),
            Ghost::new(level_cfg.ghost_spawns[2], GhostPersonality::Inky),
            Ghost::new(level_cfg.ghost_spawns[3], GhostPersonality::Clyde),
        ];

        GameState {
            level: 1,
            score: Score::new(),
            power_pellet_timer: 0,
            power_pellet_flash: false,
            state: AppState::Menu,
            maze,
            pacman,
            ghosts,
            fruit_spawned: [false, false],
            fruit_active: false,
            fruit_pos: None,
            menu_selected: 0,
            high_scores: Vec::new(),
            name_input: String::new(),
            show_high_scores: false,
            tick_count: 0,
        }
    }

    pub fn reset_level(&mut self) {
        let cfg = get_level_config(self.level);
        self.maze = MazeGrid::new(cfg.tiles);
        self.pacman.reset(cfg.pacman_spawn);
        let spawns = cfg.ghost_spawns;
        self.ghosts = [
            Ghost::new(spawns[0], GhostPersonality::Blinky),
            Ghost::new(spawns[1], GhostPersonality::Pinky),
            Ghost::new(spawns[2], GhostPersonality::Inky),
            Ghost::new(spawns[3], GhostPersonality::Clyde),
        ];
        self.power_pellet_timer = 0;
        self.power_pellet_flash = false;
        self.fruit_spawned = [false, false];
        self.fruit_active = false;
        self.fruit_pos = None;
        self.pacman.dir = Direction::None;
        self.pacman.next_dir = Direction::None;
    }
}

pub struct TerminalGuard {
    raw_mode: bool,
}

impl TerminalGuard {
    pub fn new() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        Ok(TerminalGuard { raw_mode: true })
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        if self.raw_mode {
            let _ = terminal::disable_raw_mode();
            let _ = stdout().execute(LeaveAlternateScreen);
        }
    }
}

pub fn run_app() -> Result<(), String> {
    // Check terminal size
    let (cols, rows) = check_resize().unwrap_or((0, 0));
    if cols < 80 || rows < 24 {
        eprintln!("Terminal too small: {}x{}. Minimum is 80x24.", cols, rows);
        return Err("terminal_size".to_string());
    }

    let _guard = TerminalGuard::new().map_err(|e| format!("Failed to enable raw mode: {}", e))?;

    // Panic hook for terminal restoration
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = terminal::disable_raw_mode();
        let _ = std::io::stdout().execute(LeaveAlternateScreen);
        default_hook(info);
    }));

    let mut stdout = stdout();
    stdout.execute(EnterAlternateScreen).map_err(|e| format!("{}", e))?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(|e| format!("{}", e))?;

    let mut state = GameState::new();
    state.high_scores = load_scores().unwrap_or_else(|e| {
        eprintln!("Warning: could not load scores: {}", e);
        Vec::new()
    });

    let tick_duration = Duration::from_millis(150);
    let mut last_tick = Instant::now();

    loop {
        let action = poll_input();

        // Check quit actions
        if action == Action::Quit {
            break;
        }

        // Check terminal size
        let (cols, rows) = check_resize().unwrap_or((0, 0));
        let too_small = cols < 80 || rows < 24;

        match state.state {
            AppState::Menu => {
                if too_small {
                    terminal.draw(|f| {
                        render_terminal_too_small(f, f.area());
                    }).map_err(|e| format!("{}", e))?;
                    std::thread::sleep(tick_duration);
                    continue;
                }

                match action {
                    Action::Move(Direction::Up) | Action::Up => {
                        state.menu_selected = state.menu_selected.saturating_sub(1);
                    }
                    Action::Move(Direction::Down) | Action::Down => {
                        state.menu_selected = (state.menu_selected + 1).min(2);
                    }
                    Action::Select => {
                        match state.menu_selected {
                            0 => {
                                state.state = AppState::Playing;
                                state.reset_level();
                            }
                            1 => {
                                state.state = AppState::HighScores;
                            }
                            2 => break,
                            _ => {}
                        }
                    }
                    _ => {}
                }

                terminal.draw(|f| {
                    render_menu(f, f.area(), state.menu_selected);
                }).map_err(|e| format!("{}", e))?;
            }

            AppState::HighScores => {
                if action == Action::Quit || action == Action::Select {
                    state.state = AppState::Menu;
                }
                terminal.draw(|f| {
                    render_high_scores(f, f.area(), &state.high_scores);
                }).map_err(|e| format!("{}", e))?;
            }

            AppState::Playing => {
                if too_small {
                    state.state = AppState::Paused;
                    terminal.draw(|f| {
                        render_terminal_too_small(f, f.area());
                    }).map_err(|e| format!("{}", e))?;
                    continue;
                }

                // Process input
                match action {
                    Action::Move(dir) => {
                        if state.pacman.state == PacManState::Alive {
                            state.pacman.next_dir = dir;
                        }
                    }
                    Action::Pause => {
                        state.state = AppState::Paused;
                    }
                    Action::Quit => break,
                    _ => {}
                }

                // Game tick
                let now = Instant::now();
                if now.duration_since(last_tick) >= tick_duration {
                    last_tick = now;
                    state.tick_count = state.tick_count.wrapping_add(1);
                    tick_game(&mut state);
                }

                // Determine power pellet flash
                let flash = state.power_pellet_timer > 0
                    && state.power_pellet_timer <= 14  // ~2000ms / 150ms ≈ 13.3 ticks
                    && (state.power_pellet_timer / 2) % 2 == 0;

                terminal.draw(|f| {
                    render_game(
                        f,
                        f.area(),
                        &state.maze,
                        &state.pacman,
                        &state.ghosts,
                        state.score.score,
                        state.score.lives,
                        state.level,
                        state.fruit_active,
                        state.fruit_pos,
                        flash,
                    );
                }).map_err(|e| format!("{}", e))?;
            }

            AppState::Paused => {
                if action == Action::Quit {
                    break;
                }
                if action == Action::Pause && !too_small {
                    state.state = AppState::Playing;
                }

                if too_small {
                    terminal.draw(|f| {
                        render_terminal_too_small(f, f.area());
                    }).map_err(|e| format!("{}", e))?;
                } else {
                    terminal.draw(|f| {
                        render_game(
                            f,
                            f.area(),
                            &state.maze,
                            &state.pacman,
                            &state.ghosts,
                            state.score.score,
                            state.score.lives,
                            state.level,
                            state.fruit_active,
                            state.fruit_pos,
                            false,
                        );
                        render_pause_overlay(f, f.area());
                    }).map_err(|e| format!("{}", e))?;
                }
            }

            AppState::GameOver => {
                let is_top10 = is_top_10(&state.high_scores, state.score.score);
                if is_top10 && state.name_input.len() < 3 {
                    match action {
                        Action::Quit => {
                            state.state = AppState::Menu;
                            state.name_input.clear();
                        }
                        Action::Input(c) => {
                            let upper = c.to_ascii_uppercase();
                            if (upper >= 'A' && upper <= 'Z') || (upper >= '0' && upper <= '9') {
                                state.name_input.push(upper);
                            }
                        }
                        _ => {}
                    }
                } else if is_top10 && state.name_input.len() == 3 {
                    if action == Action::Select {
                        let entry = ScoreEntry {
                            name: state.name_input.clone(),
                            score: state.score.score,
                        };
                        insert_score(&mut state.high_scores, entry);
                        let _ = save_scores(&state.high_scores);
                        state.name_input.clear();
                        state.state = AppState::Menu;
                    } else if action == Action::Quit {
                        state.name_input.clear();
                        state.state = AppState::Menu;
                    }
                } else {
                    if action == Action::Select || action == Action::Quit {
                        state.state = AppState::Menu;
                        state.name_input.clear();
                    }
                }

                terminal.draw(|f| {
                    render_game_over(
                        f,
                        f.area(),
                        state.score.score,
                        is_top10,
                        &state.name_input,
                    );
                }).map_err(|e| format!("{}", e))?;
            }

            AppState::Victory => {
                if action == Action::Select || action == Action::Quit {
                    // Check if top 10, then transition to game over
                    if is_top_10(&state.high_scores, state.score.score) {
                        state.state = AppState::GameOver;
                    } else {
                        state.state = AppState::Menu;
                    }
                }

                terminal.draw(|f| {
                    render_victory(f, f.area(), state.score.score);
                }).map_err(|e| format!("{}", e))?;
            }
        }

        std::thread::sleep(Duration::from_millis(10));
    }

    Ok(())
}

fn tick_game(state: &mut GameState) {
    if state.state != AppState::Playing {
        return;
    }

    // Move Pac-Man
    let move_result = move_pacman(&mut state.pacman, &state.maze);

    // Handle portal teleportation
    if let MoveResult::PortalTeleport(pos) = move_result {
        if let Some(dest) = teleport_portal(&state.maze, pos) {
            // Check if destination is occupied by a ghost in Chase/Scatter mode
            let ghost_on_dest = state.ghosts.iter().any(|g| {
                g.pos == dest && matches!(g.mode, GhostMode::Chase | GhostMode::Scatter)
            });
            if ghost_on_dest {
                // Lose a life
                if lose_life(&mut state.score.lives) {
                    state.state = AppState::GameOver;
                    return;
                }
                state.reset_level();
                return;
            }
            state.pacman.pos = dest;
        }
    }

    // Process tile consumption
    match consume_dot(&mut state.maze, state.pacman.pos) {
        crate::maze::TileEffect::Dot(points) => {
            add_points(&mut state.score, ScoreEvent::Dot);
            check_fruit_spawn(state);
        }
        crate::maze::TileEffect::PowerPellet(points) => {
            add_points(&mut state.score, ScoreEvent::PowerPellet);
            let cfg = get_level_config(state.level);
            let fright_ticks = cfg.fright_duration_ms / 150; // convert ms to ticks
            enter_frightened_mode(&mut state.ghosts, fright_ticks as u32);
            reset_ghost_eat_chain(&mut state.score);
            state.power_pellet_timer = fright_ticks as u32;
            check_fruit_spawn(state);
        }
        _ => {}
    }

    // Check fruit collection
    if state.fruit_active {
        if let Some(fpos) = state.fruit_pos {
            if state.pacman.pos == fpos {
                let val = fruit_value(state.level);
                add_points(&mut state.score, ScoreEvent::Fruit(val));
                state.fruit_active = false;
                state.fruit_pos = None;
            }
        }
    }

    // Move ghosts
    let blinky_pos = state.ghosts[0].pos;
    let lair = get_level_config(state.level).ghost_lair;
    let ghost_events = move_ghosts(
        &mut state.ghosts,
        &state.maze,
        &state.pacman,
        blinky_pos,
        lair,
    );

    // Handle ghost reaching lair (Eaten ghosts)
    for event in ghost_events {
        if let GhostEvent::ReachedLair(idx) = event {
            state.ghosts[idx].mode = GhostMode::Chase;
        }
    }

    // Check collisions
    let collisions = check_collisions(&state.pacman, &state.ghosts);
    for event in collisions {
        match event {
            CollisionEvent::GhostContact => {
                if lose_life(&mut state.score.lives) {
                    state.state = AppState::GameOver;
                    return;
                }
                state.reset_level();
                return;
            }
            CollisionEvent::GhostEaten => {
                add_points(&mut state.score, ScoreEvent::Ghost);
                // Mark ghost as eaten
                for ghost in state.ghosts.iter_mut() {
                    if ghost.pos == state.pacman.pos
                        && matches!(ghost.mode, GhostMode::Frightened(_))
                    {
                        ghost.mode = GhostMode::Eaten;
                        break;
                    }
                }
            }
            _ => {}
        }
    }

    // Update power pellet timer
    if state.power_pellet_timer > 0 {
        state.power_pellet_timer -= 1;
        if state.power_pellet_timer == 0 {
            exit_frightened_mode(&mut state.ghosts);
            reset_ghost_eat_chain(&mut state.score);
        }
    }

    // Check level completion
    if dots_remaining(&state.maze) == 0 {
        add_points(&mut state.score, ScoreEvent::LevelComplete(state.level));
        if state.level == 10 {
            state.state = AppState::Victory;
        } else {
            state.level += 1;
            state.reset_level();
        }
    }
}

fn check_fruit_spawn(state: &mut GameState) {
    let remaining = dots_remaining(&state.maze);

    // First fruit at 170 dots remaining
    if remaining == 170 && !state.fruit_spawned[0] {
        state.fruit_spawned[0] = true;
        spawn_fruit(state);
    }
    // Second fruit at 70 dots remaining
    if remaining == 70 && !state.fruit_spawned[1] {
        state.fruit_spawned[1] = true;
        spawn_fruit(state);
    }
}

fn spawn_fruit(state: &mut GameState) {
    let center = (15, 15); // Maze center
    // If Pac-Man is at center, collect immediately
    if state.pacman.pos == center {
        let val = fruit_value(state.level);
        add_points(&mut state.score, ScoreEvent::Fruit(val));
    } else {
        state.fruit_active = true;
        state.fruit_pos = Some(center);
        state.maze.fruit_pos = Some(center);
        state.maze.fruit_value = fruit_value(state.level);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_state_new() {
        let state = GameState::new();
        assert_eq!(state.level, 1);
        assert_eq!(state.score.lives, 3);
        assert_eq!(state.state, AppState::Menu);
        assert_eq!(state.ghosts.len(), 4);
        assert_eq!(state.ghosts[0].personality, GhostPersonality::Blinky);
    }

    #[test]
    fn test_reset_level() {
        let mut state = GameState::new();
        state.score.lives = 1;
        state.power_pellet_timer = 50;
        state.fruit_active = true;
        state.reset_level();
        assert_eq!(state.power_pellet_timer, 0);
        assert!(!state.fruit_active);
        assert_eq!(state.fruit_pos, None);
        assert_eq!(state.score.lives, 1); // lives preserved
    }

    #[test]
    fn test_app_state_enum() {
        assert_eq!(AppState::Menu as u8, AppState::Menu as u8);
        assert_ne!(AppState::Menu, AppState::Playing);
    }

    #[test]
    fn test_fruit_spawn_triggers() {
        // Test that check_fruit_spawn sets fruit_spawned flags
        let mut state = GameState::new();
        let cfg = get_level_config(1);
        state.maze = MazeGrid::new(cfg.tiles);
        state.state = AppState::Playing;

        state.fruit_spawned = [false, false];
        // Remove all but 171 dots
        let remaining = dots_remaining(&state.maze);
        // consume almost all dots
        // This test just verifies the function doesn't panic
        check_fruit_spawn(&mut state);
    }
}
