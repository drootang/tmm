use crate::app::{App, AppResult, AppState};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    // Check global hotkeys first and always immediately handle them regardless
    // of mode

    // As long as the state is not Renaming, check the globals first
    if !matches!(app.state, AppState::Renaming(_)) {
        match key_event.code {
            // Exit application on `ESC` or `q`
            KeyCode::Char('q') => {
                app.quit();
                return Ok(());
            }
            // Exit application on `Ctrl-C`
            KeyCode::Char('c') | KeyCode::Char('C') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    app.quit();
                    return Ok(());
                }
            }
            _ => ()
        }
    }
    // Check hotkeys based on different app states that can handle different
    // keys
    match app.state {
        AppState::Sessions => {
            match key_event.code {
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
                KeyCode::Char('x') => {
                    // Start the delete process for the currently selected
                    // session
                    app.confirm_delete(app.selected_session);
                }
                KeyCode::Char('N') => {
                    // Create and attach a new session. If the user is currently
                    // in a tmux session so the attach would fail, instead of
                    // attempting attach, just refresh the list
                    app.new_session();
                }
                KeyCode::Char('r') => {
                    // TODO: r -> rename
                    app.confirm_rename(app.selected_session);
                }
                // TODO: d -> detach all clients from the session
                _ => {}
            }
        },
        AppState::Deleting(_) => {
            match key_event.code {
                KeyCode::Char('y') => {
                    // Delete the highlighted session
                    app.delete();
                },
                KeyCode::Char('n') | KeyCode::Esc => {
                    // Cancel - hide the popup
                    app.dismiss_all();
                },
                _ => (),
            }
        },
        AppState::Renaming(_) => {
            // If the renaming dialog is up, user can either escape out or hit enter to trigger
            // rename. Any valid symbols for a tmux session name should be pushed onto the rename
            // string
            match key_event.code {
                KeyCode::Esc => {
                    todo!("Dismiss renaming and cancel");
                },
                KeyCode::Enter => {
                    todo!("Confirm and implement rename");
                },
                KeyCode::Backspace => {
                    // Push this on
                    if let Some(ref mut name) = app.new_session_name {
                        name.pop();
                    }
                },
                KeyCode::Char(c) => {
                    // Push this on
                    if let Some(ref mut name) = app.new_session_name {
                        name.push(c);
                    } else {
                        app.new_session_name = Some(c.to_string());
                    }
                },
                _ => (),
            }
        },
        AppState::WarnNested => {
            // Any key should dismiss
            app.dismiss_all();
        }
    }
    Ok(())
}
