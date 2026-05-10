use std::process::Command;

use ratatui::{Terminal, backend::CrosstermBackend};

use crate::{
    app::App,
    event::{Event, EventHandler},
    ssh_config::config_reader::AuthMethod,
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
    let mut tui = Tui::new(terminal, events, 250);
    tui.enter()?;

    while !app.should_quit {
        tui.draw(&mut app)?;

        let Some(ref events) = tui.events else {
            break;
        };
        match events.next()? {
            Event::Tick => {}
            Event::Key(key_event) => update(&mut app, key_event),
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        };

        if let Some(host) = app.current_ssh.take() {
            tui.exit()?; // restore terminal
            let mut cmd = Command::new("ssh");

            if let Some(user) = &host.username {
                cmd.arg("-l").arg(user);
            }

            if let Some(port) = host.port {
                cmd.arg("-p").arg(port.to_string());
            }

            if let Some(AuthMethod::Key(key_path)) = &host.auth {
                cmd.arg("-i").arg(key_path);
            }

            cmd.arg(&host.hostname);

            let status = cmd.status()?;

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
