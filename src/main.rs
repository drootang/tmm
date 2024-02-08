use tmm::app::{App, AppResult};
use tmm::event::{Event, EventHandler};
use tmm::handler::handle_key_events;
use tmm::tui::Tui;
use std::io;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

fn main() -> AppResult<()> {
    // Create an application.
    let mut app = App::new();

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    // Start the main loop.
    while app.running {
        // Render the user interface.
        tui.draw(&mut app)?;
        // Handle events.
        match tui.events.next()? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
    }

    // Exit the user interface.
    tui.exit()?;

    // If we need to exec to a new process (attach a tmux session), do it here
    if let Some((name, detach_others)) = app.attach_session {
        // Exec 
        let mut cmd = exec::Command::new("tmux");
        cmd.arg("a");
        if detach_others {
            cmd.arg("-d");
        }
        let err = cmd.arg("-t").arg(name.as_str()).exec();
        panic!("{}", err);
    }
    Ok(())
}
