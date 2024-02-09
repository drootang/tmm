use ratatui::{
    layout::{Alignment, Layout, Direction, Constraint, Rect},
    style::{Color, Style, Stylize, Modifier},
    widgets::*,
    text::*,
    symbols,
    Frame,
};

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
        .title(title)
        .borders(Borders::ALL)
        .padding(Padding::horizontal(1))
        .style(Style::default().bg(Color::DarkGray));
    // Create the message inside the popup_block
    let msg = Paragraph::new(format!("{}{}", message, prompt))
            .block(popup_block);
    // Render
    frame.render_widget(msg, area);
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

    // Modify the list to set one of the ListItems to have another row and do
    // what we want
    //let mut items: Vec<ListItem> = items.iter().map(|x| ListItem::new(x.as_str())).collect();
    /*
    // Modify the first tiem to replace it with some stylized text
    items[0] = Line::from(vec![
        Span::raw("This is a "),
        Span::styled("new", Style::default().fg(Color::Green)),
        Span::raw(" test!"),
    ]).into();
    */
    //
    // Set up the list state
    let mut state = ListState::default();
    state.select(Some(app.selected_session));

    // Split the screen to create layout sections/chunks
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(4 + app.sessions.len() as u16),
            Constraint::Length(3),
            Constraint::Percentage(100),
        ])
        .split(frame.size());

    frame.render_stateful_widget(
        List::new(items)
            .block(Block::default().title(" Tmux Session Manager ").borders(Borders::ALL).padding(Padding::uniform(1)))
            //.style(Style::default())
            .highlight_style(Style::default().fg(Color::Cyan).reversed())
            .highlight_symbol(">> ")
            .repeat_highlight_symbol(false)
            .direction(ListDirection::TopToBottom),
        chunks[0], &mut state
    );

    frame.render_widget(
        Paragraph::new("  A: A hotkey  B: B hotkey")
            .block(Block::default().title("Hotkeys").borders(Borders::ALL))

        , chunks[1]
    );

    // Possibly render popups depending on app state
    match app.state {
        AppState::Deleting(index) => {
            // Get the name of the session
            let (name, _) = &app.sessions[index];
            // Center the popup in the sessions rect
            display_popup_centered(frame, &chunks[0], "Confirm Delete",
                format!("Are you sure you want to delete {}?", name).as_str(),
                " [Y]es / [N]o"
            )
        },
        AppState::WarnNested => {
            display_popup_centered(frame, &chunks[0], "Error",
                "Cannot create nested session.",
                " Press any key to continue."
            )
        },
        _ => ()
    }
}
