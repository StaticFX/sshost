use ratatui::Frame;

use crate::app::{App, screen::Screen};

pub fn render(app: &mut App, frame: &mut Frame) {
    match &app.current_screen {
        Screen::Intro(s) => s.draw(app, frame),
    }
}
