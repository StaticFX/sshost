use std::process::Command;

use ratatui::{Terminal, backend::CrosstermBackend};

use crate::{
    app::App,
    app::screen::Screen,
    event::{Event, EventHandler},
    ssh_config::config_reader::AuthMethod,
    tui::Tui,
};

use update::update;

pub mod app;
pub mod event;
pub mod history;
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

            if let Some(proxy) = &host.proxy_jump {
                if !proxy.is_empty() {
                    cmd.arg("-J").arg(proxy);
                }
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
            } else {
                history::log_connection(&host.host);
            }

            app.current_ssh = None;

            tui.enter()?; // reinit terminal
        }

        if let Screen::UploadExecute(req) = &app.current_screen {
            let req = req.clone();
            tui.exit()?;

            // Create a temporary askpass script to pass the password
            let askpass_path = std::env::temp_dir().join("sshost_askpass.sh");
            if let Some(password) = &req.password {
                use std::os::unix::fs::PermissionsExt;
                let escaped = password.replace('\'', "'\\''");
                std::fs::write(&askpass_path, format!("#!/bin/sh\necho '{}'\n", escaped))?;
                std::fs::set_permissions(
                    &askpass_path,
                    std::fs::Permissions::from_mode(0o700),
                )?;
            }

            let mut cmd = Command::new("ssh-copy-id");
            cmd.arg("-i").arg(&req.key_path);

            if let Some(port) = req.port {
                cmd.arg("-p").arg(port.to_string());
            }

            let target = if let Some(user) = &req.user {
                format!("{}@{}", user, req.hostname)
            } else {
                req.hostname.clone()
            };
            cmd.arg(&target);

            if req.password.is_some() {
                cmd.env("SSH_ASKPASS", &askpass_path);
                cmd.env("SSH_ASKPASS_REQUIRE", "force");
                cmd.env("DISPLAY", ":0");
            }

            let status = cmd.status()?;

            // Clean up askpass script
            let _ = std::fs::remove_file(&askpass_path);

            let mut screen = crate::app::screen::upload_screen::UploadScreen::default();
            if !status.success() {
                screen.error = Some(match status.code() {
                    Some(255) => "Connection refused or host unreachable".to_string(),
                    Some(1) => "Authentication failed".to_string(),
                    Some(5) => "Invalid password".to_string(),
                    Some(6) => "Host key verification failed".to_string(),
                    Some(code) => format!("ssh-copy-id exited with code {code}"),
                    None => "ssh-copy-id was terminated by a signal".to_string(),
                });
            } else {
                app.current_screen = Screen::Intro(
                    crate::app::screen::intro_screen::IntroScreen::default(),
                );
                tui.enter()?;
                continue;
            }

            app.current_screen = Screen::Upload(screen);
            tui.enter()?;
        }

        // Keygen execution
        if let Screen::KeygenExecute(req) = &app.current_screen {
            let req = req.clone();
            tui.exit()?;

            let mut cmd = Command::new("ssh-keygen");
            cmd.arg("-t").arg(&req.key_type);
            if let Some(bits) = &req.bits {
                cmd.arg("-b").arg(bits);
            }
            cmd.arg("-f").arg(&req.filename);
            cmd.arg("-N").arg(&req.passphrase);

            let status = cmd.status()?;

            if !status.success() {
                let mut screen = crate::app::screen::keygen_screen::KeygenScreen::default();
                screen.error = Some(format!("ssh-keygen failed with code {}", status.code().unwrap_or(-1)));
                app.current_screen = Screen::Keygen(screen);
            } else {
                app.current_screen = Screen::Intro(
                    crate::app::screen::intro_screen::IntroScreen::default(),
                );
            }

            tui.enter()?;
        }

        // Port forward execution
        if let Screen::PortForwardExecute(req) = &app.current_screen {
            let req = req.clone();
            tui.exit()?;

            let mut cmd = Command::new("ssh");

            let forward_spec = format!(
                "{}:{}:{}",
                req.local_port, req.remote_host, req.remote_port
            );
            cmd.arg(format!("-{}", req.direction)).arg(&forward_spec);
            cmd.arg("-N"); // no remote command

            // Use the host alias if available (lets SSH config handle auth/port)
            cmd.arg(&req.host.host);

            let status = cmd.status()?;

            if !status.success() {
                let mut screen = crate::app::screen::portforward_screen::PortForwardScreen::new(req.host.clone());
                screen.error = Some("Port forwarding ended or failed".to_string());
                app.current_screen = Screen::PortForward(screen);
            } else {
                app.current_screen = Screen::Connect(
                    crate::app::screen::connect_screen::ConnectScreen::default(),
                );
            }

            tui.enter()?;
        }

        // Quick command execution
        if let Screen::QuickCommandExecute(req) = &app.current_screen {
            let req = req.clone();

            let mut cmd = Command::new("ssh");

            if let Some(user) = &req.host.username {
                if !user.is_empty() {
                    cmd.arg("-l").arg(user);
                }
            }

            if let Some(port) = req.host.port {
                cmd.arg("-p").arg(port.to_string());
            }

            if let Some(AuthMethod::Key(key_path)) = &req.host.auth {
                cmd.arg("-i").arg(key_path);
            }

            cmd.arg(&req.host.hostname);
            cmd.arg(&req.command);

            tui.exit()?;
            let output = cmd.output()?;
            tui.enter()?;

            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let combined = if stderr.is_empty() {
                stdout
            } else {
                format!("{stdout}\n--- stderr ---\n{stderr}")
            };

            app.current_screen = Screen::CommandOutput(
                crate::app::screen::command_output_screen::CommandOutputScreen::new(combined),
            );
        }

        // SCP transfer execution
        if let Screen::TransferExecute(req) = &app.current_screen {
            let req = req.clone();
            tui.exit()?;

            let mut cmd = Command::new("scp");

            if let Some(port) = req.host.port {
                cmd.arg("-P").arg(port.to_string());
            }

            if let Some(AuthMethod::Key(key_path)) = &req.host.auth {
                cmd.arg("-i").arg(key_path);
            }

            let user_prefix = req.host.username.as_deref().unwrap_or("");
            let remote = if user_prefix.is_empty() {
                format!("{}:{}", req.host.hostname, req.remote_path)
            } else {
                format!("{}@{}:{}", user_prefix, req.host.hostname, req.remote_path)
            };

            if req.direction == "upload" {
                cmd.arg(&req.local_path).arg(&remote);
            } else {
                cmd.arg(&remote).arg(&req.local_path);
            }

            let status = cmd.status()?;

            if !status.success() {
                let mut screen = crate::app::screen::transfer_screen::TransferScreen::new(req.host.clone());
                screen.error = Some(format!("scp failed with code {}", status.code().unwrap_or(-1)));
                app.current_screen = Screen::Transfer(screen);
            } else {
                app.current_screen = Screen::Connect(
                    crate::app::screen::connect_screen::ConnectScreen::default(),
                );
            }

            tui.enter()?;
        }
    }

    tui.exit()?;
    Ok(())
}
