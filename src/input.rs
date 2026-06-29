use crate::maze::Direction;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Move(Direction),
    Select,
    Up,
    Down,
    Quit,
    Pause,
    None,
}

pub fn poll_input() -> Action {
    if !event::poll(std::time::Duration::from_millis(0)).unwrap_or(false) {
        return Action::None;
    }

    match event::read() {
        Ok(Event::Key(key)) => {
            if key.kind == KeyEventKind::Release {
                return Action::None;
            }
            match key.code {
                KeyCode::Up | KeyCode::Char('w') => Action::Move(Direction::Up),
                KeyCode::Down | KeyCode::Char('s') => Action::Move(Direction::Down),
                KeyCode::Left | KeyCode::Char('a') => Action::Move(Direction::Left),
                KeyCode::Right | KeyCode::Char('d') => Action::Move(Direction::Right),
                KeyCode::Enter => Action::Select,
                KeyCode::Esc => Action::Quit,
                KeyCode::Char('q') => Action::Quit,
                KeyCode::Char('p') => Action::Pause,
                _ => Action::None,
            }
        }
        Ok(Event::Resize(_, _)) => Action::None,
        _ => Action::None,
    }
}

pub fn check_resize() -> Option<(u16, u16)> {
    use crossterm::terminal;
    terminal::size().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_enum_variants() {
        let a = Action::Move(Direction::Up);
        assert_eq!(a, Action::Move(Direction::Up));
        assert_ne!(a, Action::Move(Direction::Down));
        assert_eq!(Action::Select, Action::Select);
        assert_eq!(Action::Quit, Action::Quit);
        assert_eq!(Action::None, Action::None);
    }

    #[test]
    fn test_check_resize_returns_some() {
        let sz = check_resize();
        assert!(sz.is_some());
        let (w, h) = sz.unwrap();
        assert!(w > 0);
        assert!(h > 0);
    }
}
