use iced::{Color, Theme};

pub fn primary_color() -> Color {
    Color::from_rgb(0.2, 0.5, 0.8)
}

pub fn secondary_color() -> Color {
    Color::from_rgb(0.0, 0.3, 0.6)
}

pub fn accent_color() -> Color {
    Color::from_rgb(0.9, 0.4, 0.1)
}

pub fn background_color() -> Color {
    Color::from_rgb(0.95, 0.95, 0.95)
}

pub fn card_color() -> Color {
    Color::WHITE
}

pub fn text_color() -> Color {
    Color::from_rgb(0.2, 0.2, 0.2)
}

pub fn get_theme() -> Theme {
    Theme::Light
} 