use ratatui::Frame;

use crate::app::{App, screen::intro_screen::IntroScreen};

pub mod intro_screen;

#[derive(Debug)]
pub enum Screen {
    Intro(IntroScreen),
}

impl Default for Screen {
    fn default() -> Self {
        Screen::Intro(IntroScreen::default())
    }
}
