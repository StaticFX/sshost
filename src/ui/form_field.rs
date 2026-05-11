use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

pub fn render_field<'a>(
    label: &'a str,
    value: &'a str,
    is_focused: bool,
    placeholder: &'a str,
    required: bool,
) -> Paragraph<'a> {
    let border_style = if is_focused {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let display: String = if value.is_empty() {
        if is_focused {
            "\u{2588}".to_string()
        } else {
            placeholder.to_string()
        }
    } else if is_focused {
        format!("{value}\u{2588}")
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
