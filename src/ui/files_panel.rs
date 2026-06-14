use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};

use crate::models::{FileStatus, GitStatusCode};

pub fn render(frame: &mut Frame, area: Rect, files: &[FileStatus], selected: usize, active: bool) {
    let border_style = if active {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(format!(" Changed files ({}) ", files.len()));

    if files.is_empty() {
        let list = List::new([ListItem::new(Line::from(Span::styled(
            "working tree clean",
            Style::default().fg(Color::DarkGray),
        )))])
        .block(block);
        frame.render_widget(list, area);
        return;
    }

    let items: Vec<ListItem> = files.iter().map(file_item).collect();
    let list = List::new(items).block(block).highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    );

    let mut state = ListState::default();
    state.select(Some(selected.min(files.len() - 1)));
    frame.render_stateful_widget(list, area, &mut state);
}

fn file_item(file: &FileStatus) -> ListItem<'_> {
    let code = if file.untracked {
        "??".to_string()
    } else {
        format!(
            "{}{}",
            file.index_status.as_char(),
            file.worktree_status.as_char()
        )
    };

    let color = status_color(file);

    let mut spans = vec![
        Span::styled(format!("{code} "), Style::default().fg(color)),
        Span::raw(file.path.clone()),
    ];
    if let Some(old) = &file.old_path {
        spans.push(Span::styled(
            format!(" (from {old})"),
            Style::default().fg(Color::DarkGray),
        ));
    }

    ListItem::new(Line::from(spans))
}

fn status_color(file: &FileStatus) -> Color {
    if file.conflicted {
        return Color::Red;
    }
    if file.untracked {
        return Color::Magenta;
    }
    let primary = if file.unstaged {
        file.worktree_status
    } else {
        file.index_status
    };
    match primary {
        GitStatusCode::Added => Color::Green,
        GitStatusCode::Deleted => Color::Red,
        GitStatusCode::Renamed | GitStatusCode::Copied => Color::Blue,
        GitStatusCode::Modified => Color::Yellow,
        _ => Color::White,
    }
}
