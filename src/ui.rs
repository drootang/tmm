use ratatui::{
    layout::{Alignment, Layout, Direction, Constraint},
    style::{Color, Style, Stylize, Modifier},
    widgets::*,
    text::*,
    symbols,
    Frame,
};

use crate::app::App;

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
    )
}
