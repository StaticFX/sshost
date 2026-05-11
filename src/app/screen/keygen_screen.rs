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

const KEY_TYPES: [&str; 3] = ["ed25519", "rsa", "ecdsa"];

fn default_filename(key_type_index: usize) -> String {
    match key_type_index {
        0 => "~/.ssh/id_ed25519".to_string(),
        1 => "~/.ssh/id_rsa".to_string(),
        2 => "~/.ssh/id_ecdsa".to_string(),
        _ => "~/.ssh/id_ed25519".to_string(),
    }
}

fn expand_tilde(path: &str) -> String {
    if path.starts_with("~/") {
        if let Some(home) = home_dir() {
            return format!("{}{}", home.display(), &path[1..]);
        }
    }
    path.to_string()
}

fn key_file_exists(path: &str) -> bool {
    let expanded = expand_tilde(path);
    std::path::Path::new(&expanded).exists()
}

#[derive(Debug, Default, PartialEq)]
pub enum Field {
    #[default]
    KeyType,
    Bits,
    Passphrase,
    Filename,
}

impl Field {
    fn next(&self) -> Field {
        match self {
            Field::KeyType => Field::Bits,
            Field::Bits => Field::Passphrase,
            Field::Passphrase => Field::Filename,
            Field::Filename => Field::KeyType,
        }
    }
    fn prev(&self) -> Field {
        match self {
            Field::KeyType => Field::Filename,
            Field::Bits => Field::KeyType,
            Field::Passphrase => Field::Bits,
            Field::Filename => Field::Passphrase,
        }
    }
}

#[derive(Debug)]
pub struct KeygenScreen {
    pub key_type: usize,
    pub bits: String,
    pub passphrase: String,
    pub filename: String,
    pub focused: Field,
    pub error: Option<String>,
    pub confirm_overwrite: bool,
}

