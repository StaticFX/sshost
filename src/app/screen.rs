use crate::app::screen::{
    configure_screen::ConfigureScreen, connect_screen::ConnectScreen, intro_screen::IntroScreen,
};

pub mod configure_screen;
pub mod connect_screen;
pub mod intro_screen;

pub const SCREEN_WIDTH_PERCENTAGE: u16 = 80;
pub const SCREEN_HEIGHT_PERCENTAGE: u16 = 90;

#[derive(Debug)]
pub enum Screen {
    Intro(IntroScreen),
    Connect(ConnectScreen),
    Configure(ConfigureScreen),
}

impl Default for Screen {
    fn default() -> Self {
        Screen::Intro(IntroScreen::default())
    }
}
