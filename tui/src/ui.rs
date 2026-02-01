use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use crate::app::{App, MenuItem, OptionsTab, Popup};
use dealve_core::models::Platform;

// Dealve color palette - Pastel theme (light colors for dark background)
const PURPLE_PRIMARY: Color = Color::Rgb(200, 160, 255);   // Pastel lavender - main brand color
const PURPLE_LIGHT: Color = Color::Rgb(220, 190, 255);     // Lighter pastel lavender - highlights
const PURPLE_ACCENT: Color = Color::Rgb(180, 130, 255);    // Slightly stronger pastel for accents
const ACCENT_GREEN: Color = Color::Rgb(150, 230, 150);     // Pastel mint green - good deals
const ACCENT_YELLOW: Color = Color::Rgb(255, 230, 150);    // Pastel gold/cream - medium deals
const TEXT_PRIMARY: Color = Color::White;
const TEXT_SECONDARY: Color = Color::Rgb(180, 180, 180);   // Light gray
const TEXT_DIMMED: Color = Color::Rgb(90, 90, 90);         // Dimmed text for background when menu open

const ASCII_LOGO: [&str; 6] = [
    "██████╗ ███████╗ █████╗ ██╗    ██╗   ██╗███████╗",
    "██╔══██╗██╔════╝██╔══██╗██║    ██║   ██║██╔════╝",
    "██║  ██║█████╗  ███████║██║    ██║   ██║█████╗  ",
    "██║  ██║██╔══╝  ██╔══██║██║    ╚██╗ ██╔╝██╔══╝  ",
    "██████╔╝███████╗██║  ██║███████╗╚████╔╝ ███████╗",
    "╚═════╝ ╚══════╝╚═╝  ╚═╝╚══════╝ ╚═══╝  ╚══════╝",
];

pub fn render(frame: &mut Frame, app: &mut App) {
    let dimmed = app.show_menu;
    render_main(frame, app, dimmed);

    if app.show_menu {
        render_menu_overlay(frame, app);
    }

    match app.popup {
        Popup::None => {}
        Popup::Options => render_options_popup(frame, app),
        Popup::Keybinds => render_keybinds_popup(frame),
    }
}

fn render_main(frame: &mut Frame, app: &mut App, dimmed: bool) {
    let area = frame.area();

    // Split horizontal: 50% left (deals), 50% right (details + chart)
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let left_panel = main_chunks[0];

    // Right panel: split vertical - details (40%), chart (60%)
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(main_chunks[1]);

    let details_panel = right_chunks[0];
    let chart_panel = right_chunks[1];

    render_deals_list(frame, app, left_panel, dimmed);
    render_game_details(frame, app, details_panel, dimmed);
    render_price_chart(frame, app, chart_panel, dimmed);

    if app.show_platform_dropdown {
        render_dropdown(frame, app, left_panel);
    }
}

