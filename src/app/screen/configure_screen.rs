use crate::{
    app::screen::{
        SCREEN_HEIGHT_PERCENTAGE, SCREEN_WIDTH_PERCENTAGE, Screen, intro_screen::IntroScreen,
    },
    ssh_config::config_reader::{self, SSHConfig},
};
use crossterm::event::KeyCode;
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

#[derive(Debug, Default, PartialEq)]
pub enum Field {
    #[default]
    Host,
    Hostname,
    Port,
    User,
    IdentityFile,
}

impl Field {
    fn next(&self) -> Field {
        match self {
            Field::Host => Field::Hostname,
            Field::Hostname => Field::Port,
            Field::Port => Field::User,
            Field::User => Field::IdentityFile,
            Field::IdentityFile => Field::Host,
        }
    }
    fn prev(&self) -> Field {
        match self {
            Field::Host => Field::IdentityFile,
            Field::Hostname => Field::Host,
            Field::Port => Field::Hostname,
            Field::User => Field::Port,
            Field::IdentityFile => Field::User,
        }
    }
}

#[derive(Debug)]
pub struct ConfigureScreen {
    pub host: String,
    pub hostname: String,
    pub port: String,
    pub user: String,
    pub identity_file: String,
    pub focused: Field,
    pub error: Option<String>,
}

impl Default for ConfigureScreen {
    fn default() -> Self {
        Self {
            host: String::new(),
            hostname: String::new(),
            port: String::new(),
            user: String::new(),
            identity_file: "~/.ssh/id_rsa".to_string(),
            focused: Field::default(),
            error: None,
        }
    }
}

impl ConfigureScreen {
    fn active_field_mut(&mut self) -> &mut String {
        match self.focused {
            Field::Host => &mut self.host,
            Field::Hostname => &mut self.hostname,
            Field::Port => &mut self.port,
            Field::User => &mut self.user,
            Field::IdentityFile => &mut self.identity_file,
        }
    }

    fn render_field<'a>(
        label: &'a str,
        value: &'a str,
        is_focused: bool,
        placeholder: &'a str,
        required: bool,
    ) -> Paragraph<'a> {
        let border_style = if is_focused {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let display: String = if value.is_empty() {
            if is_focused {
                "█".to_string()
            } else {
                placeholder.to_string()
            }
        } else if is_focused {
            format!("{value}█")
        } else {
            value.to_string()
        };

        let span_style = if value.is_empty() && !is_focused {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default().fg(Color::White)
        };

        let title = if required {
            format!(" {label} * ")
        } else {
            format!(" {label} ")
        };

        Paragraph::new(Line::from(Span::styled(display, span_style))).block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(border_style),
        )
    }

    pub fn draw(&self, frame: &mut Frame) {
        let area = frame.area();

        let vertical = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Percentage(SCREEN_HEIGHT_PERCENTAGE),
            Constraint::Fill(1),
        ])
        .split(area);

        let horizontal = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Percentage(SCREEN_WIDTH_PERCENTAGE),
            Constraint::Fill(1),
        ])
        .split(vertical[1]);

        let all = horizontal[1];

        let outer = Block::default()
            .title(" Configure Connection ")
            .title_style(Style::default().fg(Color::White))
            .borders(Borders::ALL)
            .border_style(Style::new().dark_gray());
        let inner = outer.inner(all);
        frame.render_widget(outer, all);

        let chunks = Layout::vertical([
            Constraint::Length(2), // header
            Constraint::Length(3), // host
            Constraint::Length(3), // hostname
            Constraint::Length(3), // port
            Constraint::Length(3), // user
            Constraint::Length(3), // identity file
            Constraint::Fill(1),
            Constraint::Length(1), // error
        ])
        .split(inner);

        // Header
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("↑↓/Tab", Style::default().fg(Color::Yellow)),
                Span::styled(" navigate  ", Style::default().fg(Color::DarkGray)),
                Span::styled("↵", Style::default().fg(Color::Yellow)),
                Span::styled(" save  ", Style::default().fg(Color::DarkGray)),
                Span::styled("Esc", Style::default().fg(Color::Yellow)),
                Span::styled(" back", Style::default().fg(Color::DarkGray)),
            ]))
            .centered(),
            chunks[0],
        );

        frame.render_widget(
            Self::render_field(
                "Host",
                &self.host,
                self.focused == Field::Host,
                "my-server",
                true,
            ),
            chunks[1],
        );
        frame.render_widget(
            Self::render_field(
                "Hostname",
                &self.hostname,
                self.focused == Field::Hostname,
                "192.168.1.1",
                true,
            ),
            chunks[2],
        );
        frame.render_widget(
            Self::render_field(
                "Port",
                &self.port,
                self.focused == Field::Port,
                "22",
                false,
            ),
            chunks[3],
        );
        frame.render_widget(
            Self::render_field(
                "User",
                &self.user,
                self.focused == Field::User,
                "root",
                false,
            ),
            chunks[4],
        );
        frame.render_widget(
            Self::render_field(
                "IdentityFile",
                &self.identity_file,
                self.focused == Field::IdentityFile,
                "~/.ssh/id_rsa",
                false,
            ),
            chunks[5],
        );

        // Error
        frame.render_widget(
            Paragraph::new(self.error.as_deref().unwrap_or(""))
                .style(Style::default().fg(Color::Red))
                .centered(),
            chunks[7],
        );
    }

    pub fn match_key(&mut self, key_code: KeyCode) -> Option<Screen> {
        match key_code {
            KeyCode::Esc => return Some(Screen::Intro(IntroScreen::default())),

            KeyCode::Tab | KeyCode::Down => self.focused = self.focused.next(),
            KeyCode::BackTab | KeyCode::Up => self.focused = self.focused.prev(),

            KeyCode::Backspace => {
                self.active_field_mut().pop();
            }

            KeyCode::Char(c) => {
                self.active_field_mut().push(c);
            }

            KeyCode::Enter => {
                if self.host.is_empty() || self.hostname.is_empty() {
                    self.error = Some("Host and Hostname are required".into());
                } else {
                    let port = if self.port.is_empty() {
                        None
                    } else {
                        match self.port.parse::<u16>() {
                            Ok(p) => Some(p),
                            Err(_) => {
                                self.error = Some("Port must be a number".into());
                                return None;
                            }
                        }
                    };

                    let username = if self.user.is_empty() {
                        None
                    } else {
                        Some(self.user.clone())
                    };

                    let auth = if self.identity_file.is_empty() {
                        None
                    } else {
                        Some(config_reader::AuthMethod::Key(self.identity_file.clone()))
                    };

                    let config = SSHConfig {
                        host: self.host.clone(),
                        hostname: self.hostname.clone(),
                        username,
                        port,
                        auth,
                    };

                    match config_reader::write_new_host(&config) {
                        Ok(_) => return Some(Screen::Intro(IntroScreen::default())),
                        Err(e) => self.error = Some(format!("Failed to write: {e}")),
                    }
                }
            }

            _ => {}
        }
        None
    }
}
