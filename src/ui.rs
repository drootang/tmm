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

fn display_prompt_centered(frame: &mut Frame, rect: &Rect, textarea: &TextArea, title: &str) {
    // TODO: accept proper trait for spans, text, etc so it can be styled
    // Compute proper size of popup. Add 4 to account for border and padding.
    let prompt = " > ";
    let plen = prompt.len() as u16;

    let width: u16 = (textarea.lines()[0].len()+4).max(18).max((rect.width/2) as usize) as u16;
    let height: u16 = 3;
    // Find the center of the provided rect
    let x = (2 * rect.x + rect.width - width)/2;
    let y = (2 * rect.y + rect.height - height)/2;
    let area = Rect::new(x, y, width, height);

    let block = Block::bordered().title(format!(" {} ", title)).style(Style::default().bg(Color::DarkGray));
    frame.render_widget(Clear, area);
    // Get the inner area of the block that will be shared by the prompt and the textarea
    let inner_area = block.inner(area);
    let prompt_area = Rect{width: plen, ..inner_area};
    let ta_area = Rect{x: inner_area.x + plen, width: inner_area.width - plen, ..inner_area};
    // Render the block, prompt, and the textarea
    frame.render_widget(block, area);
    frame.render_widget(Span::styled(prompt, Style::default().fg(Color::Cyan)), prompt_area);
    frame.render_widget(textarea.widget(), ta_area);
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
    
    // app.sessions is a vector of (name, desc). We want to join name and desc.
    let width = app.max_session_name_width();

    // Set up the list state including selected row
    let mut state = ListState::default();
    state.select(Some(app.selected_session));

    // Compute the strings that will be displayed (one per row)
    let item_strings: Vec<String> = app.sessions.iter().map(|(name, desc)| {
        format!("{:>2$}: {}", name, desc, width)
    }).collect();

    let items: Vec<ListItem> = match app.state {
        AppState::SessionsSearch => {
            // If searching, filter/modify the items based on the current search string
            let search_needle = &app.search_session_ta.as_ref().expect("Could not get search term").lines()[0];
            let mut row_idx = 0;
            app.matching_rows.clear();
            let mapped_strings = item_strings.iter().map(|row| {
                // For each string, find any/all matches and convert result into a vec of spans
                let mut spans: Vec<Span> = vec![];
                let mut idx = 0;
                let mut matched = false;
                if !search_needle.is_empty() {
                    for (jdx, _) in row.match_indices(search_needle) {
                        spans.push(Span::raw(row[idx..jdx].to_owned()));
                        spans.push(Span::styled(search_needle.to_owned(), Style::default().fg(Color::Magenta)));
                        idx = jdx + search_needle.len();
                        matched = true;
                    }
                }
                if matched {
                    app.matching_rows.push(row_idx);
                }
                if idx < row.len() {
                    spans.push(Span::raw(row[idx..].to_owned()));
                }
                row_idx += 1;
                ListItem::new(Line::from(spans))
            }).collect();
            // If there is already a desired selection among matches
            if let Some(selected_match) = app.search_session_selected {
                // There is already a requested selected match. Only keep it if that row is in the
                // current vector of matching rows. If it is not, it means the user changed the
                // needle and their desired row no longer matches. In that case, use the first
                // matching row as the new selection. If the selected matches is empty, select
                // nothing.
                if app.matching_rows.is_empty() {
                    app.search_session_selected = None;
                }
                else if !app.matching_rows.contains(&selected_match) {
                    app.search_session_selected = Some(app.matching_rows[0])
                }
            }
            else if let Some(first_match) = app.matching_rows.get(0) {
                // There is no current desired selection among matches. Use the first match or none
                // if there are no matches
                app.search_session_selected = Some(*first_match);
            } else {
                app.search_session_selected = None;
            }
            state.select(app.search_session_selected);
            mapped_strings
        }
        _ => {
            item_strings.iter().map(|s| {
                ListItem::new(s.to_owned())
            }).collect()
        }
    };

    /**********/
    /* LAYOUT */
    /**********/

    // Split the screen to create layout sections/chunks
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Max(4 + app.sessions.len() as u16),
            Constraint::Length(1),
        ])
        .split(frame.size());

    /*****************/
    /* SESSIONS LIST */
    /*****************/

    frame.render_stateful_widget(
        List::new(items)
            .block(
                Block::bordered()
                    .title(" Tmux Session Manager ")
                    .padding(Padding::uniform(1))
            )
            .highlight_style(Style::default().fg(Color::Cyan).reversed())
            .highlight_symbol(">> ")
            .highlight_spacing(HighlightSpacing::Always)
            .repeat_highlight_symbol(false)
            .direction(ListDirection::TopToBottom),
        chunks[1], &mut state
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
            display_popup_centered(frame, &chunks[1], "Confirm Delete",
                format!("Are you sure you want to delete {}?", name).as_str(),
                " [Y]es / [N]o"
            )
        }
        AppState::WarnNested => {
            display_popup_centered(frame, &chunks[1], "Error",
                "Cannot create nested session.",
                " [D]ismiss"
            )
        }
        AppState::Renaming => {
            // Render text input dialog to get the desired new name
            if let Some(textarea) = &app.rename_session_ta {
                display_prompt_centered(frame, &chunks[1], textarea, "New Session Name")
            }
        }
        AppState::NewSession => {
            // Render text input dialog to get the desired new name
            if let Some(textarea) = &app.new_session_ta {
                display_prompt_centered(frame, &chunks[1], textarea, "New Session Name")
            }
        }
        AppState::SessionsSearch => {
            // Render text input dialog to get the desired new name
            if let Some(textarea) = &app.search_session_ta {
                // Need to render the search prompt immediately after the sessions list
                // Compute the rect
                let Rect{x, y, width, height} = chunks[1];
                let prompt_rect = Rect::new(x+2, y+height-2, width-2, 1);
                let search_rect = Rect::new(x+4, y+height-2, width-4, 1);
                frame.render_widget(Clear, search_rect);
                frame.render_widget(textarea.widget(), search_rect);
                frame.render_widget(Span::styled("> ", Style::new().fg(Color::Cyan)), prompt_rect);
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
    frame.render_widget(Line::from(hotkey_spans), chunks[2]);
}
