use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::models::DiffMode;

pub fn render(
    frame: &mut Frame,
    area: Rect,
    diff: &str,
    scroll: u16,
    mode: DiffMode,
    active: bool,
) {
    let border_style = if active {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let inner_width = area.width.saturating_sub(2) as usize;
    let mut lines: Vec<Line> = Vec::new();
    for raw in diff.lines() {
        if is_metadata(raw) {
            continue;
        }
        // Visually separate hunks: blank line before each hunk header
        // (except at the very top), and a full-width highlighted header.
        if raw.starts_with("@@") {
            if !lines.is_empty() {
                lines.push(Line::from(""));
            }
            lines.push(hunk_header(raw, inner_width));
        } else {
            lines.push(diff_line(raw));
        }
    }

    let (added, removed) = count_changes(diff);
    let counts = Line::from(vec![
        Span::styled(format!(" +{added} "), Style::default().fg(Color::Green)),
        Span::styled(format!("-{removed} "), Style::default().fg(Color::Red)),
    ])
    .right_aligned();

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(format!(" Diff [{}] ", mode.label()))
                .title_top(counts),
        )
        .scroll((scroll, 0));

    frame.render_widget(paragraph, area);
}

/// Diff metadata lines that are noise for an interactive viewer.
fn is_metadata(line: &str) -> bool {
    line.starts_with("diff --git")
        || line.starts_with("index ")
        || line.starts_with("--- ")
        || line.starts_with("+++ ")
        || line.starts_with("new file mode")
        || line.starts_with("deleted file mode")
        || line.starts_with("old mode")
        || line.starts_with("new mode")
        || line.starts_with("similarity index")
        || line.starts_with("dissimilarity index")
        || line.starts_with("rename from")
        || line.starts_with("rename to")
        || line.starts_with("copy from")
        || line.starts_with("copy to")
}

/// Number of lines the panel will actually render for `diff`, accounting
/// for filtered metadata and inserted hunk separators.
pub fn rendered_line_count(diff: &str) -> usize {
    let mut count = 0;
    for line in diff.lines() {
        if is_metadata(line) {
            continue;
        }
        if line.starts_with("@@") && count > 0 {
            count += 1;
        }
        count += 1;
    }
    count
}

fn count_changes(diff: &str) -> (usize, usize) {
    let mut added = 0;
    let mut removed = 0;
    for line in diff.lines() {
        if line.starts_with("+++") || line.starts_with("---") {
            continue;
        }
        if line.starts_with('+') {
            added += 1;
        } else if line.starts_with('-') {
            removed += 1;
        }
    }
    (added, removed)
}

fn hunk_header(line: &str, width: usize) -> Line<'static> {
    let padded = format!("{line:<width$}");
    Line::from(Span::styled(
        padded,
        Style::default()
            .fg(Color::Cyan)
            .bg(Color::Indexed(236))
            .add_modifier(Modifier::BOLD),
    ))
}

fn diff_line(line: &str) -> Line<'_> {
    let style = if line.starts_with("+++") || line.starts_with("---") {
        Style::default().fg(Color::White)
    } else if line.starts_with('+') {
        Style::default().fg(Color::Green)
    } else if line.starts_with('-') {
        Style::default().fg(Color::Red)
    } else if line.starts_with("new file") || line.starts_with("==") {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default()
    };
    Line::from(Span::styled(line, style))
}
