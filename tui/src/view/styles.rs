use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

// Theme system
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemePalette {
    pub primary: Color,
    pub primary_light: Color,
    pub accent: Color,
    pub shortcut_key: Color,
    pub green: Color,
    pub yellow: Color,
    pub text: Color,
    pub text_secondary: Color,
    pub text_dimmed: Color,
    pub bg: Color,
    pub bg_highlight: Color,
    pub error: Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    #[default]
    Default,
    CatppuccinLatte,
    TokyoNight,
    Dracula,
    Gruvbox,
    Nord,
    OneDark,
    SolarizedDark,
    SolarizedLight,
}

impl Theme {
    pub const ALL: &'static [Theme] = &[
        Theme::Default,
        Theme::CatppuccinLatte,
        Theme::TokyoNight,
        Theme::Dracula,
        Theme::Gruvbox,
        Theme::Nord,
        Theme::OneDark,
        Theme::SolarizedDark,
        Theme::SolarizedLight,
    ];

    pub fn name(&self) -> &str {
        match self {
            Theme::Default => "Default",
            Theme::CatppuccinLatte => "Catppuccin Latte",
            Theme::TokyoNight => "Tokyo Night",
            Theme::Dracula => "Dracula",
            Theme::Gruvbox => "Gruvbox",
            Theme::Nord => "Nord",
            Theme::OneDark => "One Dark",
            Theme::SolarizedDark => "Solarized Dark",
            Theme::SolarizedLight => "Solarized Light",
        }
    }

    pub fn id(&self) -> &str {
        match self {
            Theme::Default => "default",
            Theme::CatppuccinLatte => "catppuccin-latte",
            Theme::TokyoNight => "tokyo-night",
            Theme::Dracula => "dracula",
            Theme::Gruvbox => "gruvbox",
            Theme::Nord => "nord",
            Theme::OneDark => "one-dark",
            Theme::SolarizedDark => "solarized-dark",
            Theme::SolarizedLight => "solarized-light",
        }
    }

    pub fn from_id(id: &str) -> Option<Theme> {
        Theme::ALL.iter().find(|t| t.id() == id).copied()
    }

    pub fn palette(&self) -> ThemePalette {
        match self {
            Theme::Default => ThemePalette {
                primary: Color::Rgb(200, 160, 255),
                primary_light: Color::Rgb(220, 190, 255),
                accent: Color::Rgb(180, 130, 255),
                shortcut_key: Color::Rgb(255, 120, 200),
                green: Color::Rgb(150, 230, 150),
                yellow: Color::Rgb(255, 230, 150),
                text: Color::White,
                text_secondary: Color::Rgb(180, 180, 180),
                text_dimmed: Color::Rgb(90, 90, 90),
                bg: Color::Rgb(20, 15, 30),
                bg_highlight: Color::Rgb(60, 45, 90),
                error: Color::Rgb(255, 120, 120),
            },
            Theme::CatppuccinLatte => ThemePalette {
                primary: Color::Rgb(136, 57, 239),         // mauve
                primary_light: Color::Rgb(220, 138, 120),  // rosewater
                accent: Color::Rgb(30, 102, 245),          // blue
                shortcut_key: Color::Rgb(223, 142, 29),    // yellow
                green: Color::Rgb(64, 160, 43),            // green
                yellow: Color::Rgb(223, 142, 29),          // yellow
                text: Color::Rgb(76, 79, 105),             // text
                text_secondary: Color::Rgb(108, 111, 133), // subtext0
                text_dimmed: Color::Rgb(172, 176, 190),    // surface2
                bg: Color::Rgb(239, 241, 245),             // base
                bg_highlight: Color::Rgb(204, 208, 218),   // surface0
                error: Color::Rgb(210, 15, 57),            // red
            },
            Theme::TokyoNight => ThemePalette {
                primary: Color::Rgb(187, 154, 247), // purple
                primary_light: Color::Rgb(199, 175, 247),
                accent: Color::Rgb(122, 162, 247),       // blue
                shortcut_key: Color::Rgb(224, 175, 104), // orange
                green: Color::Rgb(158, 206, 106),        // green
                yellow: Color::Rgb(224, 175, 104),       // orange
                text: Color::Rgb(192, 202, 245),         // fg
                text_secondary: Color::Rgb(134, 150, 190),
                text_dimmed: Color::Rgb(65, 72, 104), // comment
                bg: Color::Rgb(26, 27, 38),           // bg
                bg_highlight: Color::Rgb(41, 46, 66), // bg_highlight
                error: Color::Rgb(247, 118, 142),     // red
            },
            Theme::Dracula => ThemePalette {
                primary: Color::Rgb(189, 147, 249),       // purple
                primary_light: Color::Rgb(255, 121, 198), // pink
                accent: Color::Rgb(139, 233, 253),        // cyan
                shortcut_key: Color::Rgb(255, 184, 108),  // orange
                green: Color::Rgb(80, 250, 123),          // green
                yellow: Color::Rgb(241, 250, 140),        // yellow
                text: Color::Rgb(248, 248, 242),          // fg
                text_secondary: Color::Rgb(189, 193, 207),
                text_dimmed: Color::Rgb(98, 114, 164), // comment
                bg: Color::Rgb(40, 42, 54),            // bg
                bg_highlight: Color::Rgb(68, 71, 90),  // current_line
                error: Color::Rgb(255, 85, 85),        // red
            },
            Theme::Gruvbox => ThemePalette {
                primary: Color::Rgb(211, 134, 155),        // purple
                primary_light: Color::Rgb(235, 219, 178),  // fg
                accent: Color::Rgb(131, 165, 152),         // aqua
                shortcut_key: Color::Rgb(254, 128, 25),    // orange
                green: Color::Rgb(184, 187, 38),           // green
                yellow: Color::Rgb(250, 189, 47),          // yellow
                text: Color::Rgb(235, 219, 178),           // fg
                text_secondary: Color::Rgb(189, 174, 147), // fg3
                text_dimmed: Color::Rgb(102, 92, 84),      // bg4
                bg: Color::Rgb(40, 40, 40),                // bg
                bg_highlight: Color::Rgb(60, 56, 54),      // bg1
                error: Color::Rgb(251, 73, 52),            // red
            },
            Theme::Nord => ThemePalette {
                primary: Color::Rgb(180, 142, 173),        // purple (nord15)
                primary_light: Color::Rgb(216, 222, 233),  // snow storm
                accent: Color::Rgb(129, 161, 193),         // frost blue (nord9)
                shortcut_key: Color::Rgb(208, 135, 112),   // aurora orange (nord12)
                green: Color::Rgb(163, 190, 140),          // aurora green (nord14)
                yellow: Color::Rgb(235, 203, 139),         // aurora yellow (nord13)
                text: Color::Rgb(236, 239, 244),           // snow storm (nord6)
                text_secondary: Color::Rgb(216, 222, 233), // snow storm (nord4)
                text_dimmed: Color::Rgb(76, 86, 106),      // polar night (nord3)
                bg: Color::Rgb(46, 52, 64),                // polar night (nord0)
                bg_highlight: Color::Rgb(59, 66, 82),      // polar night (nord1)
                error: Color::Rgb(191, 97, 106),           // aurora red (nord11)
            },
            Theme::OneDark => ThemePalette {
                primary: Color::Rgb(198, 120, 221), // purple
                primary_light: Color::Rgb(224, 175, 234),
                accent: Color::Rgb(97, 175, 239),        // blue
                shortcut_key: Color::Rgb(209, 154, 102), // orange
                green: Color::Rgb(152, 195, 121),        // green
                yellow: Color::Rgb(229, 192, 123),       // yellow
                text: Color::Rgb(171, 178, 191),         // fg
                text_secondary: Color::Rgb(139, 145, 157),
                text_dimmed: Color::Rgb(76, 82, 99), // comment
                bg: Color::Rgb(40, 44, 52),          // bg
                bg_highlight: Color::Rgb(50, 56, 66),
                error: Color::Rgb(224, 108, 117), // red
            },
            Theme::SolarizedDark => ThemePalette {
                primary: Color::Rgb(108, 113, 196),        // violet
                primary_light: Color::Rgb(147, 161, 161),  // base1
                accent: Color::Rgb(38, 139, 210),          // blue
                shortcut_key: Color::Rgb(203, 75, 22),     // orange
                green: Color::Rgb(133, 153, 0),            // green
                yellow: Color::Rgb(181, 137, 0),           // yellow
                text: Color::Rgb(131, 148, 150),           // base0
                text_secondary: Color::Rgb(101, 123, 131), // base00
                text_dimmed: Color::Rgb(7, 54, 66),        // base02
                bg: Color::Rgb(0, 43, 54),                 // base03
                bg_highlight: Color::Rgb(7, 54, 66),       // base02
                error: Color::Rgb(220, 50, 47),            // red
            },
            Theme::SolarizedLight => ThemePalette {
                primary: Color::Rgb(108, 113, 196),        // violet
                primary_light: Color::Rgb(88, 110, 117),   // base01
                accent: Color::Rgb(38, 139, 210),          // blue
                shortcut_key: Color::Rgb(203, 75, 22),     // orange
                green: Color::Rgb(133, 153, 0),            // green
                yellow: Color::Rgb(181, 137, 0),           // yellow
                text: Color::Rgb(101, 123, 131),           // base00
                text_secondary: Color::Rgb(131, 148, 150), // base0
                text_dimmed: Color::Rgb(198, 205, 203),    // base2
                bg: Color::Rgb(253, 246, 227),             // base3
                bg_highlight: Color::Rgb(238, 232, 213),   // base2
                error: Color::Rgb(220, 50, 47),            // red
            },
        }
    }
}

