use crossterm::event::KeyCode;
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use crate::{
    app::screen::{
        SCREEN_HEIGHT_PERCENTAGE, SCREEN_WIDTH_PERCENTAGE, Screen,
        configure_screen::ConfigureScreen, intro_screen::IntroScreen,
    },
    ssh_config::{config_reader::get_ssh_entries, known_hosts::get_known_hosts},
};

#[derive(Debug)]
pub struct ImportScreen {
    pub hosts: Vec<String>,
    pub list_state: ListState,
    pub loaded: bool,
}

impl Default for ImportScreen {
    fn default() -> Self {
        Self {
            hosts: Vec::new(),
            list_state: ListState::default().with_selected(Some(0)),
            loaded: false,
        }
    }
}

impl ImportScreen {
    fn load_hosts(&mut self) {
        if self.loaded {
            return;
        }
        self.loaded = true;

        let known = get_known_hosts();
        let existing: Vec<String> = get_ssh_entries()
            .iter()
            .map(|e| e.hostname.clone())
            .collect();

        self.hosts = known
            .into_iter()
            .filter(|h| !existing.contains(h))
            .collect();
    }

    pub fn match_key(&mut self, key_code: KeyCode) -> Option<Screen> {
        match key_code {
            KeyCode::Up => {
                self.list_state.select_previous();
                None
            }
            KeyCode::Down => {
                self.list_state.select_next();
                None
            }
            KeyCode::Enter => {
                if let Some(selected) = self.list_state.selected() {
                    if let Some(hostname) = self.hosts.get(selected) {
                        return Some(Screen::Configure(
                            ConfigureScreen::with_hostname(hostname.clone()),
                        ));
                    }
                }
                None
            }
            KeyCode::Esc => Some(Screen::Intro(IntroScreen::default())),
            _ => None,
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        self.load_hosts();
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
            .title(" Import from known_hosts ")
            .title_style(Style::default().fg(Color::White))
            .borders(Borders::ALL)
            .border_style(Style::new().dark_gray());
        let inner = outer.inner(all);
        frame.render_widget(outer, all);

        let chunks = Layout::vertical([
            Constraint::Length(2), // header
            Constraint::Fill(1),   // list
        ])
        .split(inner);

        // Header
        let header = Paragraph::new(Line::from(vec![
            Span::styled("\u{2191}\u{2193}", Style::default().fg(Color::Yellow)),
            Span::styled(" navigate  ", Style::default().fg(Color::DarkGray)),
            Span::styled("\u{21b5}", Style::default().fg(Color::Yellow)),
            Span::styled(" import  ", Style::default().fg(Color::DarkGray)),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::styled(" back", Style::default().fg(Color::DarkGray)),
        ]))
        .centered();
        frame.render_widget(header, chunks[0]);

        if self.hosts.is_empty() {
            let msg = Paragraph::new("No new hosts found in known_hosts")
                .style(Style::default().fg(Color::DarkGray))
                .centered();
            frame.render_widget(msg, chunks[1]);
        } else {
            let items: Vec<ListItem> = self
                .hosts
                .iter()
                .map(|h| ListItem::new(h.as_str()))
                .collect();

            let list = List::new(items)
                .highlight_style(Style::default().green().bold())
                .scroll_padding(1)
                .highlight_symbol(" > ");

            frame.render_stateful_widget(list, chunks[1], &mut self.list_state);
        }
    }
}
