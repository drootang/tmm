use std::{
    error,
    collections::HashMap,
    process::Command,
    str::from_utf8,
};

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,
    /// counter
    pub counter: u8,
    /// session name to attach
    pub attach_session: Option<(String, bool)>,

    /// Existing Tmux sessions
    pub sessions: Vec<(String, String)>,

    /// Selected session index
    pub selected_session: usize,

}

impl Default for App {
    fn default() -> Self {
        let mut def = Self {
            running: true,
            counter: 0,
            sessions: vec![],
            selected_session: 0,
            attach_session: None
        };
        def.refresh();
        def
    }
}

impl App {
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
        self.running = false;
        self.attach_session = Some((name.clone(), detach_others));
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
}
