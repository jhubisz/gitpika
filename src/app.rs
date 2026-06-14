use std::path::PathBuf;

use ratatui::layout::Position;

use crate::git;
use crate::input::Action;
use crate::models::{DiffMode, RepoSnapshot};
use crate::ui::layout::AppLayout;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    Graph,
    Files,
    Diff,
}

impl Panel {
    fn next(self) -> Self {
        match self {
            Panel::Graph => Panel::Files,
            Panel::Files => Panel::Diff,
            Panel::Diff => Panel::Graph,
        }
    }

    fn prev(self) -> Self {
        match self {
            Panel::Graph => Panel::Diff,
            Panel::Files => Panel::Graph,
            Panel::Diff => Panel::Files,
        }
    }
}

pub struct App {
    cwd: PathBuf,
    pub snapshot: Option<RepoSnapshot>,
    pub active_panel: Panel,
    pub graph_selected: usize,
    pub files_selected: usize,
    pub diff_scroll: u16,
    pub diff_text: String,
    pub diff_mode: DiffMode,
    pub error: Option<String>,
    pub should_quit: bool,
    /// Panel rectangles from the most recent draw, used for mouse hit-testing.
    pub layout: Option<AppLayout>,
}

impl App {
    pub fn new(cwd: PathBuf) -> Self {
        let mut app = Self {
            cwd,
            snapshot: None,
            active_panel: Panel::Files,
            graph_selected: 0,
            files_selected: 0,
            diff_scroll: 0,
            diff_text: String::new(),
            diff_mode: DiffMode::Hunks,
            error: None,
            should_quit: false,
            layout: None,
        };
        app.refresh();
        app
    }

    pub fn handle_action(&mut self, action: Action) {
        match action {
            Action::Quit => self.should_quit = true,
            Action::Refresh => self.refresh(),
            Action::NextPanel => self.active_panel = self.active_panel.next(),
            Action::PrevPanel => self.active_panel = self.active_panel.prev(),
            Action::Up => self.scroll_panel(self.active_panel, -1),
            Action::Down => self.scroll_panel(self.active_panel, 1),
            Action::DiffUp => self.scroll_panel(Panel::Diff, -1),
            Action::DiffDown => self.scroll_panel(Panel::Diff, 1),
            Action::ToggleDiffMode => {
                self.diff_mode = self.diff_mode.toggled();
                self.diff_scroll = 0;
                self.load_diff();
            }
            Action::None => {}
        }
    }

    /// Scroll the panel under the mouse cursor, regardless of which panel
    /// is active.
    pub fn scroll_at(&mut self, x: u16, y: u16, delta: i64) {
        let Some(panel) = self.panel_at(x, y) else {
            return;
        };
        // A wheel tick conventionally scrolls a few lines of text.
        let delta = if panel == Panel::Diff {
            delta * 3
        } else {
            delta
        };
        self.scroll_panel(panel, delta);
    }

    fn panel_at(&self, x: u16, y: u16) -> Option<Panel> {
        let layout = self.layout?;
        let pos = Position::new(x, y);
        if layout.graph.contains(pos) {
            Some(Panel::Graph)
        } else if layout.files.contains(pos) {
            Some(Panel::Files)
        } else if layout.diff.contains(pos) {
            Some(Panel::Diff)
        } else {
            None
        }
    }

    pub fn refresh(&mut self) {
        self.error = None;
        match git::load_snapshot(&self.cwd) {
            Ok(snapshot) => {
                self.graph_selected = clamp(self.graph_selected, snapshot.commits.len());
                self.files_selected = clamp(self.files_selected, snapshot.files.len());
                self.snapshot = Some(snapshot);
                self.diff_scroll = 0;
                self.load_diff();
            }
            Err(err) => {
                self.snapshot = None;
                self.diff_text.clear();
                self.error = Some(format!("{err:#}"));
            }
        }
    }

    fn scroll_panel(&mut self, panel: Panel, delta: i64) {
        let Some(snapshot) = &self.snapshot else {
            return;
        };
        match panel {
            Panel::Graph => {
                self.graph_selected = step(self.graph_selected, delta, snapshot.commits.len());
            }
            Panel::Files => {
                let before = self.files_selected;
                self.files_selected = step(self.files_selected, delta, snapshot.files.len());
                if self.files_selected != before {
                    self.diff_scroll = 0;
                    self.load_diff();
                }
            }
            Panel::Diff => {
                let rendered = crate::ui::diff_panel::rendered_line_count(&self.diff_text);
                let max = rendered.saturating_sub(1) as i64;
                let new = (self.diff_scroll as i64 + delta).clamp(0, max.max(0));
                self.diff_scroll = new as u16;
            }
        }
    }

    /// Load the diff for the currently selected changed file.
    fn load_diff(&mut self) {
        let Some(snapshot) = &self.snapshot else {
            self.diff_text.clear();
            return;
        };
        let Some(file) = snapshot.files.get(self.files_selected) else {
            self.diff_text = String::from("(no changed files)");
            return;
        };
        match git::diff::diff_for_file(&snapshot.repo.root, file, self.diff_mode) {
            Ok(text) => self.diff_text = text,
            Err(err) => {
                self.diff_text.clear();
                self.error = Some(format!("{err:#}"));
            }
        }
    }
}

fn clamp(index: usize, len: usize) -> usize {
    if len == 0 { 0 } else { index.min(len - 1) }
}

fn step(index: usize, delta: i64, len: usize) -> usize {
    if len == 0 {
        return 0;
    }
    let new = index as i64 + delta;
    new.clamp(0, len as i64 - 1) as usize
}
