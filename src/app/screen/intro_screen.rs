use crossterm::event::KeyCode;
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::{
    screen::{
        SCREEN_HEIGHT_PERCENTAGE, SCREEN_WIDTH_PERCENTAGE, Screen,
        configure_screen::ConfigureScreen, connect_screen::ConnectScreen,
        keygen_screen::KeygenScreen, known_hosts_screen::KnownHostsScreen,
        upload_screen::UploadScreen,
    },
};

#[derive(Debug, Default)]
pub struct IntroScreen {
    pub selected: usize,
}

impl IntroScreen {
    const OPTION_COUNT: usize = 5;

    pub fn match_key(&mut self, key: KeyCode) -> Option<Screen> {
        return match key {
            KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
                None
            }
            KeyCode::Down => {
                if self.selected < Self::OPTION_COUNT - 1 {
                    self.selected += 1;
                }
                None
            }
            KeyCode::Enter => match self.selected {
                0 => Some(Screen::Connect(ConnectScreen::default())),
                1 => Some(Screen::Configure(ConfigureScreen::default())),
                2 => Some(Screen::Upload(UploadScreen::default())),
                3 => Some(Screen::Keygen(KeygenScreen::default())),
                4 => Some(Screen::KnownHosts(KnownHostsScreen::default())),
                _ => None,
            },
            _ => None,
        };
    }
}

impl IntroScreen {
    pub fn draw(&self, frame: &mut Frame) {
        let area = frame.area();

        // Center the whole block
        let vertical = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Percentage(SCREEN_WIDTH_PERCENTAGE),
            Constraint::Fill(1),
        ])
        .split(area);

        let horizontal = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Percentage(SCREEN_HEIGHT_PERCENTAGE),
            Constraint::Fill(1),
        ])
        .split(vertical[1]);

        let all = horizontal[1];

        let outer = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::new().dark_gray());
        let inner = outer.inner(all);
        frame.render_widget(outer, all);

        // Inner sections: art + subtitle / gap / options / hint
        let chunks = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(6), // 4 art lines + 1 blank + 1 subtitle
            Constraint::Length(1), // gap
            Constraint::Length(5), // 5 options
            Constraint::Length(1), // hint
            Constraint::Fill(1),
        ])
        .split(inner);

        // ASCII art + subtitle
        let art = vec![
            Line::styled(
                r"/ _\/ _\  /\  /\___  ___| |_ ",
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            Line::styled(
                r"\ \ \ \  / /_/ / _ \/ __| __|",
                Style::default().fg(Color::LightCyan).add_modifier(Modifier::BOLD),
            ),
            Line::styled(
                r"_\ \_\ \/ __  / (_) \__ \ |_ ",
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            Line::styled(
                r"\__/\__/\/ /_/ \___/|___/\__|",
                Style::default().fg(Color::LightCyan).add_modifier(Modifier::BOLD),
            ),
            Line::from(""),
            Line::styled("Jump into hosts", Style::default().fg(Color::White)),
        ];
        frame.render_widget(Paragraph::new(art).centered(), chunks[1]);

        // Options
        let options = [
            "Connect with existing",
            "Add new host",
            "Upload key to server",
            "Generate SSH key",
            "Manage known_hosts",
        ];
        let option_layout = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(chunks[3]);

        for (i, option) in options.iter().enumerate() {
            let selected = i == self.selected;
            let style = if selected {
                Style::new().green().bold()
            } else {
                Style::new().dark_gray()
            };
            let prefix = if selected { " > " } else { "   " };
            frame.render_widget(
                Paragraph::new(format!("{prefix}{option}"))
                    .style(style)
                    .centered(),
                option_layout[i],
            );
        }

        // Hint
        let hint = Paragraph::new(Line::from(vec![
            Span::styled("\u{2191}\u{2193}", Style::default().fg(Color::Yellow)),
            Span::styled(" navigate  ", Style::default().fg(Color::DarkGray)),
            Span::styled("\u{21b5}", Style::default().fg(Color::Yellow)),
            Span::styled(" select  ", Style::default().fg(Color::DarkGray)),
            Span::styled("q", Style::default().fg(Color::Yellow)),
            Span::styled(" quit", Style::default().fg(Color::DarkGray)),
        ]))
        .centered();
        frame.render_widget(hint, chunks[4]);
    }
}
