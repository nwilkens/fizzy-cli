use ratatui::style::{Color, Modifier, Style};

use crate::models::ColumnColor;

/// Map Fizzy column color names to terminal colors (dark-background friendly).
pub fn column_color(color: &ColumnColor) -> Color {
    let name = match color {
        ColumnColor::Plain(s) => s.as_str(),
        ColumnColor::Structured { name, .. } => name.as_str(),
    };
    match name.to_lowercase().as_str() {
        "blue" => Color::Rgb(100, 149, 237),
        "gray" | "grey" => Color::Rgb(140, 140, 155),
        "tan" => Color::Rgb(210, 180, 140),
        "yellow" => Color::Rgb(230, 190, 50),
        "lime" => Color::Rgb(130, 200, 80),
        "aqua" => Color::Rgb(80, 200, 190),
        "violet" => Color::Rgb(160, 120, 210),
        "purple" => Color::Rgb(140, 100, 190),
        "pink" => Color::Rgb(210, 130, 160),
        _ => Color::Rgb(180, 180, 190),
    }
}

pub fn selected_card() -> Style {
    Style::default().fg(Color::Black).bg(Color::White)
}

pub fn golden_accent() -> Style {
    Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD)
}

pub fn dim_meta() -> Style {
    Style::default().fg(Color::DarkGray)
}

pub fn blocked_indicator() -> Style {
    Style::default().fg(Color::Red)
}

pub fn ready_indicator() -> Style {
    Style::default().fg(Color::Green)
}

pub fn status_info() -> Style {
    Style::default().fg(Color::Cyan)
}

pub fn status_success() -> Style {
    Style::default().fg(Color::Green)
}

pub fn status_error() -> Style {
    Style::default().fg(Color::Red)
}

pub fn help_key() -> Style {
    Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD)
}

pub fn help_desc() -> Style {
    Style::default().fg(Color::Rgb(180, 180, 190))
}

pub fn board_title() -> Style {
    Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD)
}

pub fn column_border_selected() -> Style {
    Style::default().fg(Color::White)
}

pub fn column_border_normal() -> Style {
    Style::default().fg(Color::Rgb(60, 60, 70))
}
