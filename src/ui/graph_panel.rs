use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use ratatui::Frame;

use crate::models::CommitNode;

pub fn render(
    frame: &mut Frame,
    area: Rect,
    commits: &[CommitNode],
    selected: usize,
    active: bool,
) {
    let items: Vec<ListItem> = commits.iter().map(commit_item).collect();

    let border_style = if active {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(" Commits "),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    let mut state = ListState::default();
    if !commits.is_empty() {
        state.select(Some(selected.min(commits.len() - 1)));
    }
    frame.render_stateful_widget(list, area, &mut state);
}

fn commit_item(commit: &CommitNode) -> ListItem<'_> {
    let mut spans = vec![
        Span::styled("* ", Style::default().fg(Color::Yellow)),
        Span::styled(
            commit.short_hash.clone(),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" "),
    ];

    for r in &commit.refs {
        let color = if r.contains("HEAD") {
            Color::Cyan
        } else if r.starts_with("tag: ") {
            Color::Magenta
        } else {
            Color::Green
        };
        spans.push(Span::styled(format!("({r}) "), Style::default().fg(color)));
    }

    spans.push(Span::raw(commit.subject.clone()));
    spans.push(Span::styled(
        format!("  {} - {}", commit.author, commit.relative_date),
        Style::default().fg(Color::DarkGray),
    ));

    ListItem::new(Line::from(spans))
}
