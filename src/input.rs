use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    Refresh,
    NextPanel,
    PrevPanel,
    Up,
    Down,
    DiffUp,
    DiffDown,
    ToggleDiffMode,
    None,
}

pub fn map_key(key: KeyEvent) -> Action {
    if key.kind != KeyEventKind::Press {
        return Action::None;
    }
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => Action::Quit,
        KeyCode::Char('r') => Action::Refresh,
        KeyCode::Char('f') => Action::ToggleDiffMode,
        KeyCode::Tab if key.modifiers.contains(KeyModifiers::CONTROL) => Action::PrevPanel,
        KeyCode::Tab => Action::NextPanel,
        // Shift+Tab arrives as BackTab; many terminals don't deliver
        // Ctrl+Tab at all, so this serves as a reliable fallback.
        KeyCode::BackTab => Action::PrevPanel,
        KeyCode::Up if key.modifiers.contains(KeyModifiers::SHIFT) => Action::DiffUp,
        KeyCode::Down if key.modifiers.contains(KeyModifiers::SHIFT) => Action::DiffDown,
        // Shifted letters arrive as uppercase chars rather than a modifier.
        KeyCode::Char('W') => Action::DiffUp,
        KeyCode::Char('S') => Action::DiffDown,
        KeyCode::Up | KeyCode::Char('w') => Action::Up,
        KeyCode::Down | KeyCode::Char('s') => Action::Down,
        _ => Action::None,
    }
}
