use crate::{
    app::screen::{
        SCREEN_HEIGHT_PERCENTAGE, SCREEN_WIDTH_PERCENTAGE, Screen, intro_screen::IntroScreen,
    },
    ssh_config::config_reader::{self, AuthMethod, SSHConfig},
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
use std::env::home_dir;

fn find_ssh_private_keys() -> Vec<String> {
    let ssh_dir = match home_dir() {
        Some(h) => h.join(".ssh"),
        None => return vec![],
    };
    let mut keys = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&ssh_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.ends_with(".pub")
                    || name == "known_hosts"
                    || name == "known_hosts.old"
                    || name == "authorized_keys"
                    || name == "config"
                {
                    continue;
                }
                let pub_path = ssh_dir.join(format!("{}.pub", name));
                if pub_path.exists() {
                    keys.push(format!("~/.ssh/{}", name));
                }
            }
        }
    }
    keys.sort();
    keys
}

#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum Step {
    #[default]
    Basic,
    Tunnel,
}

#[derive(Debug, Default, PartialEq)]
pub enum Field {
    // Step 1: Basic
    #[default]
    Host,
    Hostname,
    Port,
    User,
    IdentityFile,
    ProxyJump,
    // Step 2: Tunnel
    LocalForward,
    RemoteForward,
}

impl Field {
    fn next(&self, step: Step) -> Field {
        match step {
            Step::Basic => match self {
                Field::Host => Field::Hostname,
                Field::Hostname => Field::Port,
                Field::Port => Field::User,
                Field::User => Field::IdentityFile,
                Field::IdentityFile => Field::ProxyJump,
                Field::ProxyJump => Field::Host,
                _ => Field::Host,
            },
            Step::Tunnel => match self {
                Field::LocalForward => Field::RemoteForward,
                Field::RemoteForward => Field::LocalForward,
                _ => Field::LocalForward,
            },
        }
    }
    fn prev(&self, step: Step) -> Field {
        match step {
            Step::Basic => match self {
                Field::Host => Field::ProxyJump,
                Field::Hostname => Field::Host,
                Field::Port => Field::Hostname,
                Field::User => Field::Port,
                Field::IdentityFile => Field::User,
                Field::ProxyJump => Field::IdentityFile,
                _ => Field::Host,
            },
            Step::Tunnel => match self {
                Field::LocalForward => Field::RemoteForward,
                Field::RemoteForward => Field::LocalForward,
                _ => Field::LocalForward,
            },
        }
    }
}

#[derive(Debug)]
pub struct ConfigureScreen {
    pub host: String,
    pub hostname: String,
    pub port: String,
    pub user: String,
    pub identity_file: String,
    pub proxy_jump: String,
    pub local_forward: String,
    pub remote_forward: String,
    pub step: Step,
    pub focused: Field,
    pub error: Option<String>,
    pub editing: Option<String>,
    pub available_keys: Vec<String>,
    pub key_index: usize,
}

impl Default for ConfigureScreen {
    fn default() -> Self {
        let available_keys = find_ssh_private_keys();
        let identity_file = available_keys
            .first()
            .cloned()
            .unwrap_or_else(|| "~/.ssh/id_rsa".to_string());
        Self {
            host: String::new(),
            hostname: String::new(),
            port: String::new(),
            user: String::new(),
            identity_file,
            proxy_jump: String::new(),
            local_forward: String::new(),
            remote_forward: String::new(),
            step: Step::default(),
            focused: Field::default(),
            error: None,
            editing: None,
            available_keys,
            key_index: 0,
        }
    }
}

impl ConfigureScreen {
    pub fn with_hostname(hostname: String) -> Self {
        Self {
            hostname: hostname.clone(),
            host: hostname,
            ..Default::default()
        }
    }

    pub fn from_config(config: &SSHConfig) -> Self {
        let available_keys = find_ssh_private_keys();
        let identity_file = match &config.auth {
            Some(AuthMethod::Key(path)) => path.clone(),
            _ => available_keys
                .first()
                .cloned()
                .unwrap_or_else(|| "~/.ssh/id_rsa".to_string()),
        };
        let key_index = available_keys
            .iter()
            .position(|k| k == &identity_file)
            .unwrap_or(0);
        Self {
            host: config.host.clone(),
            hostname: config.hostname.clone(),
            port: config.port.map(|p| p.to_string()).unwrap_or_default(),
            user: config.username.clone().unwrap_or_default(),
            identity_file,
            proxy_jump: config.proxy_jump.clone().unwrap_or_default(),
            local_forward: config.local_forward.clone().unwrap_or_default(),
            remote_forward: config.remote_forward.clone().unwrap_or_default(),
            step: Step::default(),
            focused: Field::default(),
            error: None,
            editing: Some(config.host.clone()),
            available_keys,
            key_index,
        }
    }

