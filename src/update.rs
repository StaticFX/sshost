use crossterm::event::KeyEvent;

use crate::app::{App, screen::Screen};

pub fn update(app: &mut App, key_event: KeyEvent) {
    let ssh_callback = |host: &str| app.current_ssh = Some(String::from(host));

    let optional_screen = match &mut app.current_screen {
        Screen::Intro(s) => s.match_key(key_event.code),
        Screen::Configure(s) => s.match_key(key_event.code),
        Screen::Connect(s) => s.match_key(key_event.code, ssh_callback),
    };

    if let Some(new_screen) = optional_screen {
        app.current_screen = new_screen;
    }
}
