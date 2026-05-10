use crate::app::screen::Screen;

pub mod screen;

#[derive(Debug, Default)]
pub struct App {
    pub should_quit: bool,
    pub current_screen: Screen,
}

impl App {
    pub fn tick(&self) {}

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}
