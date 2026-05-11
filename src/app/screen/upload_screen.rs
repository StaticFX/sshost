use crate::{
    app::screen::{
        SCREEN_HEIGHT_PERCENTAGE, SCREEN_WIDTH_PERCENTAGE, Screen, intro_screen::IntroScreen,
    },
    ui::form_field::render_field,
};
use crossterm::event::KeyCode;
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use std::env::home_dir;

fn find_ssh_pub_keys() -> Vec<String> {
    let ssh_dir = home_dir().unwrap().join(".ssh");
    let mut keys = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&ssh_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.ends_with(".pub") {
                    keys.push(format!("~/.ssh/{}", name));
                }
            }
        }
    }
    keys.sort();
    keys
}

#[derive(Debug, Default, PartialEq)]
pub enum Field {
    #[default]
    KeyPath,
    Hostname,
    Port,
    User,
    Password,
}

impl Field {
    fn next(&self) -> Field {
        match self {
            Field::KeyPath => Field::Hostname,
            Field::Hostname => Field::Port,
            Field::Port => Field::User,
            Field::User => Field::Password,
            Field::Password => Field::KeyPath,
        }
    }
    fn prev(&self) -> Field {
        match self {
            Field::KeyPath => Field::Password,
            Field::Hostname => Field::KeyPath,
            Field::Port => Field::Hostname,
            Field::User => Field::Port,
            Field::Password => Field::User,
        }
    }
}

#[derive(Debug)]
pub struct UploadScreen {
    pub key_path: String,
    pub hostname: String,
    pub port: String,
    pub user: String,
    pub password: String,
    pub focused: Field,
    pub error: Option<String>,
    pub available_keys: Vec<String>,
    pub key_index: usize,
}

impl Default for UploadScreen {
    fn default() -> Self {
        let available_keys = find_ssh_pub_keys();
        let key_path = available_keys
            .first()
            .cloned()
            .unwrap_or_else(|| "~/.ssh/id_rsa.pub".to_string());
        Self {
            key_path,
            hostname: String::new(),
            port: String::new(),
            user: String::new(),
            password: String::new(),
            focused: Field::default(),
            error: None,
            available_keys,
            key_index: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UploadRequest {
    pub key_path: String,
    pub hostname: String,
    pub port: Option<u16>,
    pub user: Option<String>,
    pub password: Option<String>,
}

impl UploadScreen {
    fn active_field_mut(&mut self) -> &mut String {
        match self.focused {
            Field::KeyPath => &mut self.key_path,
            Field::Hostname => &mut self.hostname,
            Field::Port => &mut self.port,
            Field::User => &mut self.user,
            Field::Password => &mut self.password,
        }
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
            .title(" Upload SSH Key ")
            .title_style(Style::default().fg(Color::White))
            .borders(Borders::ALL)
            .border_style(Style::new().dark_gray());
        let inner = outer.inner(all);
        frame.render_widget(outer, all);

        let chunks = Layout::vertical([
            Constraint::Length(2), // header
            Constraint::Length(3), // key path
            Constraint::Length(3), // hostname
            Constraint::Length(3), // port
            Constraint::Length(3), // user
            Constraint::Length(3), // password
            Constraint::Fill(1),
            Constraint::Length(1), // error
        ])
        .split(inner);

        // Header
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("↑↓/Tab", Style::default().fg(Color::Yellow)),
                Span::styled(" navigate  ", Style::default().fg(Color::DarkGray)),
                Span::styled("←→", Style::default().fg(Color::Yellow)),
                Span::styled(" key  ", Style::default().fg(Color::DarkGray)),
                Span::styled("↵", Style::default().fg(Color::Yellow)),
                Span::styled(" upload  ", Style::default().fg(Color::DarkGray)),
                Span::styled("Esc", Style::default().fg(Color::Yellow)),
                Span::styled(" back", Style::default().fg(Color::DarkGray)),
            ]))
            .centered(),
            chunks[0],
        );

        // Public key field with left/right selection
        let key_display = if self.focused == Field::KeyPath && !self.available_keys.is_empty() {
            format!("\u{25C0} {} \u{25B6}", self.key_path)
        } else {
            self.key_path.clone()
        };
        frame.render_widget(
            render_field(
                "Public Key",
                &key_display,
                self.focused == Field::KeyPath,
                "~/.ssh/id_rsa.pub",
                false,
            ),
            chunks[1],
        );
        frame.render_widget(
            render_field(
                "Hostname",
                &self.hostname,
                self.focused == Field::Hostname,
                "192.168.1.1",
                true,
            ),
            chunks[2],
        );
        frame.render_widget(
            render_field(
                "Port",
                &self.port,
                self.focused == Field::Port,
                "22",
                false,
            ),
            chunks[3],
        );
        frame.render_widget(
            render_field(
                "User",
                &self.user,
                self.focused == Field::User,
                "root",
                false,
            ),
            chunks[4],
        );

        // Password field (masked)
        let masked: String = "*".repeat(self.password.len());
        frame.render_widget(
            render_field(
                "Password",
                &masked,
                self.focused == Field::Password,
                "",
                true,
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

    pub fn handle_key(&mut self, key_code: KeyCode) -> Option<Screen> {
        match key_code {
            KeyCode::Esc => return Some(Screen::Intro(IntroScreen::default())),

            KeyCode::Tab | KeyCode::Down => self.focused = self.focused.next(),
            KeyCode::BackTab | KeyCode::Up => self.focused = self.focused.prev(),

            KeyCode::Left => {
                if self.focused == Field::KeyPath && !self.available_keys.is_empty() {
                    if self.key_index == 0 {
                        self.key_index = self.available_keys.len() - 1;
                    } else {
                        self.key_index -= 1;
                    }
                    self.key_path = self.available_keys[self.key_index].clone();
                }
            }
            KeyCode::Right => {
                if self.focused == Field::KeyPath && !self.available_keys.is_empty() {
                    self.key_index = (self.key_index + 1) % self.available_keys.len();
                    self.key_path = self.available_keys[self.key_index].clone();
                }
            }

            KeyCode::Backspace => {
                self.active_field_mut().pop();
            }

            KeyCode::Char(c) => {
                self.active_field_mut().push(c);
            }

            KeyCode::Enter => {
                if self.hostname.is_empty() {
                    self.error = Some("Hostname is required".into());
                } else if self.password.is_empty() {
                    self.error = Some("Password is required for authentication".into());
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

                    let key_path = if self.key_path.is_empty() {
                        "~/.ssh/id_rsa.pub".to_string()
                    } else {
                        self.key_path.clone()
                    };

                    let user = if self.user.is_empty() {
                        None
                    } else {
                        Some(self.user.clone())
                    };

                    return Some(Screen::UploadExecute(UploadRequest {
                        key_path,
                        hostname: self.hostname.clone(),
                        port,
                        user,
                        password: Some(self.password.clone()),
                    }));
                }
            }

            _ => {}
        }
        None
    }
}