// Active theme (global)
use std::sync::atomic::{AtomicUsize, Ordering};

static ACTIVE_THEME_INDEX: AtomicUsize = AtomicUsize::new(0);

pub fn set_active_theme(theme: Theme) {
    let idx = Theme::ALL.iter().position(|t| *t == theme).unwrap_or(0);
    ACTIVE_THEME_INDEX.store(idx, Ordering::Relaxed);
}

pub fn active_theme() -> Theme {
    let idx = ACTIVE_THEME_INDEX.load(Ordering::Relaxed);
    Theme::ALL[idx]
}

pub fn palette() -> ThemePalette {
    active_theme().palette()
}

// Convenience color accessors
pub fn primary() -> Color {
    palette().primary
}
pub fn primary_light() -> Color {
    palette().primary_light
}
pub fn accent() -> Color {
    palette().accent
}
pub fn shortcut_key() -> Color {
    palette().shortcut_key
}
pub fn green() -> Color {
    palette().green
}
pub fn yellow() -> Color {
    palette().yellow
}
pub fn text_primary() -> Color {
    palette().text
}
pub fn text_secondary() -> Color {
    palette().text_secondary
}
pub fn text_dimmed() -> Color {
    palette().text_dimmed
}
pub fn bg_dark() -> Color {
    palette().bg
}
pub fn bg_highlight() -> Color {
    palette().bg_highlight
}
pub fn error_red() -> Color {
    palette().error
}

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
