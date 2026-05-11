use crate::app::screen::{
    configure_screen::ConfigureScreen,
    connect_screen::ConnectScreen,
    intro_screen::IntroScreen,
    keygen_screen::{KeygenRequest, KeygenScreen},
    known_hosts_screen::KnownHostsScreen,
    upload_screen::{UploadRequest, UploadScreen},
};

pub mod configure_screen;
pub mod connect_screen;
pub mod intro_screen;
pub mod keygen_screen;
pub mod known_hosts_screen;
pub mod upload_screen;

pub const SCREEN_WIDTH_PERCENTAGE: u16 = 90;
pub const SCREEN_HEIGHT_PERCENTAGE: u16 = 95;

#[derive(Debug)]
pub enum Screen {
    Intro(IntroScreen),
    Connect(ConnectScreen),
    Configure(ConfigureScreen),
    Upload(UploadScreen),
    UploadExecute(UploadRequest),
    Keygen(KeygenScreen),
    KeygenExecute(KeygenRequest),
    KnownHosts(KnownHostsScreen),
}

impl Default for Screen {
    fn default() -> Self {
        Screen::Intro(IntroScreen::default())
    }
}
