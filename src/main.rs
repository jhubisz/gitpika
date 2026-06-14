mod app;
mod git;
mod input;
mod models;
mod ui;

use anyhow::Result;
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, MouseEventKind};
use crossterm::execute;

use app::App;
use input::Action;

fn main() -> Result<()> {
    let cwd = std::env::current_dir()?;
    let mut app = App::new(cwd);

    let mut terminal = ratatui::init();
    let _ = execute!(std::io::stdout(), EnableMouseCapture);
    let result = run(&mut terminal, &mut app);
    let _ = execute!(std::io::stdout(), DisableMouseCapture);
    ratatui::restore();
    result
}

fn run(terminal: &mut ratatui::DefaultTerminal, app: &mut App) -> Result<()> {
    while !app.should_quit {
        terminal.draw(|frame| ui::draw(frame, app))?;

        match event::read()? {
            Event::Key(key) => {
                let action = input::map_key(key);
                if action != Action::None {
                    app.handle_action(action);
                }
            }
            Event::Mouse(mouse) => match mouse.kind {
                MouseEventKind::ScrollUp => {
                    app.scroll_at(mouse.column, mouse.row, -1);
                }
                MouseEventKind::ScrollDown => {
                    app.scroll_at(mouse.column, mouse.row, 1);
                }
                _ => {}
            },
            Event::Resize(_, _) => {}
            _ => {}
        }
    }
    Ok(())
}
