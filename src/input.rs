use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::game::GameAction;

/// Maps a raw terminal key event to a semantic [`GameAction`].
///
/// Returns `None` for any key that has no game binding.
/// This function is pure and stateless — it holds no mutable state.
pub fn map_key(event: KeyEvent) -> Option<GameAction> {
    // Ignore events with modifiers (Ctrl, Alt, etc.) to avoid capturing
    // terminal shortcuts accidentally, except for plain Shift which is
    // handled implicitly by the key code variants below.
    if event.modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) {
        return None;
    }

    match event.code {
        // Arrow keys
        KeyCode::Up => Some(GameAction::MoveUp),
        KeyCode::Down => Some(GameAction::MoveDown),
        KeyCode::Left => Some(GameAction::MoveLeft),
        KeyCode::Right => Some(GameAction::MoveRight),

        // WASD — both cases
        KeyCode::Char('w') | KeyCode::Char('W') => Some(GameAction::MoveUp),
        KeyCode::Char('a') | KeyCode::Char('A') => Some(GameAction::MoveLeft),
        KeyCode::Char('s') | KeyCode::Char('S') => Some(GameAction::MoveDown),
        KeyCode::Char('d') | KeyCode::Char('D') => Some(GameAction::MoveRight),

        // Pause
        KeyCode::Char('p') | KeyCode::Char('P') | KeyCode::Char(' ') => {
            Some(GameAction::Pause)
        }

        // Quit
        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => Some(GameAction::Quit),

        _ => None,
    }
}
