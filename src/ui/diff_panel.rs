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
    let lines = build_lines(diff, inner_width);

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

const GUTTER_STYLE: Style = Style::new().fg(Color::Indexed(242));
const DIVIDER_STYLE: Style = Style::new().fg(Color::Indexed(238));

fn build_lines(diff: &str, inner_width: usize) -> Vec<Line<'_>> {
    let num_w = number_width(diff);
    let mut old_no: usize = 1;
    let mut new_no: usize = 1;
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
            if let Some((old, new)) = parse_hunk_header(raw) {
                old_no = old;
                new_no = new;
            }
            lines.push(hunk_header(raw, inner_width));
            continue;
        }

        // The leading +/- marker moves into the gutter; the code after the
        // divider is shown without it.
        let (gutter, sign, style, content) = match raw.as_bytes().first() {
            Some(b'+') => {
                let g = format!("{:>num_w$} {:>num_w$}", "", new_no);
                new_no += 1;
                (g, '+', Style::default().fg(Color::Green), &raw[1..])
            }
            Some(b'-') => {
                let g = format!("{:>num_w$} {:>num_w$}", old_no, "");
                old_no += 1;
                (g, '-', Style::default().fg(Color::Red), &raw[1..])
            }
            Some(b' ') => {
                let g = format!("{old_no:>num_w$} {new_no:>num_w$}");
                old_no += 1;
                new_no += 1;
                (g, ' ', Style::default(), &raw[1..])
            }
            // Non-diff lines: section labels, untracked-file header,
            // "\ No newline at end of file", placeholders.
            _ => {
                let g = format!("{:>num_w$} {:>num_w$}", "", "");
                (g, ' ', Style::default().fg(Color::DarkGray), raw)
            }
        };

        lines.push(Line::from(vec![
            Span::styled(gutter, GUTTER_STYLE),
            Span::styled(format!(" {sign} "), style),
            Span::styled("\u{2502} ", DIVIDER_STYLE),
            Span::styled(content, style),
        ]));
    }
    lines
}

/// Width of one line-number column, sized to the largest number shown.
fn number_width(diff: &str) -> usize {
    let mut max = 1usize;
    let mut old_no = 1usize;
    let mut new_no = 1usize;
    for line in diff.lines() {
        if is_metadata(line) {
            continue;
        }
        if line.starts_with("@@") {
            if let Some((old, new)) = parse_hunk_header(line) {
                old_no = old;
                new_no = new;
            }
            continue;
        }
        match line.as_bytes().first() {
            Some(b'+') => {
                max = max.max(new_no);
                new_no += 1;
            }
            Some(b'-') => {
                max = max.max(old_no);
                old_no += 1;
            }
            Some(b' ') => {
                max = max.max(old_no.max(new_no));
                old_no += 1;
                new_no += 1;
            }
            _ => {}
        }
    }
    max.to_string().len().max(3)
}

/// Extract the old/new start line numbers from `@@ -a,b +c,d @@ ...`.
fn parse_hunk_header(line: &str) -> Option<(usize, usize)> {
    let mut old = None;
    let mut new = None;
    for tok in line.split_whitespace() {
        if let (None, Some(rest)) = (old, tok.strip_prefix('-')) {
            old = rest.split(',').next().and_then(|s| s.parse().ok());
            continue;
        }
        if let (None, Some(rest)) = (new, tok.strip_prefix('+')) {
            new = rest.split(',').next().and_then(|s| s.parse().ok());
            continue;
        }
        if old.is_some() && new.is_some() {
            break;
        }
    }
    Some((old?, new?))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hunk_header_numbers() {
        assert_eq!(parse_hunk_header("@@ -12,8 +14,10 @@ fn main() {"), Some((12, 14)));
        assert_eq!(parse_hunk_header("@@ -5 +5,3 @@"), Some((5, 5)));
        assert_eq!(parse_hunk_header("@@ garbage @@"), None);
    }

    #[test]
    fn hunk_subject_does_not_confuse_parser() {
        // Trailing context contains -/+ tokens; only the first of each counts.
        assert_eq!(
            parse_hunk_header("@@ -1,2 +3,4 @@ a -b +c"),
            Some((1, 3))
        );
    }
}