fn render_deals_list(frame: &mut Frame, app: &mut App, area: Rect, dimmed: bool) {
    // When dimmed, ALL colors become TEXT_DIMMED
    let text_color = if dimmed { TEXT_DIMMED } else { TEXT_PRIMARY };
    let border_color = if dimmed { TEXT_DIMMED } else { PURPLE_ACCENT };
    let title_color = if dimmed { TEXT_DIMMED } else { PURPLE_LIGHT };

    if app.loading {
        let spinner = app.spinner_char();
        let loading = Paragraph::new(format!("{} Loading deals...", spinner))
            .alignment(Alignment::Center)
            .style(Style::default().fg(text_color))
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .title(format!(" Deals [{}] ", app.platform_filter.name())));
        frame.render_widget(loading, area);
        return;
    }

    if let Some(error) = &app.error {
        let error_msg = Paragraph::new(format!("Error: {}", error))
            .style(Style::default().fg(Color::Red))
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .title(" Error "));
        frame.render_widget(error_msg, area);
        return;
    }

    let filtered_deals = app.filtered_deals();

    if filtered_deals.is_empty() {
        let empty = Paragraph::new("No deals found")
            .alignment(Alignment::Center)
            .style(Style::default().fg(text_color))
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .title(format!(" Deals [{}] ", app.platform_filter.name())));
        frame.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = filtered_deals
        .iter()
        .map(|deal| {
            let price_str = format!("{}{:.2}", deal.price.currency_symbol(), deal.price.amount);
            let discount_str = format!("-{}%", deal.price.discount);

            // Calculate available width for title
            let available_width = area.width.saturating_sub(20) as usize;
            let title = truncate(&deal.title, available_width);

            // Check if this is an all-time low - highlight in purple!
            let is_atl = deal.history_low
                .map(|low| (low - deal.price.amount).abs() < 0.01)
                .unwrap_or(false);

            // Color scheme based on deal quality (respects dimmed state)
            let (item_title_color, price_color, discount_color) = if dimmed {
                (TEXT_DIMMED, TEXT_DIMMED, TEXT_DIMMED)
            } else if is_atl {
                // All-time low: purple theme
                (PURPLE_LIGHT, PURPLE_PRIMARY, PURPLE_PRIMARY)
            } else if deal.price.discount >= 75 {
                (text_color, ACCENT_GREEN, ACCENT_GREEN)
            } else if deal.price.discount >= 50 {
                (text_color, ACCENT_YELLOW, ACCENT_YELLOW)
            } else {
                (text_color, text_color, TEXT_SECONDARY)
            };

            let mut spans = vec![
                Span::styled(format!("{:<width$}", title, width = available_width), Style::default().fg(item_title_color)),
                Span::styled(format!("{:>8}", price_str), Style::default().fg(price_color)),
                Span::styled(format!("{:>6}", discount_str), Style::default().fg(discount_color)),
            ];

            // Add ATL indicator
            if is_atl {
                let atl_color = if dimmed { TEXT_DIMMED } else { PURPLE_PRIMARY };
                spans.push(Span::styled(" ATL", Style::default().fg(atl_color).add_modifier(Modifier::BOLD)));
            }

            let content = Line::from(spans);
            ListItem::new(content)
        })
        .collect();

    let highlight_style = if dimmed {
        Style::default().fg(TEXT_DIMMED)
    } else {
        Style::default().bg(PURPLE_ACCENT).fg(TEXT_PRIMARY)
    };

    let deals_list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(Span::styled(
                format!(" Deals [{}] ", app.platform_filter.name()),
                Style::default().fg(title_color)
            )))
        .highlight_style(highlight_style)
        .highlight_symbol("> ");

    frame.render_stateful_widget(deals_list, area, &mut app.list_state);
}

