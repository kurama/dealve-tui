use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

// Dealve color palette - Pastel theme (light colors for dark background)
pub const PURPLE_PRIMARY: Color = Color::Rgb(200, 160, 255); // Pastel lavender - main brand color
pub const PURPLE_LIGHT: Color = Color::Rgb(220, 190, 255); // Lighter pastel lavender - highlights
pub const PURPLE_ACCENT: Color = Color::Rgb(180, 130, 255); // Slightly stronger pastel for accents
pub const SHORTCUT_KEY: Color = Color::Rgb(255, 120, 200); // Pink/magenta for shortcut keys (btop style)
pub const ACCENT_GREEN: Color = Color::Rgb(150, 230, 150); // Pastel mint green - good deals
pub const ACCENT_YELLOW: Color = Color::Rgb(255, 230, 150); // Pastel gold/cream - medium deals
pub const TEXT_PRIMARY: Color = Color::White;
pub const TEXT_SECONDARY: Color = Color::Rgb(180, 180, 180); // Light gray
pub const TEXT_DIMMED: Color = Color::Rgb(90, 90, 90); // Dimmed text for background when menu open
pub const BG_DARK: Color = Color::Rgb(20, 15, 30); // Very dark purple background
pub const BG_HIGHLIGHT: Color = Color::Rgb(60, 45, 90); // Darker purple for selection highlight
pub const ERROR_RED: Color = Color::Rgb(255, 120, 120);

pub const ASCII_LOGO: [&str; 6] = [
    "██████╗ ███████╗ █████╗ ██╗    ██╗   ██╗███████╗",
    "██╔══██╗██╔════╝██╔══██╗██║    ██║   ██║██╔════╝",
    "██║  ██║█████╗  ███████║██║    ██║   ██║█████╗  ",
    "██║  ██║██╔══╝  ██╔══██║██║    ╚██╗ ██╔╝██╔══╝  ",
    "██████╔╝███████╗██║  ██║███████╗╚████╔╝ ███████╗",
    "╚═════╝ ╚══════╝╚═╝  ╚═╝╚══════╝ ╚═══╝  ╚══════╝",
];

/// Build a title with btop-style brackets
pub fn build_title(text: &str, border_color: Color, title_color: Color) -> Line<'static> {
    Line::from(vec![
        Span::styled("┐", Style::default().fg(border_color)),
        Span::styled(text.to_string(), Style::default().fg(title_color)),
        Span::styled("┌", Style::default().fg(border_color)),
    ])
}

/// Calculate vertical padding to center text within an area
pub fn vertical_padding(area_height: u16, text_lines: u16) -> String {
    let inner_height = area_height.saturating_sub(2); // Account for borders
    let padding = inner_height.saturating_sub(text_lines) / 2;
    "\n".repeat(padding as usize)
}

pub trait CurrencySymbol {
    fn currency_symbol(&self) -> &str;
}

impl CurrencySymbol for dealve_core::models::Price {
    fn currency_symbol(&self) -> &str {
        match self.currency.as_str() {
            "USD" => "$",
            "EUR" => "€",
            "GBP" => "£",
            _ => &self.currency,
        }
    }
}