impl Default for KeygenScreen {
    fn default() -> Self {
        Self {
            key_type: 0,
            bits: "4096".to_string(),
            passphrase: String::new(),
            filename: "~/.ssh/id_ed25519".to_string(),
            focused: Field::default(),
            error: None,
            confirm_overwrite: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct KeygenRequest {
    pub key_type: String,
    pub bits: Option<String>,
    pub passphrase: String,
    pub filename: String,
}

impl KeygenScreen {
    fn active_field_mut(&mut self) -> Option<&mut String> {
        match self.focused {
            Field::KeyType => None,
            Field::Bits => Some(&mut self.bits),
            Field::Passphrase => Some(&mut self.passphrase),
            Field::Filename => Some(&mut self.filename),
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
            .title(" Generate SSH Key ")
            .title_style(Style::default().fg(Color::White))
            .borders(Borders::ALL)
            .border_style(Style::new().dark_gray());
        let inner = outer.inner(all);
        frame.render_widget(outer, all);

        let chunks = Layout::vertical([
            Constraint::Length(2), // header
            Constraint::Length(3), // key type
            Constraint::Length(3), // bits
            Constraint::Length(3), // passphrase
            Constraint::Length(3), // filename
            Constraint::Fill(1),
            Constraint::Length(1), // error
        ])
        .split(inner);

        // Header
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("\u{2191}\u{2193}/Tab", Style::default().fg(Color::Yellow)),
                Span::styled(" navigate  ", Style::default().fg(Color::DarkGray)),
                Span::styled("\u{25c0}\u{25b6}", Style::default().fg(Color::Yellow)),
                Span::styled(" type  ", Style::default().fg(Color::DarkGray)),
                Span::styled("\u{21b5}", Style::default().fg(Color::Yellow)),
                Span::styled(" generate  ", Style::default().fg(Color::DarkGray)),
                Span::styled("Esc", Style::default().fg(Color::Yellow)),
                Span::styled(" back", Style::default().fg(Color::DarkGray)),
            ]))
            .centered(),
            chunks[0],
        );

        // Key type field (special rendering with arrows)
        let key_type_focused = self.focused == Field::KeyType;
        let key_type_border = if key_type_focused {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let key_type_display = format!("\u{25c0} {} \u{25b6}", KEY_TYPES[self.key_type]);
        let key_type_style = if key_type_focused {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::White)
        };
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(key_type_display, key_type_style))).block(
                Block::default()
                    .title(" Key Type ")
                    .borders(Borders::ALL)
                    .border_style(key_type_border),
            ),
            chunks[1],
        );

        // Bits field
        frame.render_widget(
            render_field(
                "Bits (RSA only)",
                &self.bits,
                self.focused == Field::Bits,
                "4096",
                false,
            ),
            chunks[2],
        );

        // Passphrase field (masked)
        let masked: String = "*".repeat(self.passphrase.len());
        frame.render_widget(
            render_field(
                "Passphrase",
                &masked,
                self.focused == Field::Passphrase,
                "",
                false,
            ),
            chunks[3],
        );

        // Filename field
        frame.render_widget(
            render_field(
                "Filename",
                &self.filename,
                self.focused == Field::Filename,
                &default_filename(self.key_type),
                false,
            ),
            chunks[4],
        );

        // Error / warning
        let msg = self.error.as_deref().unwrap_or("");
        let color = if self.confirm_overwrite {
            Color::Yellow
        } else {
            Color::Red
        };
        frame.render_widget(
            Paragraph::new(msg)
                .style(Style::default().fg(color))
                .centered(),
            chunks[6],
        );
    }

    pub fn handle_key(&mut self, key_code: KeyCode) -> Option<Screen> {
        // Handle overwrite confirmation
        if self.confirm_overwrite {
            match key_code {
                KeyCode::Char('y') => {
                    self.confirm_overwrite = false;
                    self.error = None;
                    return self.build_request();
                }
                KeyCode::Char('n') | KeyCode::Esc => {
                    self.confirm_overwrite = false;
                    self.error = None;
                    if key_code == KeyCode::Esc {
                        return Some(Screen::Intro(IntroScreen::default()));
                    }
                    return None;
                }
                _ => return None,
            }
        }

        match key_code {
            KeyCode::Esc => return Some(Screen::Intro(IntroScreen::default())),

            KeyCode::Tab | KeyCode::Down => self.focused = self.focused.next(),
            KeyCode::BackTab | KeyCode::Up => self.focused = self.focused.prev(),

            KeyCode::Left if self.focused == Field::KeyType => {
                if self.key_type > 0 {
                    self.key_type -= 1;
                } else {
                    self.key_type = KEY_TYPES.len() - 1;
                }
                self.filename = default_filename(self.key_type);
            }

            KeyCode::Right if self.focused == Field::KeyType => {
                if self.key_type < KEY_TYPES.len() - 1 {
                    self.key_type += 1;
                } else {
                    self.key_type = 0;
                }
                self.filename = default_filename(self.key_type);
            }

            KeyCode::Backspace => {
                if let Some(field) = self.active_field_mut() {
                    field.pop();
                }
            }

            KeyCode::Char(c) => {
                if let Some(field) = self.active_field_mut() {
                    field.push(c);
                }
            }

            KeyCode::Enter => {
                let filename = if self.filename.is_empty() {
                    default_filename(self.key_type)
                } else {
                    self.filename.clone()
                };

                // Check if key file already exists
                if key_file_exists(&filename) {
                    self.confirm_overwrite = true;
                    self.error = Some(format!("'{}' already exists. Overwrite? y/n", filename));
                    return None;
                }

                return self.build_request();
            }

            _ => {}
        }
        None
    }

    fn build_request(&self) -> Option<Screen> {
        let filename = if self.filename.is_empty() {
            default_filename(self.key_type)
        } else {
            self.filename.clone()
        };

        let key_type_str = KEY_TYPES[self.key_type].to_string();

        let bits = if key_type_str == "rsa" {
            let b = if self.bits.is_empty() {
                "4096".to_string()
            } else {
                self.bits.clone()
            };
            match b.parse::<u32>() {
                Ok(_) => Some(b),
                Err(_) => {
                    return None;
                }
            }
        } else {
            None
        };

        Some(Screen::KeygenExecute(KeygenRequest {
            key_type: key_type_str,
            bits,
            passphrase: self.passphrase.clone(),
            filename,
        }))
    }
}