fn render_game_details(frame: &mut Frame, app: &App, area: Rect, dimmed: bool) {
    // When dimmed, ALL colors become TEXT_DIMMED
    let text_color = if dimmed { TEXT_DIMMED } else { TEXT_PRIMARY };
    let label_color = if dimmed { TEXT_DIMMED } else { PURPLE_LIGHT };
    let border_color = if dimmed { TEXT_DIMMED } else { PURPLE_ACCENT };
    let title_color = if dimmed { TEXT_DIMMED } else { PURPLE_LIGHT };
    let purple_color = if dimmed { TEXT_DIMMED } else { PURPLE_PRIMARY };
    let green_color = if dimmed { TEXT_DIMMED } else { ACCENT_GREEN };
    let yellow_color = if dimmed { TEXT_DIMMED } else { ACCENT_YELLOW };
    let secondary_color = if dimmed { TEXT_DIMMED } else { TEXT_SECONDARY };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(" Game Details ", Style::default().fg(title_color)));

    // Get selected deal for basic info
    let selected_deal = app.selected_deal();

    if selected_deal.is_none() {
        let empty = Paragraph::new("Select a deal to view details")
            .alignment(Alignment::Center)
            .style(Style::default().fg(secondary_color))
            .block(block);
        frame.render_widget(empty, area);
        return;
    }

    let deal = selected_deal.unwrap();
    let game_info = app.selected_game_info();
    let is_loading = app.loading_game_info.as_ref() == Some(&deal.id);
    let mut lines: Vec<Line> = Vec::new();

    // Check if ATL
    let is_atl = deal.history_low
        .map(|low| (low - deal.price.amount).abs() < 0.01)
        .unwrap_or(false);

    // Title with ATL badge if applicable
    if is_atl {
        lines.push(Line::from(vec![
            Span::styled(">> ALL-TIME LOW <<", Style::default().fg(purple_color).add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from("")); // Spacer
    }

    // Title
    lines.push(Line::from(vec![
        Span::styled(&deal.title, Style::default().fg(text_color).add_modifier(Modifier::BOLD)),
    ]));

    // Release date and developers from game info
    if let Some(info) = game_info {
        if let Some(ref release_date) = info.release_date {
            lines.push(Line::from(vec![
                Span::styled("Released: ", Style::default().fg(label_color)),
                Span::styled(release_date, Style::default().fg(secondary_color)),
            ]));
        }
        if !info.developers.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("Developer: ", Style::default().fg(label_color)),
                Span::styled(info.developers.join(", "), Style::default().fg(secondary_color)),
            ]));
        }
        if !info.publishers.is_empty() && info.publishers != info.developers {
            lines.push(Line::from(vec![
                Span::styled("Publisher: ", Style::default().fg(label_color)),
                Span::styled(info.publishers.join(", "), Style::default().fg(secondary_color)),
            ]));
        }
    } else if is_loading {
        lines.push(Line::from(vec![
            Span::styled("Loading game info...", Style::default().fg(secondary_color)),
        ]));
    }

    lines.push(Line::from("")); // Spacer

    // Shop
    lines.push(Line::from(vec![
        Span::styled("Shop: ", Style::default().fg(label_color)),
        Span::styled(&deal.shop.name, Style::default().fg(text_color)),
    ]));

    // Price section
    let regular_str = format!("{}{:.2}", deal.price.currency_symbol(), deal.regular_price);
    let price_str = format!("{}{:.2}", deal.price.currency_symbol(), deal.price.amount);
    let discount_str = format!("-{}%", deal.price.discount);
    let price_color = if is_atl { purple_color } else { green_color };

    lines.push(Line::from(vec![
        Span::styled(regular_str, Style::default().fg(secondary_color).add_modifier(Modifier::CROSSED_OUT)),
        Span::styled(" -> ", Style::default().fg(secondary_color)),
        Span::styled(price_str, Style::default().fg(price_color).add_modifier(Modifier::BOLD)),
        Span::styled(format!(" ({})", discount_str), Style::default().fg(yellow_color)),
    ]));

    // Savings
    let savings = deal.regular_price - deal.price.amount;
    if savings > 0.0 {
        let savings_str = format!("{}{:.2}", deal.price.currency_symbol(), savings);
        lines.push(Line::from(vec![
            Span::styled("You save ", Style::default().fg(secondary_color)),
            Span::styled(savings_str, Style::default().fg(green_color).add_modifier(Modifier::BOLD)),
        ]));
    }

    // History low
    if let Some(low) = deal.history_low {
        let low_str = format!("{}{:.2}", deal.price.currency_symbol(), low);
        let low_price_color = if is_atl { purple_color } else { text_color };
        lines.push(Line::from(vec![
            Span::styled("History low: ", Style::default().fg(label_color)),
            Span::styled(low_str, Style::default().fg(low_price_color)),
            if is_atl && !dimmed {
                Span::styled(" (current!)", Style::default().fg(purple_color))
            } else {
                Span::raw("")
            },
        ]));
    }

    // Tags from game info
    if let Some(info) = game_info {
        if !info.tags.is_empty() {
            lines.push(Line::from("")); // Spacer
            let tags_str = info.tags.iter().take(5).cloned().collect::<Vec<_>>().join(" | ");
            lines.push(Line::from(vec![
                Span::styled(tags_str, Style::default().fg(secondary_color)),
            ]));
        }
    }

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}

fn render_price_chart(frame: &mut Frame, _app: &App, area: Rect, dimmed: bool) {
    let text_color = if dimmed { TEXT_DIMMED } else { TEXT_SECONDARY };
    let border_color = if dimmed { TEXT_DIMMED } else { PURPLE_ACCENT };
    let title_color = if dimmed { TEXT_DIMMED } else { PURPLE_LIGHT };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(" Price History ", Style::default().fg(title_color)));

    // Price history chart placeholder
    let content = vec![
        "",
        "  Price history chart coming soon...",
        "",
        "  Visit IsThereAnyDeal.com for full",
        "  price history information.",
    ];

    let placeholder = Paragraph::new(content.join("\n"))
        .alignment(Alignment::Center)
        .style(Style::default().fg(text_color))
        .block(block);

    frame.render_widget(placeholder, area);
}

