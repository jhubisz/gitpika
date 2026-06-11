use ratatui::layout::{Constraint, Direction, Layout, Rect};

#[derive(Clone, Copy)]
pub struct AppLayout {
    pub header: Rect,
    pub graph: Rect,
    pub files: Rect,
    pub diff: Rect,
}

pub fn build(area: Rect) -> AppLayout {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(area);

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)])
        .split(vertical[1]);

    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(columns[0]);

    AppLayout {
        header: vertical[0],
        graph: left[0],
        files: left[1],
        diff: columns[1],
    }
}
