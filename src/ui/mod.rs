pub mod form_field;

use ratatui::Frame;

use crate::app::{App, screen::Screen};

pub fn render(app: &mut App, frame: &mut Frame) {
    let screen = &mut app.current_screen;

    match screen {
        Screen::Intro(s) => s.draw(frame),
        Screen::Configure(s) => s.draw(frame),
        Screen::Connect(s) => s.draw(app.ssh_status.clone(), frame),
        Screen::Upload(s) => s.draw(frame),
        Screen::UploadExecute(_) => {}
        Screen::Keygen(s) => s.draw(frame),
        Screen::KeygenExecute(_) => {}
        Screen::KnownHosts(s) => s.draw(frame),
    }
}
