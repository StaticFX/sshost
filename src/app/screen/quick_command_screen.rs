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
pub struct QuickCommandRequest {
    pub host: SSHConfig,
    pub command: String,
}

#[derive(Debug)]
pub struct QuickCommandScreen {
    pub host: SSHConfig,
    pub command: String,
    pub error: Option<String>,
}

impl QuickCommandScreen {
    pub fn new(host: SSHConfig) -> Self {
        Self {
            host,
            command: String::new(),
            error: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyCode) -> Option<Screen> {
        match key {
            KeyCode::Esc => Some(Screen::Connect(ConnectScreen::default())),
            KeyCode::Backspace => {
                self.command.pop();
                None
            }
            KeyCode::Char(c) => {
                self.command.push(c);
                None
            }
            KeyCode::Enter => {
                if self.command.is_empty() {
                    self.error = Some("Command cannot be empty".into());
                    None
                } else {
                    Some(Screen::QuickCommandExecute(QuickCommandRequest {
                        host: self.host.clone(),
                        command: self.command.clone(),
                    }))
                }
            }
            _ => None,
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
            .title(" Quick Command ")
            .title_style(Style::default().fg(Color::White))
            .borders(Borders::ALL)
            .border_style(Style::new().dark_gray());
        let inner = outer.inner(all);
        frame.render_widget(outer, all);

        let chunks = Layout::vertical([
            Constraint::Length(2), // header
            Constraint::Length(2), // host context
            Constraint::Length(3), // command field
            Constraint::Fill(1),
            Constraint::Length(1), // error
        ])
        .split(inner);

        // Header
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("\u{21b5}", Style::default().fg(Color::Yellow)),
                Span::styled(" run  ", Style::default().fg(Color::DarkGray)),
                Span::styled("Esc", Style::default().fg(Color::Yellow)),
                Span::styled(" back", Style::default().fg(Color::DarkGray)),
            ]))
            .centered(),
            chunks[0],
        );

        // Host context
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("Host: ", Style::default().fg(Color::DarkGray)),
                Span::styled(&self.host.host, Style::default().fg(Color::White)),
            ])),
            chunks[1],
        );

        // Command field
        frame.render_widget(
            render_field("Command", &self.command, true, "ls -la", true),
            chunks[2],
        );

        // Error
        frame.render_widget(
            Paragraph::new(self.error.as_deref().unwrap_or(""))
                .style(Style::default().fg(Color::Red))
                .centered(),
            chunks[4],
        );
    }
}
