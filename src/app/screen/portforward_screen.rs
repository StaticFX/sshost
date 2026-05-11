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

#[derive(Debug, Default, PartialEq)]
pub enum Field {
    #[default]
    Direction,
    LocalPort,
    RemoteHost,
    RemotePort,
}

impl Field {
    fn next(&self) -> Field {
        match self {
            Field::Direction => Field::LocalPort,
            Field::LocalPort => Field::RemoteHost,
            Field::RemoteHost => Field::RemotePort,
            Field::RemotePort => Field::Direction,
        }
    }
    fn prev(&self) -> Field {
        match self {
            Field::Direction => Field::RemotePort,
            Field::LocalPort => Field::Direction,
            Field::RemoteHost => Field::LocalPort,
            Field::RemotePort => Field::RemoteHost,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PortForwardRequest {
    pub host: SSHConfig,
    pub direction: String, // "L" or "R"
    pub local_port: String,
    pub remote_host: String,
    pub remote_port: String,
}

#[derive(Debug)]
pub struct PortForwardScreen {
    pub host: SSHConfig,
    pub direction: usize, // 0 = Local (-L), 1 = Remote (-R)
    pub local_port: String,
    pub remote_host: String,
    pub remote_port: String,
    pub focused: Field,
    pub error: Option<String>,
}

impl PortForwardScreen {
    pub fn new(host: SSHConfig) -> Self {
        Self {
            host,
            direction: 0,
            local_port: String::new(),
            remote_host: "localhost".to_string(),
            remote_port: String::new(),
            focused: Field::default(),
            error: None,
        }
    }

    fn active_field_mut(&mut self) -> Option<&mut String> {
        match self.focused {
            Field::Direction => None,
            Field::LocalPort => Some(&mut self.local_port),
            Field::RemoteHost => Some(&mut self.remote_host),
            Field::RemotePort => Some(&mut self.remote_port),
        }
    }

    pub fn handle_key(&mut self, key_code: KeyCode) -> Option<Screen> {
        match key_code {
            KeyCode::Esc => return Some(Screen::Connect(ConnectScreen::default())),

            KeyCode::Tab | KeyCode::Down => self.focused = self.focused.next(),
            KeyCode::BackTab | KeyCode::Up => self.focused = self.focused.prev(),

            KeyCode::Left => {
                if self.focused == Field::Direction {
                    self.direction = if self.direction == 0 { 1 } else { 0 };
                }
            }
            KeyCode::Right => {
                if self.focused == Field::Direction {
                    self.direction = if self.direction == 0 { 1 } else { 0 };
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
                if self.local_port.is_empty() {
                    self.error = Some("Local port is required".into());
                } else if self.remote_port.is_empty() {
                    self.error = Some("Remote port is required".into());
                } else if self.local_port.parse::<u16>().is_err() {
                    self.error = Some("Local port must be a valid number".into());
                } else if self.remote_port.parse::<u16>().is_err() {
                    self.error = Some("Remote port must be a valid number".into());
                } else {
                    let direction = if self.direction == 0 {
                        "L".to_string()
                    } else {
                        "R".to_string()
                    };
                    return Some(Screen::PortForwardExecute(PortForwardRequest {
                        host: self.host.clone(),
                        direction,
                        local_port: self.local_port.clone(),
                        remote_host: if self.remote_host.is_empty() {
                            "localhost".to_string()
                        } else {
                            self.remote_host.clone()
                        },
                        remote_port: self.remote_port.clone(),
                    }));
                }
            }

            _ => {}
        }
        None
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
            .title(" Port Forwarding ")
            .title_style(Style::default().fg(Color::White))
            .borders(Borders::ALL)
            .border_style(Style::new().dark_gray());
        let inner = outer.inner(all);
        frame.render_widget(outer, all);

        let chunks = Layout::vertical([
            Constraint::Length(2), // header
            Constraint::Length(3), // direction
            Constraint::Length(3), // local port
            Constraint::Length(3), // remote host
            Constraint::Length(3), // remote port
            Constraint::Fill(1),
            Constraint::Length(1), // error
        ])
        .split(inner);

        // Header
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("Tab/↑↓", Style::default().fg(Color::Yellow)),
                Span::styled(" navigate  ", Style::default().fg(Color::DarkGray)),
                Span::styled("←→", Style::default().fg(Color::Yellow)),
                Span::styled(" direction  ", Style::default().fg(Color::DarkGray)),
                Span::styled("↵", Style::default().fg(Color::Yellow)),
                Span::styled(" forward  ", Style::default().fg(Color::DarkGray)),
                Span::styled("Esc", Style::default().fg(Color::Yellow)),
                Span::styled(" back", Style::default().fg(Color::DarkGray)),
            ]))
            .centered(),
            chunks[0],
        );

        // Direction field
        let direction_display = if self.direction == 0 {
            "\u{25C0} Local (-L) \u{25B6}"
        } else {
            "\u{25C0} Remote (-R) \u{25B6}"
        };
        frame.render_widget(
            render_field(
                "Direction",
                direction_display,
                self.focused == Field::Direction,
                "",
                false,
            ),
            chunks[1],
        );

        frame.render_widget(
            render_field(
                "Local Port",
                &self.local_port,
                self.focused == Field::LocalPort,
                "8080",
                true,
            ),
            chunks[2],
        );

        frame.render_widget(
            render_field(
                "Remote Host",
                &self.remote_host,
                self.focused == Field::RemoteHost,
                "localhost",
                false,
            ),
            chunks[3],
        );

        frame.render_widget(
            render_field(
                "Remote Port",
                &self.remote_port,
                self.focused == Field::RemotePort,
                "80",
                true,
            ),
            chunks[4],
        );

        // Error
        frame.render_widget(
            Paragraph::new(self.error.as_deref().unwrap_or(""))
                .style(Style::default().fg(Color::Red))
                .centered(),
            chunks[6],
        );
    }
}
