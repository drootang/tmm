use crate::app::{App, AppResult};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        // Exit application on `ESC` or `q`
        KeyCode::Esc | KeyCode::Char('q') => {
            app.quit();
        }
        // Exit application on `Ctrl-C`
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.quit();
            }
        }
        // Move up the list
        KeyCode::Char('k') => {
            app.selected_session = app.selected_session.checked_sub(1).unwrap_or(0)
        }
        // Move down the list
        KeyCode::Char('j') => {
            app.selected_session = (app.selected_session + 1).min(app.sessions.len()-1)
        }
        // Enter/select to attach
        KeyCode::Enter => {
            let name = app.sessions[app.selected_session].0.clone();
            app.attach(name, true);
        }
        // Jump to top of list
        KeyCode::Char('g') => {
            app.selected_session = 0;
        }
        // Jump to top end of list
        KeyCode::Char('G') => {
            app.selected_session = app.sessions.len() - 1;
        }
        // TODO: r -> rename
        // TODO: x -> delete
        // TODO: d -> detach all clients from the session
        _ => {}
    }
    Ok(())
}
