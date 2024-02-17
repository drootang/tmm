use ratatui::{
    layout::{Layout, Direction, Constraint, Rect},
    style::{Color, Style, Stylize},
    widgets::*,
    text::*,
    Frame,
};
use tui_textarea::TextArea;

use crate::app::{App, AppState};

/// Display a popup
///   x, y - top left coordinate
fn display_popup_centered(frame: &mut Frame, rect: &Rect, title: &str, message: &str, prompt: &str) {
    // TODO: accept proper trait for spans, text, etc so it can be styled
    // Compute proper size of popup. Add 4 to account for border and padding.
    let width: u16 = (title.len().max(message.len() + prompt.len()) + 4) as u16;
    let height: u16 = 3;
    // Find the center of the provided rect
    let x = (2 * rect.x + rect.width - width)/2;
    let y = (2 * rect.y + rect.height - height)/2;
    let area = Rect::new(x, y, width, height);
    //let area = Rect::new(x, y, width, height);
    frame.render_widget(Clear, area);
    // Configure a block to place the confirm message in
    let popup_block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .padding(Padding::horizontal(1))
        .style(Style::default().bg(Color::DarkGray));
    // Create the message inside the popup_block
    let msg = Paragraph::new(format!("{}{}", message, prompt))
            .block(popup_block);
    // Render
    frame.render_widget(msg, area);
}

fn display_prompt_centered(frame: &mut Frame, rect: &Rect, textarea: &TextArea) {
    // TODO: accept proper trait for spans, text, etc so it can be styled
    // Compute proper size of popup. Add 4 to account for border and padding.
    let width: u16 = (textarea.lines()[0].len()+4).max(18).max((rect.width/2) as usize) as u16;
    let height: u16 = 3;
    // Find the center of the provided rect
    let x = (2 * rect.x + rect.width - width)/2;
    let y = (2 * rect.y + rect.height - height)/2;
    let area = Rect::new(x, y, width, height);
    frame.render_widget(Clear, area);
    frame.render_widget(textarea.widget(), area);
}

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame) {
    // This is where you add new widgets.
    // See the following resources:
    // - https://docs.rs/ratatui/latest/ratatui/widgets/index.html
    // - https://github.com/ratatui-org/ratatui/tree/master/examples
    
    // Rendering philosophy:
    // we will use a stateful list where the list is 1 item per tmux session.
    // Highlight the selected session.
    
    // app.sessions is a vector of (name, desc). We want to join name and desc
    let width = app.max_session_name_width();
    let items: Vec<String> = app.sessions.iter().map(|(name, desc)| {
        format!("{:>2$}: {}", name, desc, width)
    }).collect();

    /**********/
    /* LAYOUT */
    /**********/

    // Split the screen to create layout sections/chunks
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(4 + app.sessions.len() as u16),
            Constraint::Length(1),
            Constraint::Percentage(100),
        ])
        .split(frame.size());

    /*****************/
    /* SESSIONS LIST */
    /*****************/

    // Set up the list state including selected row
    let mut state = ListState::default();
    state.select(Some(app.selected_session));

    frame.render_stateful_widget(
        List::new(items)
            .block(
                Block::bordered()
                    .title(" Tmux Session Manager ")
                    .padding(Padding::uniform(1))
            )
            //.style(Style::default())
            .highlight_style(Style::default().fg(Color::Cyan).reversed())
            .highlight_symbol(">> ")
            .repeat_highlight_symbol(false)
            .direction(ListDirection::TopToBottom),
        chunks[0], &mut state
    );
    
    /**********/
    /* POPUPS */
    /**********/

    // Possibly render popups depending on app state
    match app.state {
        AppState::Deleting => {
            // Get the name of the session
            let (name, _) = &app.sessions[app.selected_session];
            // Center the popup in the sessions rect
            display_popup_centered(frame, &chunks[0], "Confirm Delete",
                format!("Are you sure you want to delete {}?", name).as_str(),
                " [Y]es / [N]o"
            )
        },
        AppState::WarnNested => {
            display_popup_centered(frame, &chunks[0], "Error",
                "Cannot create nested session.",
                " [D]ismiss"
            )
        },
        AppState::Renaming => {
            // Render text input dialog to get the desired new name
            if let Some(textarea) = &app.new_session_ta {
                display_prompt_centered(frame, &chunks[0], textarea)
            }
        }
        _ => ()
    }

    /***********/
    /* HOTKEYS */
    /***********/

    // Get hotkeys by app state and map them to styled spans
    let hotkey_spans: Vec<Span> = match &app.hotkeys.get(&app.state) {
        // Get the hotkey map if it exists for this state
        Some(hotkeys) => hotkeys,
        // Use the Sessions state as a default if the current state does not have custom hotkeys
        // defined
        _ => app.hotkeys.get(&AppState::Sessions).expect("Could not get sessions hotkeys")
    }.iter().map(|(k, v)| {
            // Each hotkey will have the key highlighted in dark gray and description in normal
            // text with some spaces padding
            vec![
                Span::raw("  "),
                Span::styled(k.to_string(), Style::new().fg(Color::DarkGray).reversed()),
                Span::raw(format!(" {}", v)),
            ]
        }).flatten().collect();
    // render it
    frame.render_widget(Line::from(hotkey_spans), chunks[1]);
}
