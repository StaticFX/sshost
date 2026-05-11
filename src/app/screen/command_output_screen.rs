use crate::app::screen::{
    SCREEN_HEIGHT_PERCENTAGE, SCREEN_WIDTH_PERCENTAGE, Screen,
    connect_screen::ConnectScreen,
};
use crossterm::event::KeyCode;
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

#[derive(Debug)]
pub struct CommandOutputScreen {
    pub output: String,
    pub scroll: u16,
}

impl CommandOutputScreen {
    pub fn new(output: String) -> Self {
        Self { output, scroll: 0 }
    }

    pub fn handle_key(&mut self, key: KeyCode) -> Option<Screen> {
        match key {
            KeyCode::Esc => Some(Screen::Connect(ConnectScreen::default())),
            KeyCode::Up | KeyCode::Char('k') => {
                self.scroll = self.scroll.saturating_sub(1);
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.scroll = self.scroll.saturating_add(1);
                None
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
            .title(" Output ")
            .title_style(Style::default().fg(Color::White))
            .borders(Borders::ALL)
            .border_style(Style::new().dark_gray());
        let inner = outer.inner(all);
        frame.render_widget(outer, all);

        let chunks = Layout::vertical([
            Constraint::Length(2), // header
            Constraint::Fill(1),   // output
        ])
        .split(inner);

        // Header
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("\u{2191}\u{2193}/jk", Style::default().fg(Color::Yellow)),
                Span::styled(" scroll  ", Style::default().fg(Color::DarkGray)),
                Span::styled("Esc", Style::default().fg(Color::Yellow)),
                Span::styled(" back", Style::default().fg(Color::DarkGray)),
            ]))
            .centered(),
            chunks[0],
        );

        // Output
        frame.render_widget(
            Paragraph::new(self.output.as_str())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::DarkGray)),
                )
                .scroll((self.scroll, 0)),
            chunks[1],
        );
    }
}