fn render_menu_overlay(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let logo_width = 50u16;
    let logo_height = 6u16;
    let menu_width = 18u16;
    let menu_height = 6u16;
    let total_height = logo_height + 1 + menu_height;

    let start_y = area.height.saturating_sub(total_height) / 2;

    let logo_x = area.width.saturating_sub(logo_width) / 2;
    let logo_area = Rect::new(logo_x, start_y, logo_width, logo_height);

    frame.render_widget(Clear, logo_area);

    // Logo in purple gradient effect
    let logo_lines: Vec<Line> = ASCII_LOGO
        .iter()
        .map(|line| Line::from(Span::styled(*line, Style::default().fg(PURPLE_PRIMARY))))
        .collect();
    let logo = Paragraph::new(logo_lines)
        .alignment(Alignment::Center);
    frame.render_widget(logo, logo_area);

    let menu_x = area.width.saturating_sub(menu_width) / 2;
    let menu_y = start_y + logo_height + 1;
    let menu_area = Rect::new(menu_x, menu_y, menu_width, menu_height);

    frame.render_widget(Clear, menu_area);

    let items: Vec<ListItem> = MenuItem::ALL
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if i == app.menu_selected {
                Style::default().bg(PURPLE_PRIMARY).fg(TEXT_PRIMARY).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(TEXT_SECONDARY)
            };
            let prefix = if i == app.menu_selected { "> " } else { "  " };
            ListItem::new(format!("{}{}", prefix, item.name())).style(style)
        })
        .collect();

    let menu = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(PURPLE_LIGHT)));

    frame.render_widget(menu, menu_area);
}

fn render_options_popup(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let popup_width = 50u16;
    let popup_height = 22u16;
    let popup_x = area.width.saturating_sub(popup_width) / 2;
    let popup_y = area.height.saturating_sub(popup_height) / 2;
    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    // Main popup block
    let block = Block::default()
        .title(Span::styled(" Options ", Style::default().fg(PURPLE_LIGHT)))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(PURPLE_ACCENT));
    frame.render_widget(block, popup_area);

    // Inner area
    let inner = Rect::new(
        popup_area.x + 1,
        popup_area.y + 1,
        popup_area.width - 2,
        popup_area.height - 2,
    );

    // Split into tabs bar and content
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(0)])
        .split(inner);

    // Render tabs bar
    let tabs: Vec<Span> = OptionsTab::ALL
        .iter()
        .enumerate()
        .map(|(i, tab)| {
            if i == app.options.current_tab {
                Span::styled(
                    format!(" {} ", tab.name()),
                    Style::default().fg(TEXT_PRIMARY).bg(PURPLE_ACCENT).add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(
                    format!(" {} ", tab.name()),
                    Style::default().fg(TEXT_SECONDARY),
                )
            }
        })
        .collect();

    let tabs_line = Line::from(tabs);
    let tabs_para = Paragraph::new(tabs_line);
    frame.render_widget(tabs_para, chunks[0]);

    // Render tab content
    let content_area = chunks[1];
    match OptionsTab::ALL[app.options.current_tab] {
        OptionsTab::Platforms => render_platforms_tab(frame, app, content_area),
    }
}

