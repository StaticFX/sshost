use std::{boxed, collections::HashMap, fmt, net::{TcpStream, ToSocketAddrs}, os::unix::process::CommandExt, process::Command, sync::mpsc::{self, Receiver}, thread, time::Duration};

use crossterm::{event::KeyCode, execute, terminal::disable_raw_mode};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Flex, Layout},
    macros::horizontal,
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use chrono::Utc;

use crate::{
    app::{
        App,
        screen::{
            SCREEN_HEIGHT_PERCENTAGE, SCREEN_WIDTH_PERCENTAGE, Screen, connect_screen,
            configure_screen::ConfigureScreen,
            intro_screen::IntroScreen,
            portforward_screen::PortForwardScreen,
            quick_command_screen::QuickCommandScreen,
            transfer_screen::TransferScreen,
        },
    },
    history::get_last_connection,
    ssh_config::config_reader::{self, AuthMethod, SSHConfig, get_ssh_entries},
};

pub struct ConnectScreen {
    // current config value selected
    pub list_state: ListState,
    pub connections: Vec<SSHConfig>,
    loaded_connections: bool,
    pub reachability: HashMap<String, Option<bool>>,
    reachability_rx: Option<Receiver<(String, bool)>>,
    pub confirm_delete: Option<usize>,
    pub filter: String,
    pub filter_mode: bool,
}

impl fmt::Debug for ConnectScreen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConnectScreen")
            .field("list_state", &self.list_state)
            .field("connections", &self.connections)
            .field("loaded_connections", &self.loaded_connections)
            .field("reachability", &self.reachability)
            .field("reachability_rx", &"<receiver>")
            .field("confirm_delete", &self.confirm_delete)
            .field("filter", &self.filter)
            .field("filter_mode", &self.filter_mode)
            .finish()
    }
}

