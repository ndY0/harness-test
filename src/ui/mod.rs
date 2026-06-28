pub const MIN_COLS: u16 = 80;
pub const MIN_ROWS: u16 = 30;

// STUB: replaced during S5 integration
pub struct App {
    pub state: AppState,
    pub game: GameState,
    pub tick_count: u64,
}

// STUB: replaced during S5 integration
pub enum AppState {
    Playing,
    Paused,
    GameOver,
    Victory,
}

// STUB: replaced during S5 integration
pub struct GameState {
    pub stage_index: usize,
    pub maze: crate::game::maze::Maze,
    pub pellets: crate::game::pellets::PelletMap,
    pub player: crate::game::player::Player,
    pub ghosts: [crate::game::ghost::Ghost; 4],
    pub score: crate::game::scoring::ScoreState,
    pub timers: crate::game::timers::GameTimers,
    pub dots_eaten: u32,
    pub status_msg: String,
}

pub mod hud;
pub mod renderer;

/// Top-level render function. Called each tick.
/// Shows an error widget if the terminal is too small; otherwise renders HUD + maze.
pub fn render(frame: &mut ratatui::Frame, app: &App) {
    use ratatui::{
        layout::{Constraint, Direction, Layout},
        widgets::Paragraph,
    };

    let size = frame.size();
    if size.width < MIN_COLS || size.height < MIN_ROWS {
        let msg = format!(
            "Terminal too small.\nNeed at least 80\u{00D7}30. Current: {}\u{00D7}{}.\nPlease resize and the game will resume automatically.",
            size.width, size.height
        );
        frame.render_widget(Paragraph::new(msg), size);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(size);

    hud::draw_hud(
        frame,
        chunks[0],
        app.game.score.score,
        app.game.player.lives,
        app.game.stage_index + 1,
        &app.game.status_msg,
    );
    renderer::draw_maze(frame, chunks[1], &app.game, app.tick_count);
}