fn render_platforms_tab(frame: &mut Frame, app: &App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();

    // Default platform selector (index 0)
    let is_default_selected = app.options.platform_list_index == 0;
    let default_style = if is_default_selected {
        Style::default().fg(TEXT_PRIMARY).bg(PURPLE_ACCENT)
    } else {
        Style::default().fg(TEXT_PRIMARY)
    };
    lines.push(Line::from(vec![
        Span::styled("Default: ", Style::default().fg(PURPLE_LIGHT)),
        Span::styled(
            format!("{} ", app.options.default_platform.name()),
            default_style,
        ),
        if is_default_selected {
            Span::styled("[Enter to change]", Style::default().fg(TEXT_SECONDARY))
        } else {
            Span::raw("")
        },
    ]));

    lines.push(Line::from("")); // Spacer
    lines.push(Line::from(Span::styled(
        "Enabled platforms:",
        Style::default().fg(PURPLE_LIGHT),
    )));
    lines.push(Line::from("")); // Spacer

    // Platform list with checkboxes (starting at index 1)
    for (i, platform) in Platform::ALL.iter().enumerate() {
        let list_index = i + 1;
        let is_selected = app.options.platform_list_index == list_index;
        let is_enabled = app.options.enabled_platforms.contains(platform);

        let checkbox = if *platform == Platform::All {
            "[*]" // Always enabled, can't disable
        } else if is_enabled {
            "[x]"
        } else {
            "[ ]"
        };

        let line_style = if is_selected {
            Style::default().fg(TEXT_PRIMARY).bg(PURPLE_ACCENT)
        } else if is_enabled {
            Style::default().fg(TEXT_PRIMARY)
        } else {
            Style::default().fg(TEXT_DIMMED)
        };

        lines.push(Line::from(Span::styled(
            format!(" {} {}", checkbox, platform.name()),
            line_style,
        )));
    }

    lines.push(Line::from("")); // Spacer
    lines.push(Line::from(Span::styled(
        "[Space/Enter] Toggle  [Tab] Next tab  [Esc] Close",
        Style::default().fg(TEXT_SECONDARY),
    )));

    let paragraph = Paragraph::new(lines).scroll((
        // Scroll if needed to keep selection visible
        if app.options.platform_list_index > 12 {
            (app.options.platform_list_index - 12) as u16
        } else {
            0
        },
        0,
    ));
    frame.render_widget(paragraph, area);
}

fn render_keybinds_popup(frame: &mut Frame) {
    let area = frame.area();
    let popup_width = 45u16;
    let popup_height = 14u16;
    let popup_x = area.width.saturating_sub(popup_width) / 2;
    let popup_y = area.height.saturating_sub(popup_height) / 2;
    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let content = vec![
        "",
        "  [Up/Down] or [j/k]  Navigate",
        "  [Enter]             Open deal / Select",
        "  [p]                 Open platform filter",
        "  [r]                 Refresh deals",
        "  [Esc]               Menu / Close popup",
        "  [q]                 Quit (from menu)",
        "",
        "  [Esc] Close",
    ];

    let popup = Paragraph::new(content.join("\n"))
        .style(Style::default().fg(TEXT_PRIMARY))
        .block(Block::default()
            .title(Span::styled(" Keybinds ", Style::default().fg(PURPLE_LIGHT)))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(PURPLE_PRIMARY)));

    frame.render_widget(popup, popup_area);
}

fn render_dropdown(frame: &mut Frame, app: &mut App, list_area: Rect) {
    let enabled_platforms = app.enabled_platforms();
    let dropdown_width = 20u16;
    let max_visible = enabled_platforms.len().min(15) as u16;
    let dropdown_height = max_visible + 2;

    let dropdown_x = list_area.x + 2;
    let dropdown_y = list_area.y + 1;

    let frame_height = frame.area().height;
    let available_height = frame_height.saturating_sub(dropdown_y);
    let dropdown_height = dropdown_height.min(available_height);

    let dropdown_area = Rect::new(
        dropdown_x,
        dropdown_y,
        dropdown_width,
        dropdown_height,
    );

    frame.render_widget(Clear, dropdown_area);

    let items: Vec<ListItem> = enabled_platforms
        .iter()
        .map(|platform| {
            ListItem::new(format!("  {}", platform.name()))
                .style(Style::default().fg(TEXT_PRIMARY))
        })
        .collect();

    let mut list_state = ratatui::widgets::ListState::default();
    list_state.select(Some(app.dropdown_selected));

    let dropdown = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(PURPLE_ACCENT))
            .title(Span::styled(" Platform ", Style::default().fg(PURPLE_LIGHT))))
        .highlight_style(Style::default().bg(PURPLE_ACCENT).fg(TEXT_PRIMARY))
        .highlight_symbol("> ");

    frame.render_stateful_widget(dropdown, dropdown_area, &mut list_state);
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_len.saturating_sub(3)).collect();
        format!("{}...", truncated)
    }
}

trait CurrencySymbol {
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
