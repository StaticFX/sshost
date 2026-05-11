use crate::{
    app::screen::{
        SCREEN_HEIGHT_PERCENTAGE, SCREEN_WIDTH_PERCENTAGE, Screen,
        connect_screen::ConnectScreen,
    },
    ssh_config::config_reader::SSHConfig,
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

#[derive(Debug, Clone)]
pub struct TransferRequest {
    pub host: SSHConfig,
    pub direction: String, // "upload" or "download"
    pub local_path: String,
    pub remote_path: String,
}

#[derive(Debug, Default, PartialEq)]
pub enum Field {
    #[default]
    Direction,
    LocalPath,
    RemotePath,
}

impl Field {
    fn next(&self) -> Field {
        match self {
            Field::Direction => Field::LocalPath,
            Field::LocalPath => Field::RemotePath,
            Field::RemotePath => Field::Direction,
        }
    }
}

const DIRECTIONS: [&str; 2] = ["Upload", "Download"];

#[derive(Debug)]
pub struct TransferScreen {
    pub host: SSHConfig,
    pub direction: usize,
    pub local_path: String,
    pub remote_path: String,
    pub focused: Field,
    pub error: Option<String>,
}

impl TransferScreen {
    pub fn new(host: SSHConfig) -> Self {
        Self {
            host,
            direction: 0,
            local_path: String::new(),
            remote_path: String::new(),
            focused: Field::default(),
            error: None,
        }
    }

    fn active_field_mut(&mut self) -> Option<&mut String> {
        match self.focused {
            Field::Direction => None,
            Field::LocalPath => Some(&mut self.local_path),
            Field::RemotePath => Some(&mut self.remote_path),
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
            .title(format!(" SCP Transfer - {} ", self.host.host))
            .title_style(Style::default().fg(Color::White))
            .borders(Borders::ALL)
            .border_style(Style::new().dark_gray());
        let inner = outer.inner(all);
        frame.render_widget(outer, all);

        let chunks = Layout::vertical([
            Constraint::Length(2), // header
            Constraint::Length(3), // direction
            Constraint::Length(3), // local path
            Constraint::Length(3), // remote path
            Constraint::Fill(1),
            Constraint::Length(1), // error
        ])
        .split(inner);

        // Header
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("\u{2191}\u{2193}/Tab", Style::default().fg(Color::Yellow)),
                Span::styled(" navigate  ", Style::default().fg(Color::DarkGray)),
                Span::styled("\u{21b5}", Style::default().fg(Color::Yellow)),
                Span::styled(" transfer  ", Style::default().fg(Color::DarkGray)),
                Span::styled("Esc", Style::default().fg(Color::Yellow)),
                Span::styled(" back", Style::default().fg(Color::DarkGray)),
            ]))
            .centered(),
            chunks[0],
        );

        // Direction field
        let dir_focused = self.focused == Field::Direction;
        let dir_border = if dir_focused {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let dir_text = format!("\u{25c0} {} \u{25b6}", DIRECTIONS[self.direction]);
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                dir_text,
                Style::default().fg(Color::White),
            )))
            .block(
                Block::default()
                    .title(" Direction ")
                    .borders(Borders::ALL)
                    .border_style(dir_border),
            ),
            chunks[1],
        );

        // Local path
        frame.render_widget(
            render_field(
                "Local Path",
                &self.local_path,
                self.focused == Field::LocalPath,
                "/path/to/local/file",
                true,
            ),
            chunks[2],
        );

        // Remote path
        frame.render_widget(
            render_field(
                "Remote Path",
                &self.remote_path,
                self.focused == Field::RemotePath,
                "/path/to/remote/file",
                true,
            ),
            chunks[3],
        );

        // Error
        frame.render_widget(
            Paragraph::new(self.error.as_deref().unwrap_or(""))
                .style(Style::default().fg(Color::Red))
                .centered(),
            chunks[5],
        );
    }

    pub fn handle_key(&mut self, key_code: KeyCode) -> Option<Screen> {
        match key_code {
            KeyCode::Esc => {
                return Some(Screen::Connect(ConnectScreen::default()));
            }

            KeyCode::Tab | KeyCode::Down => self.focused = self.focused.next(),

            KeyCode::Left => {
                if self.focused == Field::Direction {
                    if self.direction == 0 {
                        self.direction = DIRECTIONS.len() - 1;
                    } else {
                        self.direction -= 1;
                    }
                }
            }

            KeyCode::Right => {
                if self.focused == Field::Direction {
                    self.direction = (self.direction + 1) % DIRECTIONS.len();
                }
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
                if self.local_path.is_empty() {
                    self.error = Some("Local path is required".into());
                } else if self.remote_path.is_empty() {
                    self.error = Some("Remote path is required".into());
                } else {
                    let direction = if self.direction == 0 {
                        "upload".to_string()
                    } else {
                        "download".to_string()
                    };

                    return Some(Screen::TransferExecute(TransferRequest {
                        host: self.host.clone(),
                        direction,
                        local_path: self.local_path.clone(),
                        remote_path: self.remote_path.clone(),
                    }));
                }
            }

            _ => {}
        }
        None
    }
}
