use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    app::{App, screen::Screen},
    ssh_config::config_reader::SSHConfig,
};

pub fn update(app: &mut App, key_event: KeyEvent) {
    if key_event.code == KeyCode::Char('q')
        && !matches!(
            app.current_screen,
            Screen::Configure(_) | Screen::Upload(_) | Screen::Keygen(_)
        )
    {
        app.quit();
        return;
    }

    let ssh_callback = |config: SSHConfig| app.current_ssh = Some(config);

    let optional_screen = match &mut app.current_screen {
        Screen::Intro(s) => s.match_key(key_event.code),
        Screen::Configure(s) => s.match_key(key_event.code),
        Screen::Connect(s) => s.match_key(key_event.code, ssh_callback),
        Screen::Upload(s) => s.handle_key(key_event.code),
        Screen::UploadExecute(_) => None,
        Screen::Keygen(s) => s.handle_key(key_event.code),
        Screen::KeygenExecute(_) => None,
        Screen::KnownHosts(s) => s.match_key(key_event.code),
    };

    if let Some(new_screen) = optional_screen {
        app.current_screen = new_screen;
    }
}
