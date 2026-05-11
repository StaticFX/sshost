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
    ssh_config::{
        config_reader::get_ssh_entries,
        known_hosts::{get_known_hosts, remove_known_host},
    },
};

#[derive(Debug)]
pub struct KnownHostsScreen {
    pub hosts: Vec<String>,
    pub existing: Vec<String>,
    pub list_state: ListState,
    pub loaded: bool,
    pub confirm_delete: Option<usize>,
    pub status: Option<(String, Color)>,
}

impl Default for KnownHostsScreen {
    fn default() -> Self {
        Self {
            hosts: Vec::new(),
            existing: Vec::new(),
            list_state: ListState::default().with_selected(Some(0)),
            loaded: false,
            confirm_delete: None,
            status: None,
        }
    }
}

impl KnownHostsScreen {
    fn load_hosts(&mut self) {
        if self.loaded {
            return;
        }
        self.loaded = true;
        self.hosts = get_known_hosts();
        self.existing = get_ssh_entries()
            .iter()
            .map(|e| e.hostname.clone())
            .collect();
    }

    fn reload_hosts(&mut self) {
        self.hosts = get_known_hosts();
        self.existing = get_ssh_entries()
            .iter()
            .map(|e| e.hostname.clone())
            .collect();
    }

    fn is_configured(&self, host: &str) -> bool {
        self.existing.contains(&host.to_string())
    }

    pub fn match_key(&mut self, key_code: KeyCode) -> Option<Screen> {
        // Handle delete confirmation
        if let Some(idx) = self.confirm_delete {
            return match key_code {
                KeyCode::Char('y') => {
                    if let Some(host) = self.hosts.get(idx).cloned() {
                        match remove_known_host(&host) {
                            Ok(_) => {
                                self.status = Some((format!("Removed '{}'", host), Color::Green));
                                self.reload_hosts();
                                if self.hosts.is_empty() {
                                    self.list_state.select(None);
                                } else if idx >= self.hosts.len() {
                                    self.list_state.select(Some(self.hosts.len() - 1));
                                }
                            }
                            Err(e) => {
                                self.status = Some((format!("Error: {}", e), Color::Red));
                            }
                        }
                    }
                    self.confirm_delete = None;
                    None
                }
                KeyCode::Char('n') | KeyCode::Esc => {
                    self.confirm_delete = None;
                    self.status = None;
                    None
                }
                _ => None,
            };
        }

        match key_code {
            KeyCode::Up => {
                self.list_state.select_previous();
                self.status = None;
                None
            }
            KeyCode::Down => {
                self.list_state.select_next();
                self.status = None;
                None
            }
            KeyCode::Enter => {
                if let Some(selected) = self.list_state.selected() {
                    if let Some(hostname) = self.hosts.get(selected) {
                        if !self.is_configured(hostname) {
                            return Some(Screen::Configure(
                                ConfigureScreen::with_hostname(hostname.clone()),
                            ));
                        } else {
                            self.status = Some(("Already configured".into(), Color::Yellow));
                        }
                    }
                }
                None
            }
            KeyCode::Char('d') => {
                if let Some(selected) = self.list_state.selected() {
                    if selected < self.hosts.len() {
                        self.confirm_delete = Some(selected);
                        self.status = None;
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
            .title(" known_hosts ")
            .title_style(Style::default().fg(Color::White))
            .borders(Borders::ALL)
            .border_style(Style::new().dark_gray());
        let inner = outer.inner(all);
        frame.render_widget(outer, all);

        let chunks = Layout::vertical([
            Constraint::Length(2), // header
            Constraint::Fill(1),   // list
            Constraint::Length(1), // status
        ])
        .split(inner);

        // Header
        let header = Paragraph::new(Line::from(vec![
            Span::styled("\u{2191}\u{2193}", Style::default().fg(Color::Yellow)),
            Span::styled(" navigate  ", Style::default().fg(Color::DarkGray)),
            Span::styled("\u{21b5}", Style::default().fg(Color::Yellow)),
            Span::styled(" import  ", Style::default().fg(Color::DarkGray)),
            Span::styled("d", Style::default().fg(Color::Yellow)),
            Span::styled(" remove  ", Style::default().fg(Color::DarkGray)),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::styled(" back", Style::default().fg(Color::DarkGray)),
        ]))
        .centered();
        frame.render_widget(header, chunks[0]);

        if self.hosts.is_empty() {
            let msg = Paragraph::new("No entries in known_hosts")
                .style(Style::default().fg(Color::DarkGray))
                .centered();
            frame.render_widget(msg, chunks[1]);
        } else {
            let items: Vec<ListItem> = self
                .hosts
                .iter()
                .map(|h| {
                    if self.is_configured(h) {
                        ListItem::new(Line::from(vec![
                            Span::styled(h.as_str(), Style::default().fg(Color::White)),
                            Span::styled("  configured", Style::default().fg(Color::DarkGray)),
                        ]))
                    } else {
                        ListItem::new(Span::styled(h.as_str(), Style::default().fg(Color::White)))
                    }
                })
                .collect();

            let list = List::new(items)
                .highlight_style(Style::default().green().bold())
                .scroll_padding(1)
                .highlight_symbol(" > ");

            frame.render_stateful_widget(list, chunks[1], &mut self.list_state);
        }

        // Status / confirm bar
        let bottom = if let Some(idx) = self.confirm_delete {
            let name = self.hosts.get(idx).map(|s| s.as_str()).unwrap_or("?");
            Paragraph::new(format!("Remove '{}'? y/n", name))
                .style(Style::default().fg(Color::Yellow))
                .centered()
        } else if let Some((ref msg, color)) = self.status {
            Paragraph::new(msg.as_str())
                .style(Style::default().fg(color))
                .centered()
        } else {
            Paragraph::new("")
        };
        frame.render_widget(bottom, chunks[2]);
    }
}
