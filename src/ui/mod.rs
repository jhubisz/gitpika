pub mod diff_panel;
pub mod files_panel;
pub mod graph_panel;
pub mod layout;

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::app::{App, Panel};

pub fn draw(frame: &mut Frame, app: &mut App) {
    if app.snapshot.is_none() {
        app.layout = None;
        draw_error_screen(frame, app);
        return;
    }

    let layout = layout::build(frame.area());
    app.layout = Some(layout);
    let snapshot = app.snapshot.as_ref().unwrap();

    let branch = snapshot
        .repo
        .current_branch
        .as_deref()
        .unwrap_or("(detached)");
    let head = snapshot.repo.head_short.as_deref().unwrap_or("-------");
    let mut header_spans = vec![
        Span::styled(
            " GitPika ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(branch, Style::default().fg(Color::Green)),
        Span::raw(" @ "),
        Span::styled(head, Style::default().fg(Color::Yellow)),
        Span::styled(
            "   q quit | r refresh | Tab panel | Up/Down or w/s move | f diff view",
            Style::default().fg(Color::DarkGray),
        ),
    ];
    if let Some(err) = &app.error {
        header_spans.push(Span::styled(
            format!("   {err}"),
            Style::default().fg(Color::Red),
        ));
    }
    frame.render_widget(Paragraph::new(Line::from(header_spans)), layout.header);

    graph_panel::render(
        frame,
        layout.graph,
        &snapshot.commits,
        app.graph_selected,
        app.active_panel == Panel::Graph,
    );
    files_panel::render(
        frame,
        layout.files,
        &snapshot.files,
        app.files_selected,
        app.active_panel == Panel::Files,
    );
    diff_panel::render(
        frame,
        layout.diff,
        &app.diff_text,
        app.diff_scroll,
        app.diff_mode,
        app.active_panel == Panel::Diff,
    );
}

fn draw_error_screen(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let message = app
        .error
        .as_deref()
        .unwrap_or("not inside a Git repository");

    let lines = vec![
        Line::from(Span::styled(
            "GitPika",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(message, Style::default().fg(Color::Red))),
        Line::from(""),
        Line::from(Span::styled(
            "q to quit | r to retry",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let height = lines.len() as u16;
    let y = area.y + area.height.saturating_sub(height) / 2;
    let centered = Rect::new(area.x, y, area.width, height.min(area.height));
    frame.render_widget(
        Paragraph::new(lines).alignment(ratatui::layout::Alignment::Center),
        centered,
    );
}

#[cfg(test)]
mod quirk_test {
    use super::*;
    use crate::input::Action;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    #[test]
    fn tab_does_not_move_content() {
        let cwd = std::env::current_dir().unwrap();
        let mut app = App::new(cwd);
        let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();

        terminal.draw(|f| draw(f, &mut app)).unwrap();
        let before: Vec<String> = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol().to_string())
            .collect();

        app.handle_action(Action::NextPanel);
        terminal.draw(|f| draw(f, &mut app)).unwrap();
        let after: Vec<String> = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol().to_string())
            .collect();

        let diffs: Vec<usize> = (0..before.len())
            .filter(|&i| before[i] != after[i])
            .collect();
        for &i in diffs.iter().take(20) {
            println!(
                "cell ({}, {}): {:?} -> {:?}",
                i % 100,
                i / 100,
                before[i],
                after[i]
            );
        }
        assert!(
            diffs.is_empty(),
            "{} cells changed symbol after Tab",
            diffs.len()
        );
    }
}
