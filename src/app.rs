use std::{
    collections::HashMap, env, error, process::Command, str::from_utf8
};
use tui_textarea::TextArea;
use ratatui::style::{Color, Style};
use ratatui::widgets::*;
use indexmap::IndexMap;

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum AppState {
    Sessions,
    SessionsSearch,
    Deleting,
    Renaming,
    WarnNested,
}

#[derive(Debug)]
pub enum ExitAction {
    AttachSession(String, bool),
    NewSession,
    None
}


/// Application.
#[derive(Debug)]
pub struct App<'a> {
    /// Is the application running?
    pub running: bool,
    /// counter
    pub counter: u8,
    /// session name to attach
    pub on_exit: ExitAction,
    /// Existing Tmux sessions (name, desc)
    pub sessions: Vec<(String, String)>,
    /// Selected session index
    pub selected_session: usize,
    /// The application state
    pub state: AppState,
    /// Rename string
    pub new_session_ta: Option<TextArea<'a>>,
    /// Rename string
    pub search_session_ta: Option<TextArea<'a>>,
    /// The row selected by a search operation
    pub search_session_selected: Option<usize>,
    /// All row indexes that match current search terms
    pub matching_rows: Vec<usize>,
    /// hotkey bar
    pub hotkeys: HashMap<AppState, IndexMap<&'a str, &'a str>>,
}

impl<'a> Default for App<'a> {
    fn default() -> Self {
        let mut def = Self {
            running: true,
            counter: 0,
            sessions: vec![],
            selected_session: 0,
            on_exit: ExitAction::None,
            state: AppState::Sessions,
            new_session_ta: None,
            search_session_ta: None,
            search_session_selected: None,
            matching_rows: vec![],
            hotkeys: [
                (AppState::Sessions, [
                    ("Q", "Quit"),
                    ("A", "Attach Session"),
                    ("R", "Rename"),
                    ("J", "Down"),
                    ("K", "Up"),
                    ("X", "Delete"),
                    ("/", "Search"),
                ].iter().cloned().collect()),
                (AppState::Deleting, [
                    ("Q", "Quit"),
                    ("Esc", "Back"),
                    ("Y", "Delete"),
                    ("N", "Cancel"),
                ].iter().cloned().collect()),
                (AppState::Renaming, [
                    ("Esc", "Back"),
                    ("Enter", "Rename"),
                ].iter().cloned().collect()),
                (AppState::WarnNested, [
                    ("Q", "Quit"),
                    ("Any", "Dismiss"),
                ].iter().cloned().collect()),
                (AppState::SessionsSearch, [
                    ("Esc", "Cancel"),
                    ("Enter", "Confirm"),
                    ("C-n", "Select next match"),
                    ("C-p", "Select previous match"),
                ].iter().cloned().collect()),
            ].iter().cloned().collect(),
        };
        def.refresh();
        def
    }
}

impl<'a> App<'a> {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn attach(&mut self, name: String, detach_others: bool) {
        if Self::is_nested() {
            self.state = AppState::WarnNested;
        } else {
            self.running = false;
            self.on_exit = ExitAction::AttachSession(name.clone(), detach_others);
        }
    }

    pub fn increment_counter(&mut self) {
        if let Some(res) = self.counter.checked_add(1) {
            self.counter = res;
        }
    }

    pub fn decrement_counter(&mut self) {
        if let Some(res) = self.counter.checked_sub(1) {
            self.counter = res;
        }
    }

    /// Refresh list of tmux sessions
    pub fn refresh(&mut self) {
        let output = Command::new("tmux")
            .args(["ls"])
            .output()
            .expect("failed to refresh tmux");
        if let Ok(stdout) = from_utf8(&output.stdout) {
            self.sessions = stdout.lines().filter_map(|line| {
                let mut parts = line.split(":");
                if let Some(name) = parts.next() {
                    // Get the name and remaining description
                    let remainder = parts.collect::<Vec<&str>>().join(" ");
                    Some((name.to_owned(), remainder))
                } else {
                    None
                }
            }).collect();
        }
    }

    /// Get the maximum width of all session names
    pub fn max_session_name_width(&self) -> usize {
        self.sessions.iter().map(|(name, _)| {
            name.len()
        }).fold(0, |acc, x| acc.max(x))
    }

    /// Start a confirmed delete
    pub fn confirm_delete(&mut self) {
        self.state = AppState::Deleting;
    }

    /// Start a confirmed rename
    pub fn confirm_rename(&mut self) {
        // Create the textarea and switch to renaming state
        let mut textarea = TextArea::default();
        textarea.set_cursor_line_style(Style::default());
        textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(" New Session Name ")
                .style(Style::default().bg(Color::DarkGray))
        );
        self.new_session_ta = Some(textarea);
        self.state = AppState::Renaming;
    }

    /// Start searching
    pub fn search(&mut self) {
        // Create the textarea and switch to renaming state
        let mut textarea = TextArea::default();
        textarea.set_cursor_line_style(Style::default());
        textarea.set_style(Style::default().fg(Color::DarkGray));
        self.search_session_ta = Some(textarea);
        self.state = AppState::SessionsSearch;
    }

    /// Return to the sessions view
    pub fn dismiss_all(&mut self) {
        self.new_session_ta = None;
        self.search_session_ta = None;
        self.state = AppState::Sessions;
    }

    pub fn is_nested() -> bool {
        let envs: HashMap<String, String> = env::vars().collect();
        envs.get("TMUX").is_some()
    }

    /// Rename selected session
    pub fn rename(&mut self, rename: &str) {
        let Some((name, _)) = self.sessions.get(self.selected_session) else {
            panic!("Could not identify session to delete");
        };
        let proc = Command::new("tmux")
            .args(["rename-session", "-t", name, rename])
            .output()
            .expect(format!("failed to rename tmux session {}", name).as_str());
        if !proc.status.success() {
            panic!("This is the failure message: {}", std::str::from_utf8(&proc.stderr).unwrap());
            // TODO: display popup with error
        }
        self.refresh();
        self.dismiss_all();
    }

    /// Delete a session
    pub fn delete(&mut self) {
        let Some((name, _)) = self.sessions.get(self.selected_session) else {
            panic!("Could not identify session to delete");
        };
        // Kill the session
        Command::new("tmux")
            .args(["kill-session", "-t", name])
            .output()
            .expect(format!("failed to kill tmux session {}", name).as_str());
        // TODO: check output.status and present dialog or message to user
        // instead of just expect panic?
        // Restore state with a refresh
        self.refresh();
        self.dismiss_all();
    }

    /// Start a new session and attach it if possible. If attach is expected to
    /// fail, refresh list instead and stay in tmm
    pub fn new_session(&mut self) {
        // Create a new session
        if Self::is_nested() {
            self.state = AppState::WarnNested;
        } else {
            // Exit and attach new session
            self.running = false;
            self.on_exit = ExitAction::NewSession;
        }
    }
}
