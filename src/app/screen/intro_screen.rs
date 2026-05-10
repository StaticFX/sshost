use ratatui::{
    Frame,
    layout::{Constraint, Direction, Flex, Layout},
    macros::horizontal,
    style::{Color, Style},
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::App;

#[derive(Debug, Default)]
pub struct IntroScreen {
    // 0 = existing host
    // 1 = new host
    pub selected: usize,
}

impl IntroScreen {
    fn selectExisting(&mut self) {
        self.selected = 0;
    }

    fn selectNew(&mut self) {
        self.selected = 0;
    }
}

impl IntroScreen {
    pub fn draw(&self, app: &App, frame: &mut Frame) {
        let area = frame.area();

        let vertical = Layout::vertical([Constraint::Percentage(75)]).split(area);
        let horizontal = Layout::horizontal([Constraint::Fill(1)]).split(vertical[0]);

        let all = horizontal[0];

        let outer = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::new().dark_gray());

        let inner = outer.inner(all);
        frame.render_widget(outer, all);

        let chunks = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Percentage(70),
            Constraint::Percentage(30),
            Constraint::Fill(1),
        ])
        .split(inner);

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
        ];

        let titles =
            Layout::vertical([Constraint::Length(4), Constraint::Length(1)]).split(chunks[1]);

        let title = Paragraph::new(art).centered();
        let sub_title = Paragraph::new("Jump into hosts").centered();

        frame.render_widget(title, titles[0]);
        frame.render_widget(sub_title, titles[1]);

        let option_layout =
            Layout::vertical([Constraint::Fill(1), Constraint::Fill(1)]).split(chunks[2]);

        for (i, option) in ["Connect with existing", "Add new host"].iter().enumerate() {
            let selected = i == self.selected;
            let style = if selected {
                Style::new().green().bold()
            } else {
                Style::new().dark_gray()
            };

            let current_text = *option;
            let prefix = if selected { " > " } else { "   " };
            let text = Paragraph::new(format!("{prefix}{current_text}"))
                .style(style)
                .centered();

            frame.render_widget(text, option_layout[i]);
        }
    }
}
