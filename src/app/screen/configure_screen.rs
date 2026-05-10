use crate::app::{
    App,
    screen::{
        SCREEN_HEIGHT_PERCENTAGE, SCREEN_WIDTH_PERCENTAGE, Screen, intro_screen::IntroScreen,
    },
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
};

#[derive(Debug, Default, PartialEq)]
pub enum AuthMethod {
    #[default]
    Password,
    SshKey,
}

#[derive(Debug, Default, PartialEq)]
pub enum Field {
    #[default]
    Host,
    Hostname,
    Port,
    User,
    AuthMethod,
    AuthValue,
}

impl Field {
    fn next(&self) -> Field {
        match self {
            Field::Host => Field::Hostname,
            Field::Hostname => Field::Port,
            Field::Port => Field::User,
            Field::User => Field::AuthMethod,
            Field::AuthMethod => Field::AuthValue,
            Field::AuthValue => Field::Host,
        }
    }
    fn prev(&self) -> Field {
        match self {
            Field::Host => Field::AuthValue,
            Field::Hostname => Field::Host,
            Field::Port => Field::Hostname,
            Field::User => Field::Port,
            Field::AuthMethod => Field::User,
            Field::AuthValue => Field::AuthMethod,
        }
    }
}

#[derive(Debug, Default)]
pub struct ConfigureScreen {
    pub host: String,
    pub hostname: String,
    pub port: String,
    pub user: String,
    pub auth_value: String,
    pub auth_method: AuthMethod,
    pub focused: Field,
    pub error: Option<String>,
}

impl ConfigureScreen {
    fn active_field_mut(&mut self) -> Option<&mut String> {
        match self.focused {
            Field::Host => Some(&mut self.host),
            Field::Hostname => Some(&mut self.hostname),
            Field::Port => Some(&mut self.port),
            Field::User => Some(&mut self.user),
            Field::AuthValue => Some(&mut self.auth_value),
            Field::AuthMethod => None,
        }
    }

    fn render_field<'a>(
        label: &'a str,
        value: &'a str,
        is_focused: bool,
        placeholder: &'a str,
        masked: bool,
        required: bool,
    ) -> Paragraph<'a> {
        let border_style = if is_focused {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let display: String = if value.is_empty() {
            if is_focused {
                "█".to_string()
            } else {
                placeholder.to_string()
            }
        } else if masked {
            let dots = "•".repeat(value.len());
            if is_focused {
                format!("{dots}█")
            } else {
                dots
            }
        } else if is_focused {
            format!("{value}█")
        } else {
            value.to_string()
        };

        let span_style = if value.is_empty() && !is_focused {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default().fg(Color::White)
        };

        let title = if required {
            format!(" {label} * ")
        } else {
            format!(" {label} ")
        };

        Paragraph::new(Line::from(Span::styled(display, span_style))).block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(border_style),
        )
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
            .title(" Configure Connection ")
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
            Constraint::Length(3), // auth method toggle
            Constraint::Length(3), // auth value
            Constraint::Fill(1),
            Constraint::Length(1), // error
        ])
        .split(inner);

        // Header
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("↑↓/Tab", Style::default().fg(Color::Yellow)),
                Span::styled(" navigate  ", Style::default().fg(Color::DarkGray)),
                Span::styled("↵", Style::default().fg(Color::Yellow)),
                Span::styled(" save  ", Style::default().fg(Color::DarkGray)),
                Span::styled("←→", Style::default().fg(Color::Yellow)),
                Span::styled(" toggle auth  ", Style::default().fg(Color::DarkGray)),
                Span::styled("Esc", Style::default().fg(Color::Yellow)),
                Span::styled(" back", Style::default().fg(Color::DarkGray)),
            ]))
            .centered(),
            chunks[0],
        );

        frame.render_widget(
            Self::render_field(
                "Host",
                &self.host,
                self.focused == Field::Host,
                "my-server",
                false,
                true,
            ),
            chunks[1],
        );
        frame.render_widget(
            Self::render_field(
                "Hostname",
                &self.hostname,
                self.focused == Field::Hostname,
                "192.168.1.1",
                false,
                true,
            ),
            chunks[2],
        );
        frame.render_widget(
            Self::render_field(
                "Port",
                &self.port,
                self.focused == Field::Port,
                "22",
                false,
                false,
            ),
            chunks[3],
        );
        frame.render_widget(
            Self::render_field(
                "User",
                &self.user,
                self.focused == Field::User,
                "root",
                false,
                false,
            ),
            chunks[4],
        );

        // Auth method as tabs
        let selected_tab = match self.auth_method {
            AuthMethod::Password => 0,
            AuthMethod::SshKey => 1,
        };
        let tabs = Tabs::new(vec!["Password", "SSH Key"])
            .select(selected_tab)
            .highlight_style(
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )
            .style(Style::default().fg(Color::DarkGray))
            .divider("│")
            .block(
                Block::default()
                    .title(" Auth Method ")
                    .borders(Borders::ALL)
                    .border_style(if self.focused == Field::AuthMethod {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    }),
            );
        frame.render_widget(tabs, chunks[5]);

        let (auth_label, auth_placeholder, auth_masked) = match self.auth_method {
            AuthMethod::Password => ("Password", "••••••••", true),
            AuthMethod::SshKey => ("SSH Key Path", "~/.ssh/id_rsa", false),
        };
        frame.render_widget(
            Self::render_field(
                auth_label,
                &self.auth_value,
                self.focused == Field::AuthValue,
                auth_placeholder,
                auth_masked,
                false,
            ),
            chunks[6],
        );

        // Error
        frame.render_widget(
            Paragraph::new(self.error.as_deref().unwrap_or(""))
                .style(Style::default().fg(Color::Red))
                .centered(),
            chunks[8],
        );
    }

    pub fn match_key(&mut self, key_code: KeyCode) -> Option<Screen> {
        match key_code {
            KeyCode::Esc => return Some(Screen::Intro(IntroScreen::default())),

            KeyCode::Tab | KeyCode::Down => self.focused = self.focused.next(),
            KeyCode::BackTab | KeyCode::Up => self.focused = self.focused.prev(),

            KeyCode::Left | KeyCode::Right if self.focused == Field::AuthMethod => {
                self.auth_method = match self.auth_method {
                    AuthMethod::Password => AuthMethod::SshKey,
                    AuthMethod::SshKey => AuthMethod::Password,
                };
                self.auth_value.clear();
            }

            KeyCode::Backspace => {
                if let Some(field) = self.active_field_mut() {
                    field.pop();
                }
            }

            KeyCode::Char(c) if self.focused != Field::AuthMethod => {
                if let Some(field) = self.active_field_mut() {
                    field.push(c);
                }
            }

            KeyCode::Enter => {
                if self.host.is_empty() || self.hostname.is_empty() {
                    self.error = Some("Host and Hostname are required".into());
                } else {
                    return Some(Screen::Intro(IntroScreen::default()));
                }
            }

            _ => {}
        }
        None
    }
}
