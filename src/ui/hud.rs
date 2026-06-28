use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

/// Renders a single-line HUD: "SCORE: 00000   LIVES: ♥ ♥ ♥   STAGE: N   [status_msg]"
///
/// `stage` is 1-indexed. `status_msg` is appended if non-empty.
pub fn draw_hud(
    frame: &mut Frame,
    area: Rect,
    score: u32,
    lives: u8,
    stage: usize,
    status_msg: &str,
) {
    let lives_display = "\u{2665} ".repeat(lives as usize);
    let lives_str = lives_display.trim_end();

    let mut text = format!("SCORE: {:05}   LIVES: {}   STAGE: {}", score, lives_str, stage);
    if !status_msg.is_empty() {
        text.push_str("   ");
        text.push_str(status_msg);
    }

    let widget = Paragraph::new(Line::from(Span::raw(text)))
        .style(Style::default().fg(Color::White));
    frame.render_widget(widget, area);
}
