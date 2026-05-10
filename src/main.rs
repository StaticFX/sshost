use std::process::Command;

use ratatui::{Terminal, backend::CrosstermBackend};

use crate::{
    app::App,
    event::{Event, EventHandler},
    tui::Tui,
};

use update::update;

pub mod app;
pub mod event;
mod ssh_config;
pub mod tui;
pub mod ui;
pub mod update;

fn main() -> color_eyre::Result<()> {
    let mut app = App::default();
    let backend = CrosstermBackend::new(std::io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.enter()?;

    while !app.should_quit {
        tui.draw(&mut app)?;

        match tui.events.next()? {
            Event::Tick => {}
            Event::Key(key_event) => update(&mut app, key_event),
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        };

        if let Some(host) = app.current_ssh.take() {
            tui.exit()?; // restore terminal
            let status = Command::new("ssh").arg(&host).status()?;

            if !status.success() {
                let error = match status.code() {
                    Some(255) => "Connection refused or host unreachable",
                    Some(1) => "Authentication failed",
                    Some(code) => &format!("SSH exited with code {code}"),
                    None => "SSH was terminated by a signal",
                };
                app.ssh_status = Some(error.to_string());
            }

            app.current_ssh = None;

            tui.enter()?; // reinit terminal
        }
    }

    tui.exit()?;
    Ok(())
}
