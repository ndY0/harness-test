use crate::maze::{MazeGrid, Tile};
use crate::entities::{Ghost, GhostMode, GhostPersonality, PacMan, PacManState};
use crate::scoreboard::ScoreEntry;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_menu(frame: &mut Frame, area: Rect, selected: usize) {
    let items = ["Start Game", "High Scores", "Quit"];
    let mut lines: Vec<Line> = vec![
        Line::from(Span::styled(
            "=== PAC-MAN ===",
            Style::default().fg(Color::Yellow),
        )),
        Line::from(""),
    ];

    for (i, item) in items.iter().enumerate() {
        let style = if i == selected {
            Style::default().fg(Color::Yellow).add_modifier(ratatui::style::Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let prefix = if i == selected { "> " } else { "  " };
        lines.push(Line::from(Span::styled(
            format!("{}{}", prefix, item),
            style,
        )));
    }

    let paragraph = Paragraph::new(Text::from(lines))
        .block(Block::default().borders(Borders::NONE))
        .alignment(Alignment::Center);

    let centered_area = centered_rect(area, 30, 10);
    frame.render_widget(paragraph, centered_area);
}

pub fn render_high_scores(frame: &mut Frame, area: Rect, entries: &[ScoreEntry]) {
    let mut lines: Vec<Line> = vec![
        Line::from(Span::styled(
            "=== HIGH SCORES ===",
            Style::default().fg(Color::Yellow),
        )),
        Line::from(""),
    ];

    if entries.is_empty() {
        lines.push(Line::from(Span::styled(
            "No scores yet!",
            Style::default().fg(Color::White),
        )));
    } else {
        for (i, entry) in entries.iter().enumerate() {
            let pos = format!("{}.", i + 1);
            lines.push(Line::from(vec![
                Span::styled(
                    format!("{:>3}  ", pos),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    &entry.name,
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(
                    format!("  {}", entry.score),
                    Style::default().fg(Color::Green),
                ),
            ]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Press ESC to return",
        Style::default().fg(Color::DarkGray),
    )));

    let paragraph = Paragraph::new(Text::from(lines))
        .block(Block::default().borders(Borders::NONE))
        .alignment(Alignment::Center);

    let centered_area = centered_rect(area, 40, 16);
    frame.render_widget(paragraph, centered_area);
}

pub struct RenderContext<'a> {
    pub maze: &'a MazeGrid,
    pub pacman: &'a PacMan,
    pub ghosts: &'a [Ghost],
    pub score: u32,
    pub lives: u8,
    pub level: u8,
    pub fruit_active: bool,
    pub fruit_pos: Option<(usize, usize)>,
    pub power_pellet_flash: bool,
}

pub fn render_game(
    frame: &mut Frame,
    area: Rect,
    ctx: &RenderContext,
) {
    let hud_height = 1;

    // HUD
    let hud_area = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: hud_height as u16,
    };

    let hud_text = Line::from(vec![
        Span::styled(
            format!("SCORE: {}  ", ctx.score),
            Style::default().fg(Color::White),
        ),
        Span::styled(
            format!("LIVES: {}  ", "♥".repeat(ctx.lives as usize)),
            Style::default().fg(Color::Red),
        ),
        Span::styled(
            format!("LEVEL: {}", ctx.level),
            Style::default().fg(Color::Yellow),
        ),
    ]);

    frame.render_widget(Paragraph::new(hud_text), hud_area);

    // Maze area
    let maze_area = Rect {
        x: area.x,
        y: area.y + hud_height as u16,
        width: area.width,
        height: area.height - hud_height as u16,
    };

    // Center the 31x31 grid in the maze area
    let grid_start_x = maze_area.x + (maze_area.width.saturating_sub(31)) / 2;
    let grid_start_y = maze_area.y + (maze_area.height.saturating_sub(31)) / 2;

    for y in 0..31 {
        for x in 0..31 {
            let cell_x = grid_start_x + x as u16;
            let cell_y = grid_start_y + y as u16;

            // Skip if outside frame
            if cell_x >= area.x + area.width || cell_y >= area.y + area.height {
                continue;
            }

            let tile = ctx.maze.tiles[y][x];
            let consumed = ctx.maze.consumed[y][x];

            // Check if entity occupies this cell
            let mut entity_rendered = false;

            // Check ghosts
            for ghost in ctx.ghosts {
                if ghost.pos == (x, y) {
                    let (glyph, color) = ghost_glyph(ghost, ctx.power_pellet_flash);
                    render_cell(frame, cell_x, cell_y, glyph, color);
                    entity_rendered = true;
                    break;
                }
            }

            // Check Pac-Man
            if !entity_rendered && ctx.pacman.pos == (x, y) && ctx.pacman.state == PacManState::Alive {
                render_cell(frame, cell_x, cell_y, "●", Color::Yellow);
                entity_rendered = true;
            }

            // Check fruit
            if !entity_rendered && ctx.fruit_active {
                if let Some(fpos) = ctx.fruit_pos {
                    if fpos == (x, y) {
                        render_cell(frame, cell_x, cell_y, "♠", Color::Red);
                        entity_rendered = true;
                    }
                }
            }

            // Render tile if no entity
            if !entity_rendered {
                match tile {
                    Tile::Wall => render_cell(frame, cell_x, cell_y, "█", Color::Blue),
                    Tile::Dot if !consumed => render_cell(frame, cell_x, cell_y, "·", Color::White),
                    Tile::PowerPellet if !consumed => {
                        render_cell(frame, cell_x, cell_y, "◎", Color::White);
                    }
                    Tile::Portal(_) => render_cell(frame, cell_x, cell_y, "◎", Color::Cyan),
                    _ => {} // Empty or consumed tiles render as space
                }
            }
        }
    }
}

fn ghost_glyph(ghost: &Ghost, power_pellet_flash: bool) -> (&'static str, Color) {
    match ghost.mode {
        GhostMode::Frightened(_) => {
            if power_pellet_flash {
                ("♛", Color::Blue)
            } else {
                ("♛", Color::White)
            }
        }
        GhostMode::Eaten => ("●", Color::DarkGray),
        _ => {
            let color = match ghost.personality {
                GhostPersonality::Blinky => Color::Red,
                GhostPersonality::Pinky => Color::Magenta,
                GhostPersonality::Inky => Color::Cyan,
                GhostPersonality::Clyde => Color::Rgb(255, 165, 0), // Orange
            };
            ("♛", color)
        }
    }
}

fn render_cell(frame: &mut Frame, x: u16, y: u16, glyph: &str, color: Color) {
    let span = Span::styled(glyph, Style::default().fg(color));
    let area = Rect { x, y, width: 1, height: 1 };
    frame.render_widget(Paragraph::new(span), area);
}

pub fn render_game_over(
    frame: &mut Frame,
    area: Rect,
    score: u32,
    is_top10: bool,
    name_input: &str,
) {
    let mut lines: Vec<Line> = vec![
        Line::from(Span::styled(
            "=== GAME OVER ===",
            Style::default().fg(Color::Red),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("Final Score: {}", score),
            Style::default().fg(Color::White),
        )),
        Line::from(""),
    ];

    if is_top10 {
        lines.push(Line::from(Span::styled(
            "NEW HIGH SCORE!",
            Style::default().fg(Color::Yellow),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Enter your name (3 characters, A-Z 0-9):",
            Style::default().fg(Color::White),
        )));
        lines.push(Line::from(""));
        let display_name = format!("[{}]", name_input);
        lines.push(Line::from(Span::styled(
            display_name,
            Style::default().fg(Color::Cyan),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "Press Enter to return to menu",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let paragraph = Paragraph::new(Text::from(lines))
        .block(Block::default().borders(Borders::NONE))
        .alignment(Alignment::Center);

    let centered_area = centered_rect(area, 50, 14);
    frame.render_widget(paragraph, centered_area);
}

pub fn render_victory(frame: &mut Frame, area: Rect, score: u32) {
    let lines: Vec<Line> = vec![
        Line::from(Span::styled(
            "=== VICTORY! ===",
            Style::default().fg(Color::Yellow),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "You cleared all 10 levels!",
            Style::default().fg(Color::Green),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("Final Score: {}", score),
            Style::default().fg(Color::White),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Press Enter to continue",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let paragraph = Paragraph::new(Text::from(lines))
        .block(Block::default().borders(Borders::NONE))
        .alignment(Alignment::Center);

    let centered_area = centered_rect(area, 40, 10);
    frame.render_widget(paragraph, centered_area);
}

pub fn render_pause_overlay(frame: &mut Frame, area: Rect) {
    let lines: Vec<Line> = vec![
        Line::from(""),
        Line::from(Span::styled(
            "PAUSED",
            Style::default().fg(Color::Yellow).add_modifier(ratatui::style::Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Press P to resume, ESC to quit",
            Style::default().fg(Color::White),
        )),
    ];

    let paragraph = Paragraph::new(Text::from(lines))
        .alignment(Alignment::Center);

    let centered_area = centered_rect(area, 30, 6);
    frame.render_widget(
        ratatui::widgets::Clear,
        centered_area,
    );
    frame.render_widget(paragraph, centered_area);
}

pub fn render_terminal_too_small(frame: &mut Frame, area: Rect) {
    let lines: Vec<Line> = vec![
        Line::from(Span::styled(
            "Terminal too small",
            Style::default().fg(Color::Red),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Resize to at least 80x24 to continue.",
            Style::default().fg(Color::White),
        )),
    ];

    let paragraph = Paragraph::new(Text::from(lines))
        .alignment(Alignment::Center);

    let centered_area = centered_rect(area, 40, 5);
    frame.render_widget(paragraph, centered_area);
}

fn centered_rect(r: Rect, width: u16, height: u16) -> Rect {
    let x = r.x + (r.width.saturating_sub(width)) / 2;
    let y = r.y + (r.height.saturating_sub(height)) / 2;
    Rect {
        x,
        y,
        width: width.min(r.width),
        height: height.min(r.height),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::maze::Direction;

    #[test]
    fn test_ghost_glyph_blinky() {
        let ghost = Ghost {
            pos: (0, 0),
            dir: Direction::Up,
            personality: GhostPersonality::Blinky,
            mode: GhostMode::Chase,
            spawn: (0, 0),
            tick_counter: 0,
        };
        let (glyph, _) = ghost_glyph(&ghost, false);
        assert_eq!(glyph, "♛");
    }

    #[test]
    fn test_ghost_glyph_frightened() {
        let ghost = Ghost {
            pos: (0, 0),
            dir: Direction::Up,
            personality: GhostPersonality::Blinky,
            mode: GhostMode::Frightened(50),
            spawn: (0, 0),
            tick_counter: 0,
        };
        let (glyph, color) = ghost_glyph(&ghost, false);
        assert_eq!(glyph, "♛");
        assert_eq!(color, Color::White);
    }

    #[test]
    fn test_ghost_glyph_frightened_flash() {
        let ghost = Ghost {
            pos: (0, 0),
            dir: Direction::Up,
            personality: GhostPersonality::Blinky,
            mode: GhostMode::Frightened(50),
            spawn: (0, 0),
            tick_counter: 0,
        };
        let (glyph, color) = ghost_glyph(&ghost, true);
        assert_eq!(glyph, "♛");
        assert_eq!(color, Color::Blue);
    }

    #[test]
    fn test_ghost_glyph_eaten() {
        let ghost = Ghost {
            pos: (0, 0),
            dir: Direction::Up,
            personality: GhostPersonality::Blinky,
            mode: GhostMode::Eaten,
            spawn: (0, 0),
            tick_counter: 0,
        };
        let (glyph, _) = ghost_glyph(&ghost, false);
        assert_eq!(glyph, "●");
    }

    #[test]
    fn test_centered_rect() {
        let area = Rect { x: 0, y: 0, width: 100, height: 50 };
        let c = centered_rect(area, 40, 20);
        assert_eq!(c.x, 30);
        assert_eq!(c.y, 15);
        assert_eq!(c.width, 40);
        assert_eq!(c.height, 20);
    }

    #[test]
    fn test_centered_rect_smaller_than_parent() {
        let area = Rect { x: 5, y: 5, width: 20, height: 20 };
        let c = centered_rect(area, 40, 30);
        assert_eq!(c.width, 20);
        assert_eq!(c.height, 20);
    }
}
