use std::{os::unix::process::CommandExt, process::Command};

use crossterm::{event::KeyCode, execute, terminal::disable_raw_mode};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Flex, Layout},
    macros::horizontal,
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use crate::app::{
    App,
    screen::{
        SCREEN_HEIGHT_PERCENTAGE, SCREEN_WIDTH_PERCENTAGE, Screen, connect_screen,
        intro_screen::IntroScreen,
    },
};

#[derive(Debug)]
pub struct ConnectScreen {
    // current config value selected
    pub list_state: ListState,
    pub connections: Vec<u16>,
}

impl Default for ConnectScreen {
    fn default() -> Self {
        Self {
            list_state: ListState::default().with_selected(Some(0)),
            connections: vec![],
        }
    }
}

impl ConnectScreen {
    fn next(&mut self) {
        self.list_state.select_next();
    }

    fn previous(&mut self) {
        self.list_state.select_previous();
    }

    fn loadConnections(&mut self) {
        todo!()
    }

    pub fn match_key(&mut self, key: KeyCode, mut callback: impl FnMut(&str)) -> Option<Screen> {
        return match key {
            KeyCode::Up => {
                self.previous();
                None
            }
            KeyCode::Down => {
                self.next();
                None
            }
            KeyCode::Enter => {
                if let Some(selected) = self.list_state.selected() {
                    //let connection = self.connections[selected];
                    callback("hallo")
                }
                None
            }
            KeyCode::Esc => Some(Screen::Intro(IntroScreen::default())),
            _ => None,
        };
    }
}

impl ConnectScreen {
    pub fn draw(&mut self, error: Option<String>, frame: &mut Frame) {
        let area = frame.area();

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

        let chunks = Layout::vertical([
            Constraint::Length(2), // nav/title
            Constraint::Fill(1),   // list
            Constraint::Length(1), // error
        ])
        .split(inner);

        let header = Paragraph::new(Line::from(vec![
            Span::styled("↑↓", Style::default().fg(Color::Yellow)),
            Span::styled(" navigate  ", Style::default().fg(Color::DarkGray)),
            Span::styled("↵", Style::default().fg(Color::Yellow)),
            Span::styled(" select  ", Style::default().fg(Color::DarkGray)),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::styled(" go back  ", Style::default().fg(Color::DarkGray)),
            Span::styled("q", Style::default().fg(Color::Yellow)),
            Span::styled(" quit", Style::default().fg(Color::DarkGray)),
        ]))
        .centered();
        frame.render_widget(header, chunks[0]);

        // List
        let items = vec![
            ListItem::new(Text::from(vec![
                Line::styled("Connect", Style::default().fg(Color::White)),
                Line::styled(
                    "  Connect to a saved host",
                    Style::default().fg(Color::DarkGray),
                ),
            ])),
            ListItem::new(Text::from(vec![
                Line::styled("Settings", Style::default().fg(Color::White)),
                Line::styled(
                    "  Manage configuration",
                    Style::default().fg(Color::DarkGray),
                ),
            ])),
        ];

        let list = List::new(items)
            .highlight_style(Style::default().green().bold())
            .scroll_padding(1)
            .highlight_symbol(" > ");

        frame.render_stateful_widget(list, chunks[1], &mut self.list_state);

        let error = Paragraph::new(if let Some(error_string) = error {
            error_string
        } else {
            "".to_string()
        })
        .style(Style::default().fg(Color::Red))
        .centered();
        frame.render_widget(error, chunks[2]);
    }
}
