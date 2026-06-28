use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::game::{Cell, GhostMode, Position};
use crate::game::ghost::{Ghost, GhostPersonality};

/// Renders the 21×21 maze grid into `area`, centred horizontally.
/// Player → 'C' yellow. Ghosts → 'ᗣ' with personality/fright colour.
/// Dots → '.' white. Power pellets → 'o' white bold. Walls → '#'. Empty → ' '.
pub fn draw_maze(frame: &mut Frame, area: Rect, game: &crate::ui::GameState, tick: u64) {
    let mut lines: Vec<Line> = Vec::with_capacity(21);

    for row in 0..21usize {
        let mut spans: Vec<Span<'static>> = Vec::with_capacity(21);
        for col in 0..21usize {
            let pos = Position { row, col };
            spans.push(cell_span(game, pos, tick));
        }
        lines.push(Line::from(spans));
    }

    let widget = Paragraph::new(lines).alignment(Alignment::Center);
    frame.render_widget(widget, area);
}

fn cell_span(game: &crate::ui::GameState, pos: Position, tick: u64) -> Span<'static> {
    // Player takes highest rendering priority.
    if game.player.pos == pos {
        return Span::styled("C", Style::default().fg(Color::Yellow));
    }

    let frightened_remaining = game.timers.frightened_remaining;

    // Non-frightened ghosts rendered before frightened (matches collision priority).
    for ghost in &game.ghosts {
        if ghost.is_in_house || ghost.pos != pos {
            continue;
        }
        if ghost.mode != GhostMode::Frightened {
            return Span::styled("\u{15E3}", Style::default().fg(personality_color(ghost.personality)));
        }
    }
    for ghost in &game.ghosts {
        if ghost.is_in_house || ghost.pos != pos {
            continue;
        }
        if ghost.mode == GhostMode::Frightened {
            let color = fright_color(frightened_remaining, tick);
            return Span::styled("\u{15E3}", Style::default().fg(color));
        }
    }

    // Background cell from maze + pellet map.
    background_span(game, pos)
}

fn fright_color(frightened_remaining: u32, tick: u64) -> Color {
    if frightened_remaining < 30 && tick % 2 == 0 {
        Color::White
    } else {
        Color::Blue
    }
}

fn personality_color(personality: GhostPersonality) -> Color {
    match personality {
        GhostPersonality::Blinky => Color::Red,
        GhostPersonality::Pinky => Color::Magenta,
        GhostPersonality::Inky => Color::Cyan,
        GhostPersonality::Clyde => Color::Yellow,
    }
}

fn background_span(game: &crate::ui::GameState, pos: Position) -> Span<'static> {
    match game.maze.cell(pos) {
        Cell::Wall => Span::raw("#"),
        Cell::Dot => {
            if game.pellets.is_eaten(pos) {
                Span::raw(" ")
            } else {
                Span::styled(".", Style::default().fg(Color::White))
            }
        }
        Cell::PowerPellet => {
            if game.pellets.is_eaten(pos) {
                Span::raw(" ")
            } else {
                Span::styled("o", Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
            }
        }
        Cell::PlayerStart | Cell::GhostHouse | Cell::Empty => Span::raw(" "),
    }
}