impl Default for ConnectScreen {
    fn default() -> Self {
        Self {
            list_state: ListState::default().with_selected(Some(0)),
            connections: vec![],
            loaded_connections: false,
            reachability: HashMap::new(),
            reachability_rx: None,
            confirm_delete: None,
            filter: String::new(),
            filter_mode: false,
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

    fn load_connections(&mut self) {
        if !self.loaded_connections {
            self.connections = get_ssh_entries();
            self.loaded_connections = true;
        }
    }

    fn reload_connections(&mut self) {
        self.connections = get_ssh_entries();
        self.loaded_connections = true;
    }

    fn filtered_connections(&self) -> Vec<&SSHConfig> {
        if self.filter.is_empty() {
            self.connections.iter().collect()
        } else {
            let lower_filter = self.filter.to_lowercase();
            self.connections
                .iter()
                .filter(|c| {
                    c.host.to_lowercase().contains(&lower_filter)
                        || c.hostname.to_lowercase().contains(&lower_filter)
                })
                .collect()
        }
    }

    fn test_reachability(&mut self) {
        let (tx, rx) = mpsc::channel();
        self.reachability_rx = Some(rx);

        for conn in &self.connections {
            let hostname = conn.hostname.clone();
            let port = conn.port.unwrap_or(22);
            let host_key = conn.host.clone();
            let tx = tx.clone();

            thread::spawn(move || {
                let addr_string = format!("{}:{}", hostname, port);
                let reachable = if let Ok(mut addrs) = addr_string.to_socket_addrs() {
                    if let Some(addr) = addrs.next() {
                        TcpStream::connect_timeout(&addr, Duration::from_secs(3)).is_ok()
                    } else {
                        false
                    }
                } else {
                    false
                };
                let _ = tx.send((host_key, reachable));
            });
        }
    }

    pub fn poll_reachability(&mut self) {
        if let Some(rx) = &self.reachability_rx {
            while let Ok((host, reachable)) = rx.try_recv() {
                self.reachability.insert(host, Some(reachable));
            }
        }
    }

    pub fn match_key(
        &mut self,
        key: KeyCode,
        mut callback: impl FnMut(SSHConfig),
    ) -> Option<Screen> {
        // Handle delete confirmation mode
        if let Some(idx) = self.confirm_delete {
            return match key {
                KeyCode::Char('y') => {
                    if let Some(connection) = self.connections.get(idx) {
                        let _ = config_reader::delete_host(&connection.host);
                        self.confirm_delete = None;
                        self.reload_connections();
                        // Adjust selection if needed
                        if self.connections.is_empty() {
                            self.list_state.select(None);
                        } else if idx >= self.connections.len() {
                            self.list_state.select(Some(self.connections.len() - 1));
                        }
                    }
                    None
                }
                KeyCode::Char('n') | KeyCode::Esc => {
                    self.confirm_delete = None;
                    None
                }
                _ => None,
            };
        }

        // Handle filter mode
        if self.filter_mode {
            match key {
                KeyCode::Esc => {
                    self.filter_mode = false;
                    self.filter.clear();
                    self.list_state.select(Some(0));
                }
                KeyCode::Backspace => {
                    self.filter.pop();
                    self.list_state.select(Some(0));
                }
                KeyCode::Enter => {
                    self.filter_mode = false;
                }
                KeyCode::Char(c) => {
                    self.filter.push(c);
                    self.list_state.select(Some(0));
                }
                _ => {}
            }
            return None;
        }

        return match key {
            KeyCode::Up => {
                self.previous();
                None
            }
            KeyCode::Down => {
                self.next();
                None
            }
            KeyCode::Char('/') => {
                self.filter_mode = true;
                self.filter.clear();
                self.list_state.select(Some(0));
                None
            }
            KeyCode::Char('t') => {
                self.test_reachability();
                None
            }
            KeyCode::Char('d') => {
                let filtered = self.filtered_connections();
                if let Some(selected) = self.list_state.selected() {
                    if selected < filtered.len() {
                        // Find the index in the original connections list
                        let host = filtered[selected].host.clone();
                        if let Some(idx) = self.connections.iter().position(|c| c.host == host) {
                            self.confirm_delete = Some(idx);
                        }
                    }
                }
                None
            }
            KeyCode::Char('e') => {
                let filtered = self.filtered_connections();
                if let Some(selected) = self.list_state.selected() {
                    if let Some(connection) = filtered.get(selected) {
                        let screen = ConfigureScreen::from_config(connection);
                        return Some(Screen::Configure(screen));
                    }
                }
                None
            }
            KeyCode::Char('s') => {
                let filtered = self.filtered_connections();
                if let Some(selected) = self.list_state.selected() {
                    if let Some(connection) = filtered.get(selected) {
                        return Some(Screen::Transfer(TransferScreen::new((*connection).clone())));
                    }
                }
                None
            }
            KeyCode::Char('r') => {
                let filtered = self.filtered_connections();
                if let Some(selected) = self.list_state.selected() {
                    if let Some(connection) = filtered.get(selected) {
                        return Some(Screen::QuickCommand(QuickCommandScreen::new((*connection).clone())));
                    }
                }
                None
            }
            KeyCode::Char('f') => {
                let filtered = self.filtered_connections();
                if let Some(selected) = self.list_state.selected() {
                    if let Some(connection) = filtered.get(selected) {
                        return Some(Screen::PortForward(PortForwardScreen::new((*connection).clone())));
                    }
                }
                None
            }
            KeyCode::Enter => {
                let filtered = self.filtered_connections();
                if let Some(selected) = self.list_state.selected() {
                    if let Some(connection) = filtered.get(selected) {
                        callback((*connection).clone())
                    }
                }
                None
            }
            KeyCode::Esc => Some(Screen::Intro(IntroScreen::default())),
            _ => None,
        };
    }
}

fn format_relative_time(timestamp: chrono::DateTime<Utc>) -> String {
    let duration = Utc::now().signed_duration_since(timestamp);
    let minutes = duration.num_minutes();
    if minutes < 1 {
        "just now".to_string()
    } else if minutes < 60 {
        format!("{}m ago", minutes)
    } else {
        let hours = duration.num_hours();
        if hours < 24 {
            format!("{}h ago", hours)
        } else {
            let days = duration.num_days();
            format!("{}d ago", days)
        }
    }
}

impl ConnectScreen {
    pub fn draw(&mut self, error: Option<String>, frame: &mut Frame) {
        self.poll_reachability();
        self.load_connections();
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

        let has_filter_bar = self.filter_mode || !self.filter.is_empty();

        let chunks = Layout::vertical([
            Constraint::Length(2), // nav/title
            Constraint::Length(if has_filter_bar { 1 } else { 0 }), // filter bar
            Constraint::Fill(1),   // list
            Constraint::Length(1), // error
        ])
        .split(inner);

        let header = Paragraph::new(Line::from(vec![
            Span::styled("\u{2191}\u{2193}", Style::default().fg(Color::Yellow)),
            Span::styled(" navigate  ", Style::default().fg(Color::DarkGray)),
            Span::styled("\u{21b5}", Style::default().fg(Color::Yellow)),
            Span::styled(" select  ", Style::default().fg(Color::DarkGray)),
            Span::styled("/", Style::default().fg(Color::Yellow)),
            Span::styled(" filter  ", Style::default().fg(Color::DarkGray)),
            Span::styled("e", Style::default().fg(Color::Yellow)),
            Span::styled(" edit  ", Style::default().fg(Color::DarkGray)),
            Span::styled("d", Style::default().fg(Color::Yellow)),
            Span::styled(" delete  ", Style::default().fg(Color::DarkGray)),
            Span::styled("t", Style::default().fg(Color::Yellow)),
            Span::styled(" test  ", Style::default().fg(Color::DarkGray)),
            Span::styled("s", Style::default().fg(Color::Yellow)),
            Span::styled(" scp  ", Style::default().fg(Color::DarkGray)),
            Span::styled("r", Style::default().fg(Color::Yellow)),
            Span::styled(" run cmd  ", Style::default().fg(Color::DarkGray)),
            Span::styled("f", Style::default().fg(Color::Yellow)),
            Span::styled(" forward  ", Style::default().fg(Color::DarkGray)),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::styled(" go back  ", Style::default().fg(Color::DarkGray)),
            Span::styled("q", Style::default().fg(Color::Yellow)),
            Span::styled(" quit", Style::default().fg(Color::DarkGray)),
        ]))
        .centered();
        frame.render_widget(header, chunks[0]);

        // Filter bar
        if has_filter_bar {
            let cursor = if self.filter_mode { "\u{2588}" } else { "" };
            let filter_text = format!("Filter: {}{}", self.filter, cursor);
            let filter_bar = Paragraph::new(Line::from(vec![
                Span::styled(filter_text, Style::default().fg(Color::Cyan)),
            ]));
            frame.render_widget(filter_bar, chunks[1]);
        }

        // List
        let filtered = self.filtered_connections();
        let items: Vec<ListItem> = filtered
            .iter()
            .map(|config| {
                let user = config.username.as_deref().unwrap_or("");
                let port = config.port.map(|p| p.to_string()).unwrap_or_default();
                let auth = match &config.auth {
                    Some(AuthMethod::Key(path)) => format!("key: {}", path),
                    Some(AuthMethod::Password(_)) => "password".to_string(),
                    None => String::new(),
                };

                let (indicator, indicator_color) = match self.reachability.get(&config.host) {
                    Some(Some(true)) => ("\u{25cf}", Color::Green),
                    Some(Some(false)) => ("\u{25cf}", Color::Red),
                    _ => ("\u{25cb}", Color::DarkGray),
                };

                let last_conn = match get_last_connection(&config.host) {
                    Some(ts) => format!("last: {}", format_relative_time(ts)),
                    None => "last: never".to_string(),
                };

                ListItem::new(Text::from(vec![
                    Line::from(vec![
                        Span::styled(format!("{} ", indicator), Style::default().fg(indicator_color)),
                        Span::styled(config.host.clone(), Style::default().fg(Color::White)),
                    ]),
                    Line::styled(
                        format!("  {} | {}:{} | {}", user, config.hostname, port, auth),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Line::styled(
                        format!("  {}", last_conn),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]))
            })
            .collect();

        let list = List::new(items)
            .highlight_style(Style::default().green().bold())
            .scroll_padding(1)
            .highlight_symbol(" > ");

        frame.render_stateful_widget(list, chunks[2], &mut self.list_state);

        // Show delete confirmation or error
        let bottom_text = if let Some(idx) = self.confirm_delete {
            let name = self.connections.get(idx)
                .map(|c| c.host.as_str())
                .unwrap_or("?");
            Paragraph::new(format!("Delete '{}'? y/n", name))
                .style(Style::default().fg(Color::Yellow))
                .centered()
        } else if let Some(error_string) = error {
            Paragraph::new(error_string)
                .style(Style::default().fg(Color::Red))
                .centered()
        } else {
            Paragraph::new("")
                .style(Style::default().fg(Color::Red))
                .centered()
        };
        frame.render_widget(bottom_text, chunks[3]);
    }
}
