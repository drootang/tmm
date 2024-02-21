use std::{
    collections::{HashMap, HashSet}, env, error, process::Command, str::from_utf8
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
    NewSession,
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
    /// Rename prompt
    pub rename_session_ta: Option<TextArea<'a>>,
    /// New session name prompt
    pub new_session_ta: Option<TextArea<'a>>,
    /// Search prompt
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
            rename_session_ta: None,
            search_session_ta: None,
            search_session_selected: None,
            matching_rows: vec![],
            hotkeys: [
                (AppState::Sessions, [
                    ("q", "Quit"),
                    ("a", "Attach Session"),
                    ("r", "Rename"),
                    ("n", "New"),
                    ("x", "Delete"),
                    ("/", "Search"),
                ].iter().cloned().collect()),
                (AppState::Deleting, [
                    ("q", "Quit"),
                    ("Esc", "Back"),
                    ("y", "Delete"),
                    ("n", "Cancel"),
                ].iter().cloned().collect()),
                (AppState::Renaming, [
                    ("Esc", "Back"),
                    ("Enter", "Rename"),
                ].iter().cloned().collect()),
                (AppState::WarnNested, [
                    ("q", "Quit"),
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

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn attach(&mut self, name: String, detach_others: bool) {
        self.running = false;
        self.on_exit = ExitAction::AttachSession(name.clone(), detach_others);
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
        let Ok(stdout) = from_utf8(&output.stdout) else { return };
        // Since the list can change between refreshes, need to get the name of the currently
        // highlighted session and then re-select that row after the list is updated.
        let selected_name = self.sessions.get(self.selected_session).map_or(None, |x| Some(x.0.to_owned()));
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
        // Find the selected_name in the new session list and select it. If it's not there, do not
        // change the selected row (e.g., on a rename, the new session will not be present, but
        // want to maintain the selection)
        if let Some(selected_name) = selected_name {
            if let Some(idx) = self.sessions.iter().position(|(name, _)| name == &selected_name) {
                self.selected_session = idx
            }
        }
        // Ensure the selected session is legal
        self.selected_session = self.selected_session.max(0).min(self.sessions.len()-1);
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
        self.rename_session_ta = Some(textarea);
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
        self.rename_session_ta = None;
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
            .expect(format!("failed to rename tmux session: {}", name).as_str());
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

    pub fn confirm_new_session(&mut self) {
        // Create the textarea and switch to renaming state
        let mut textarea = TextArea::default();
        textarea.set_cursor_line_style(Style::default());
        self.new_session_ta = Some(textarea);
        self.state = AppState::NewSession;
    }

    /// Create a new session
    pub fn new_session(&mut self, name: Option<&str>) {
        if let Some(name) = name {
            // Create the named session, and highlight it in the list
            let proc = Command::new("tmux")
                .args(["new-session", "-d", "-s", name])
                .output()
                .expect(format!("failed to create new tmux session: {}", name).as_str());
            if !proc.status.success() {
                panic!("This is the failure message: {}", std::str::from_utf8(&proc.stderr).unwrap());
                // TODO: display popup with error
            }
            // TODO: one common failure mode might be that the name already exists, e.g,
            // "duplicate session: <name>"

            // Highlight the newly created session. Tmux may modify characters that are provided
            // based on illegal tmux session names (e.g., 8.1 -> 8_1). It does not report this
            // modification, so we should discover the new session name using the set difference of
            // the new list of sessions and the old list of sessions.
            //
            // TODO: if the user creates new sessions once the new-session procedure has started in
            // tmm, multiple new sessions will appear in this set difference. Use fuzzy-matching to
            // find the best match for the session name among the new sessions to give the best
            // changes of highlighting the correct new session.
            //
            // Before refreshing, build a set of the current names
            let old_session_names: HashSet<String> = self.sessions.iter().map(|(name, _)| name.to_owned()).collect();
            self.refresh();
            let new_session_names: HashSet<String> = self.sessions.iter().map(|(name, _)| name.to_owned()).collect();
            if let Some(new_session_name) = new_session_names.difference(&old_session_names).next() {
                // We were able to find the new session name
                if let Some(idx) = self.sessions.iter().position(|(name, _)| name == new_session_name) {
                    self.selected_session = idx;
                }
            } else {
                // New session name not found for some reason. Do not change the selection.
            }
            self.dismiss_all();
        } else {
            // Exit and attach new session
            self.running = false;
            self.on_exit = ExitAction::NewSession;
        }
    }
}
