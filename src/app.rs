use crate::{app::screen::Screen, ssh_config::config_reader::SSHConfig};

pub mod screen;

#[derive(Debug, Default)]
pub struct App {
    pub should_quit: bool,
    pub current_screen: Screen,
    pub current_ssh: Option<SSHConfig>,
    pub ssh_status: Option<String>,
}

impl App {
    pub fn tick(&self) {}

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}
