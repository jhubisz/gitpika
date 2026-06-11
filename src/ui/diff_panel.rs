use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub fn render(frame: &mut Frame, area: Rect, diff: &str, scroll: u16, active: bool) {
    let border_style = if active {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let lines: Vec<Line> = diff.lines().map(diff_line).collect();

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(" Diff "),
        )
        .scroll((scroll, 0));

    frame.render_widget(paragraph, area);
}

fn diff_line(line: &str) -> Line<'_> {
    let style = if line.starts_with("+++") || line.starts_with("---") {
        Style::default().fg(Color::White)
    } else if line.starts_with('+') {
        Style::default().fg(Color::Green)
    } else if line.starts_with('-') {
        Style::default().fg(Color::Red)
    } else if line.starts_with("@@") {
        Style::default().fg(Color::Cyan)
    } else if line.starts_with("diff ")
        || line.starts_with("index ")
        || line.starts_with("new file")
        || line.starts_with("deleted file")
        || line.starts_with("==")
    {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default()
    };
    Line::from(Span::styled(line, style))
}
