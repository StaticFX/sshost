use crossterm::event::KeyCode;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Flex, Layout},
    macros::horizontal,
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::{
    App,
    screen::{
        SCREEN_HEIGHT_PERCENTAGE, SCREEN_WIDTH_PERCENTAGE, Screen,
        configure_screen::ConfigureScreen, connect_screen::ConnectScreen,
    },
};

#[derive(Debug, Default)]
pub struct IntroScreen {
    // 0 = existing host
    // 1 = new host
    pub selected: usize,
}

impl IntroScreen {
    fn select_existing(&mut self) {
        self.selected = 0;
    }

    fn select_new(&mut self) {
        self.selected = 1;
    }

    pub fn match_key(&mut self, key: KeyCode) -> Option<Screen> {
        return match key {
            KeyCode::Up => {
                self.select_existing();
                None
            }
            KeyCode::Down => {
                self.select_new();
                None
            }
            KeyCode::Enter => {
                if self.selected == 0 {
                    Some(Screen::Connect(ConnectScreen::default()))
                } else {
                    Some(Screen::Configure(ConfigureScreen::default()))
                }
            }
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
            Constraint::Length(5), // 4 art lines + 1 subtitle
            Constraint::Length(1), // gap
            Constraint::Length(2), // 2 options
            Constraint::Length(1), // hint
            Constraint::Fill(1),
        ])
        .split(inner);

        // ASCII art + subtitle
        let art = vec![
            Line::styled(
                r"/ _\/ _\  /\  /\___  ___| |_ ",
                Style::default().fg(Color::DarkGray),
            ),
            Line::styled(
                r"\ \ \ \  / /_/ / _ \/ __| __|",
                Style::default().fg(Color::Gray),
            ),
            Line::styled(
                r"_\ \_\ \/ __  / (_) \__ \ |_ ",
                Style::default().fg(Color::DarkGray),
            ),
            Line::styled(
                r"\__/\__/\/ /_/ \___/|___/\__|",
                Style::default().fg(Color::Gray),
            ),
            Line::styled("Jump into hosts", Style::default().fg(Color::DarkGray)),
        ];
        frame.render_widget(Paragraph::new(art).centered(), chunks[1]);

        // Options
        let options = ["Connect with existing", "Add new host"];
        let option_layout =
            Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).split(chunks[3]);

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
            Span::styled("↑↓", Style::default().fg(Color::Yellow)),
            Span::styled(" navigate  ", Style::default().fg(Color::DarkGray)),
            Span::styled("↵", Style::default().fg(Color::Yellow)),
            Span::styled(" select", Style::default().fg(Color::DarkGray)),
        ]))
        .centered();
        frame.render_widget(hint, chunks[4]);
    }
}
