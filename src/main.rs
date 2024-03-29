use tmm::app::{App, AppResult};
use tmm::event::{Event, EventHandler};
use tmm::handler::handle_key_events;
use tmm::tui::Tui;
use std::io;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use clap::Parser;

use tmm::app::ExitAction;

/// A Textual User Interface (TUI) Tmux session manager
#[derive(Parser)]
#[command(about="
A Textual User Interface (TUI) Tmux session manager

When invoked with no session name, the TUI will start allowing easy management of Tmux sessions
such as attachment, renaming, deletion, etc.

If a session name is provided on the command line, the session will instead be immediately attached
or switched.")]
struct Args {
    /// Attach the named session immediately instead of starting the TUI
    #[arg(value_name="session name")]
    session_name: Option<String>
}

/// Attach or switch to a session name and exit
fn attach(name: &str, detach_others: bool) -> ! {
    let mut cmd = exec::Command::new("tmux");
    if App::is_nested() {
        // If currently nested, use switch-client instead of attach
        cmd.arg("switch-client");
    } else {
        cmd.arg("attach-session").arg("-d");
        if detach_others {
            cmd.arg("-d");
        }
    }
    let err = cmd.arg("-t").arg(name).exec();
    panic!("{}", err);
}

fn main() -> AppResult<()> {
    // Create an application.
    let mut app = App::new();

    let args = Args::parse();
    if let Some(session_name) = args.session_name {
        attach(&session_name, true);
    }

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new();
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    // Start the main loop.
    while app.running {
        // Render the user interface.
        tui.draw(&mut app)?;
        // Handle events.
        match tui.events.next()? {
            Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
    }

    // Exit the user interface.
    tui.exit()?;

    match app.on_exit {
        ExitAction::AttachSession(name, detach_others) => {
            attach(&name, detach_others);
        },
        ExitAction::NewSession => {
            let err = exec::Command::new("tmux")
                .arg("new-session").exec();
            panic!("{}", err);
        }
        ExitAction::None => ()
    }
    Ok(())
}
