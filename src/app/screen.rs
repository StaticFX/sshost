use crate::app::screen::{
    command_output_screen::CommandOutputScreen,
    configure_screen::ConfigureScreen,
    connect_screen::ConnectScreen,
    import_screen::ImportScreen,
    intro_screen::IntroScreen,
    keygen_screen::{KeygenRequest, KeygenScreen},
    portforward_screen::{PortForwardRequest, PortForwardScreen},
    quick_command_screen::{QuickCommandRequest, QuickCommandScreen},
    transfer_screen::{TransferRequest, TransferScreen},
    upload_screen::{UploadRequest, UploadScreen},
};

pub mod command_output_screen;
pub mod configure_screen;
pub mod connect_screen;
pub mod import_screen;
pub mod intro_screen;
pub mod keygen_screen;
pub mod portforward_screen;
pub mod quick_command_screen;
pub mod transfer_screen;
pub mod upload_screen;

pub const SCREEN_WIDTH_PERCENTAGE: u16 = 80;
pub const SCREEN_HEIGHT_PERCENTAGE: u16 = 90;

#[derive(Debug)]
pub enum Screen {
    Intro(IntroScreen),
    Connect(ConnectScreen),
    Configure(ConfigureScreen),
    Upload(UploadScreen),
    UploadExecute(UploadRequest),
    Keygen(KeygenScreen),
    KeygenExecute(KeygenRequest),
    PortForward(PortForwardScreen),
    PortForwardExecute(PortForwardRequest),
    Transfer(TransferScreen),
    TransferExecute(TransferRequest),
    Import(ImportScreen),
    QuickCommand(QuickCommandScreen),
    QuickCommandExecute(QuickCommandRequest),
    CommandOutput(CommandOutputScreen),
}

impl Default for Screen {
    fn default() -> Self {
        Screen::Intro(IntroScreen::default())
    }
}