    fn active_field_mut(&mut self) -> Option<&mut String> {
        match self.focused {
            Field::Host => Some(&mut self.host),
            Field::Hostname => Some(&mut self.hostname),
            Field::Port => Some(&mut self.port),
            Field::User => Some(&mut self.user),
            Field::IdentityFile => None,
            Field::ProxyJump => Some(&mut self.proxy_jump),
            Field::LocalForward => Some(&mut self.local_forward),
            Field::RemoteForward => Some(&mut self.remote_forward),
        }
    }

    fn draw_basic(&self, frame: &mut Frame) {
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

        let title = if self.editing.is_some() {
            " Edit Connection (1/2) "
        } else {
            " New Connection (1/2) "
        };

        let outer = Block::default()
            .title(title)
            .title_style(Style::default().fg(Color::White))
            .borders(Borders::ALL)
            .border_style(Style::new().dark_gray());
        let inner = outer.inner(all);
        frame.render_widget(outer, all);

        let chunks = Layout::vertical([
            Constraint::Length(2), // header
            Constraint::Length(3), // host
            Constraint::Length(3), // hostname
            Constraint::Length(3), // port
            Constraint::Length(3), // user
            Constraint::Length(3), // identity file
            Constraint::Length(3), // proxy jump
            Constraint::Fill(1),
            Constraint::Length(1), // error
        ])
        .split(inner);

        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("\u{2191}\u{2193}/Tab", Style::default().fg(Color::Yellow)),
                Span::styled(" navigate  ", Style::default().fg(Color::DarkGray)),
                Span::styled("\u{21b5}", Style::default().fg(Color::Yellow)),
                Span::styled(" next  ", Style::default().fg(Color::DarkGray)),
                Span::styled("Esc", Style::default().fg(Color::Yellow)),
                Span::styled(" cancel", Style::default().fg(Color::DarkGray)),
            ]))
            .centered(),
            chunks[0],
        );

        frame.render_widget(
            render_field("Host", &self.host, self.focused == Field::Host, "my-server", true),
            chunks[1],
        );
        frame.render_widget(
            render_field("Hostname", &self.hostname, self.focused == Field::Hostname, "192.168.1.1", true),
            chunks[2],
        );
        frame.render_widget(
            render_field("Port", &self.port, self.focused == Field::Port, "22", false),
            chunks[3],
        );
        frame.render_widget(
            render_field("User", &self.user, self.focused == Field::User, "root", false),
            chunks[4],
        );

        // IdentityFile selector
        let id_focused = self.focused == Field::IdentityFile;
        let id_border = if id_focused {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let id_display = if id_focused && !self.available_keys.is_empty() {
            format!("\u{25c0} {} \u{25b6}", self.identity_file)
        } else {
            self.identity_file.clone()
        };
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(id_display, Style::default().fg(Color::White)))).block(
                Block::default()
                    .title(" IdentityFile ")
                    .borders(Borders::ALL)
                    .border_style(id_border),
            ),
            chunks[5],
        );

        frame.render_widget(
            render_field("ProxyJump", &self.proxy_jump, self.focused == Field::ProxyJump, "bastion-host", false),
            chunks[6],
        );

        frame.render_widget(
            Paragraph::new(self.error.as_deref().unwrap_or(""))
                .style(Style::default().fg(Color::Red))
                .centered(),
            chunks[8],
        );
    }

    fn draw_tunnel(&self, frame: &mut Frame) {
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

        let title = if self.editing.is_some() {
            " Edit Connection (2/2) - Tunnels "
        } else {
            " New Connection (2/2) - Tunnels "
        };

        let outer = Block::default()
            .title(title)
            .title_style(Style::default().fg(Color::White))
            .borders(Borders::ALL)
            .border_style(Style::new().dark_gray());
        let inner = outer.inner(all);
        frame.render_widget(outer, all);

        let chunks = Layout::vertical([
            Constraint::Length(2), // header
            Constraint::Length(2), // description
            Constraint::Length(3), // local forward
            Constraint::Length(3), // remote forward
            Constraint::Fill(1),
            Constraint::Length(1), // error
        ])
        .split(inner);

        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("\u{2191}\u{2193}/Tab", Style::default().fg(Color::Yellow)),
                Span::styled(" navigate  ", Style::default().fg(Color::DarkGray)),
                Span::styled("\u{21b5}", Style::default().fg(Color::Yellow)),
                Span::styled(" save  ", Style::default().fg(Color::DarkGray)),
                Span::styled("Esc", Style::default().fg(Color::Yellow)),
                Span::styled(" back", Style::default().fg(Color::DarkGray)),
            ]))
            .centered(),
            chunks[0],
        );

        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("Optional: ", Style::default().fg(Color::DarkGray)),
                Span::styled("configure SSH tunnels that activate on connect", Style::default().fg(Color::DarkGray)),
            ]))
            .centered(),
            chunks[1],
        );

        frame.render_widget(
            render_field("LocalForward", &self.local_forward, self.focused == Field::LocalForward, "8080 localhost:80", false),
            chunks[2],
        );
        frame.render_widget(
            render_field("RemoteForward", &self.remote_forward, self.focused == Field::RemoteForward, "9090 localhost:3000", false),
            chunks[3],
        );

        frame.render_widget(
            Paragraph::new(self.error.as_deref().unwrap_or(""))
                .style(Style::default().fg(Color::Red))
                .centered(),
            chunks[5],
        );
    }

    pub fn draw(&self, frame: &mut Frame) {
        match self.step {
            Step::Basic => self.draw_basic(frame),
            Step::Tunnel => self.draw_tunnel(frame),
        }
    }

    fn validate_basic(&mut self) -> bool {
        if self.host.is_empty() || self.hostname.is_empty() {
            self.error = Some("Host and Hostname are required".into());
            return false;
        }
        if !self.port.is_empty() {
            if self.port.parse::<u16>().is_err() {
                self.error = Some("Port must be a number".into());
                return false;
            }
        }
        true
    }

    fn save(&mut self) -> Option<Screen> {
        let port = if self.port.is_empty() {
            None
        } else {
            self.port.parse::<u16>().ok()
        };

        let username = if self.user.is_empty() {
            None
        } else {
            Some(self.user.clone())
        };

        let identity = if self.identity_file.is_empty() {
            "~/.ssh/id_rsa".to_string()
        } else {
            self.identity_file.clone()
        };
        let auth = Some(config_reader::AuthMethod::Key(identity));

        let proxy_jump = if self.proxy_jump.is_empty() { None } else { Some(self.proxy_jump.clone()) };
        let local_forward = if self.local_forward.is_empty() { None } else { Some(self.local_forward.clone()) };
        let remote_forward = if self.remote_forward.is_empty() { None } else { Some(self.remote_forward.clone()) };

        let config = SSHConfig {
            host: self.host.clone(),
            hostname: self.hostname.clone(),
            username,
            port,
            auth,
            proxy_jump,
            local_forward,
            remote_forward,
        };

        let result = if let Some(ref old_name) = self.editing {
            config_reader::update_host(old_name, &config)
        } else {
            config_reader::write_new_host(&config)
        };

        match result {
            Ok(_) => Some(Screen::Intro(IntroScreen::default())),
            Err(e) => {
                self.error = Some(format!("Failed to write: {e}"));
                None
            }
        }
    }

    pub fn match_key(&mut self, key_code: KeyCode) -> Option<Screen> {
        match key_code {
            KeyCode::Esc => {
                match self.step {
                    Step::Basic => return Some(Screen::Intro(IntroScreen::default())),
                    Step::Tunnel => {
                        self.step = Step::Basic;
                        self.focused = Field::Host;
                        self.error = None;
                    }
                }
            }

            KeyCode::Tab | KeyCode::Down => self.focused = self.focused.next(self.step),
            KeyCode::BackTab | KeyCode::Up => self.focused = self.focused.prev(self.step),

            KeyCode::Left => {
                if self.focused == Field::IdentityFile && !self.available_keys.is_empty() {
                    if self.key_index == 0 {
                        self.key_index = self.available_keys.len() - 1;
                    } else {
                        self.key_index -= 1;
                    }
                    self.identity_file = self.available_keys[self.key_index].clone();
                }
            }
            KeyCode::Right => {
                if self.focused == Field::IdentityFile && !self.available_keys.is_empty() {
                    self.key_index = (self.key_index + 1) % self.available_keys.len();
                    self.identity_file = self.available_keys[self.key_index].clone();
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
                match self.step {
                    Step::Basic => {
                        if self.validate_basic() {
                            self.step = Step::Tunnel;
                            self.focused = Field::LocalForward;
                            self.error = None;
                        }
                    }
                    Step::Tunnel => {
                        return self.save();
                    }
                }
            }

            _ => {}
        }
        None
    }
}
