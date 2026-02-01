use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Table, Row, Cell},
    Frame,
};

use crate::app::{App, MenuItem, OptionsTab, Popup};
use dealve_core::models::{Platform, Region};

// Dealve color palette - Pastel theme (light colors for dark background)
const PURPLE_PRIMARY: Color = Color::Rgb(200, 160, 255);   // Pastel lavender - main brand color
const PURPLE_LIGHT: Color = Color::Rgb(220, 190, 255);     // Lighter pastel lavender - highlights
const PURPLE_ACCENT: Color = Color::Rgb(180, 130, 255);    // Slightly stronger pastel for accents
const SHORTCUT_KEY: Color = Color::Rgb(255, 120, 200);     // Pink/magenta for shortcut keys (btop style)
const ACCENT_GREEN: Color = Color::Rgb(150, 230, 150);     // Pastel mint green - good deals
const ACCENT_YELLOW: Color = Color::Rgb(255, 230, 150);    // Pastel gold/cream - medium deals
const TEXT_PRIMARY: Color = Color::White;
const TEXT_SECONDARY: Color = Color::Rgb(180, 180, 180);   // Light gray
const TEXT_DIMMED: Color = Color::Rgb(90, 90, 90);         // Dimmed text for background when menu open
const BG_DARK: Color = Color::Rgb(20, 15, 30);             // Very dark purple background

const ASCII_LOGO: [&str; 6] = [
    "██████╗ ███████╗ █████╗ ██╗    ██╗   ██╗███████╗",
    "██╔══██╗██╔════╝██╔══██╗██║    ██║   ██║██╔════╝",
    "██║  ██║█████╗  ███████║██║    ██║   ██║█████╗  ",
    "██║  ██║██╔══╝  ██╔══██║██║    ╚██╗ ██╔╝██╔══╝  ",
    "██████╔╝███████╗██║  ██║███████╗╚████╔╝ ███████╗",
    "╚═════╝ ╚══════╝╚═╝  ╚═╝╚══════╝ ╚═══╝  ╚══════╝",
];

pub fn render(frame: &mut Frame, app: &mut App) {
    // Fill entire screen with dark purple background
    let bg_block = Block::default().style(Style::default().bg(BG_DARK));
    frame.render_widget(bg_block, frame.area());

    let dimmed = app.show_menu;
    render_main(frame, app, dimmed);

    if app.show_menu {
        render_menu_overlay(frame, app);
    }

    match app.popup {
        Popup::None => {}
        Popup::Options => render_options_popup(frame, app),
        Popup::Keybinds => render_keybinds_popup(frame),
        Popup::Platform => render_platform_popup(frame, app),
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
}

fn render_deals_list(frame: &mut Frame, app: &mut App, area: Rect, dimmed: bool) {
    // When dimmed, ALL colors become TEXT_DIMMED
    let text_color = if dimmed { TEXT_DIMMED } else { TEXT_PRIMARY };
    let border_color = if dimmed { TEXT_DIMMED } else { PURPLE_ACCENT };
    let title_color = if dimmed { TEXT_DIMMED } else { TEXT_PRIMARY };

    // Build title with btop-style brackets
    let title_text = format!("Deals [{}]", app.platform_filter.name());
    let title = build_title(&title_text, border_color, title_color);

    // Build bottom status bar (btop style)
    let status_line = build_status_line(app, dimmed);

    if app.loading {
        let spinner = app.spinner_char();
        let padding = vertical_padding(area.height, 1);
        let loading = Paragraph::new(format!("{}{} Loading deals...", padding, spinner))
            .alignment(Alignment::Center)
            .style(Style::default().fg(text_color))
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .title(title)
                .title_bottom(status_line));
        frame.render_widget(loading, area);
        return;
    }

    if let Some(error) = &app.error {
        let error_title = build_title("Error", border_color, title_color);
        let error_msg = Paragraph::new(format!("Error: {}", error))
            .style(Style::default().fg(Color::Red))
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .title(error_title)
                .title_bottom(status_line));
        frame.render_widget(error_msg, area);
        return;
    }

    let filtered_deals = app.filtered_deals();

    if filtered_deals.is_empty() {
        let padding = vertical_padding(area.height, 1);
        let empty = Paragraph::new(format!("{}No deals found", padding))
            .alignment(Alignment::Center)
            .style(Style::default().fg(text_color))
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .title(title)
                .title_bottom(status_line));
        frame.render_widget(empty, area);
        return;
    }

    // Build table header
    let header_color = if dimmed { TEXT_DIMMED } else { TEXT_PRIMARY };
    let header = Row::new(vec![
        Cell::from("Title").style(Style::default().fg(header_color)),
        Cell::from("Price").style(Style::default().fg(header_color)),
        Cell::from("Deal").style(Style::default().fg(header_color)),
        Cell::from("").style(Style::default().fg(header_color)), // ATL column
    ]);

    // Build table rows
    let rows: Vec<Row> = filtered_deals
        .iter()
        .map(|deal| {
            let price_str = format!("{}{:.2}", deal.price.currency_symbol(), deal.price.amount);
            let discount_str = format!("-{}%", deal.price.discount);

            // Check if this is an all-time low - highlight in purple!
            let is_atl = deal.history_low
                .map(|low| (low - deal.price.amount).abs() < 0.01)
                .unwrap_or(false);

            // Color scheme based on deal quality (respects dimmed state)
            let (item_title_color, price_color, discount_color) = if dimmed {
                (TEXT_DIMMED, TEXT_DIMMED, TEXT_DIMMED)
            } else if is_atl {
                (TEXT_SECONDARY, PURPLE_PRIMARY, PURPLE_PRIMARY)
            } else if deal.price.discount >= 75 {
                (TEXT_SECONDARY, ACCENT_GREEN, ACCENT_GREEN)
            } else if deal.price.discount >= 50 {
                (TEXT_SECONDARY, ACCENT_YELLOW, ACCENT_YELLOW)
            } else {
                (TEXT_SECONDARY, TEXT_SECONDARY, TEXT_SECONDARY)
            };

            let atl_cell = if is_atl {
                let atl_color = if dimmed { TEXT_DIMMED } else { PURPLE_PRIMARY };
                Cell::from("ATL").style(Style::default().fg(atl_color).add_modifier(Modifier::BOLD))
            } else {
                Cell::from("")
            };

            Row::new(vec![
                Cell::from(deal.title.clone()).style(Style::default().fg(item_title_color)),
                Cell::from(price_str).style(Style::default().fg(price_color)),
                Cell::from(discount_str).style(Style::default().fg(discount_color)),
                atl_cell,
            ])
        })
        .collect();

    let highlight_style = if dimmed {
        Style::default().fg(TEXT_DIMMED)
    } else {
        Style::default().bg(PURPLE_ACCENT).fg(TEXT_PRIMARY)
    };

    // Save values for scrollbar before render
    let total_items = filtered_deals.len();
    let selected = app.table_state.selected().unwrap_or(0);

    // Counter for bottom right corner (white and bold)
    let counter_color = if dimmed { TEXT_DIMMED } else { TEXT_PRIMARY };
    let counter_text = if app.loading_more {
        format!(" {}/{}  {} ", selected + 1, total_items, app.spinner_char())
    } else if app.has_more_deals {
        format!(" {}/{}+ ", selected + 1, total_items)
    } else {
        format!(" {}/{} ", selected + 1, total_items)
    };
    let counter = Span::styled(counter_text, Style::default().fg(counter_color).add_modifier(Modifier::BOLD));

    // Column widths: Title takes remaining space, Price 8, Deal 6, ATL 4
    let widths = [
        Constraint::Min(20),
        Constraint::Length(10),
        Constraint::Length(7),
        Constraint::Length(4),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(title)
            .title_bottom(status_line)
            .title_bottom(Line::from(counter).alignment(Alignment::Right)))
        .row_highlight_style(highlight_style)
        .highlight_symbol("> ");

    frame.render_stateful_widget(table, area, &mut app.table_state);

    // Render scrollbar with pink arrows (shortcut style)
    let scrollbar_track_color = if dimmed { TEXT_DIMMED } else { PURPLE_ACCENT };
    let scrollbar_arrow_color = if dimmed { TEXT_DIMMED } else { SHORTCUT_KEY };

    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"))
        .track_symbol(Some("│"))
        .thumb_symbol("█")
        .style(Style::default().fg(scrollbar_track_color))
        .begin_style(Style::default().fg(scrollbar_arrow_color))
        .end_style(Style::default().fg(scrollbar_arrow_color));

    let mut scrollbar_state = ScrollbarState::new(total_items).position(selected);

    // Render scrollbar in the inner area (inside the border)
    let scrollbar_area = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: area.height.saturating_sub(1), // Don't overlap with bottom border
    };
    frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
}

/// Build status bar line with btop-style highlighted shortcut keys and separators
fn build_status_line(app: &App, dimmed: bool) -> Line<'static> {
    let text_color = if dimmed { TEXT_DIMMED } else { TEXT_PRIMARY };
    let shortcut_color = if dimmed { TEXT_DIMMED } else { SHORTCUT_KEY };
    let value_color = if dimmed { TEXT_DIMMED } else { TEXT_PRIMARY };
    let border_color = if dimmed { TEXT_DIMMED } else { PURPLE_ACCENT };

    let mut spans: Vec<Span> = Vec::new();

    // First element opening bracket
    spans.push(Span::styled("┘", Style::default().fg(border_color)));

    // Filter: show input field when active, otherwise show "filter" shortcut
    if app.filter_active {
        spans.push(Span::styled("f ", Style::default().fg(shortcut_color)));
        spans.push(Span::styled(app.filter_text.clone(), Style::default().fg(text_color)));
        spans.push(Span::styled("_", Style::default().fg(text_color)));
        spans.push(Span::styled(" ⏎", Style::default().fg(shortcut_color)));
    } else if !app.filter_text.is_empty() {
        spans.push(Span::styled("f", Style::default().fg(shortcut_color)));
        spans.push(Span::styled(format!("[{}] ", app.filter_text.clone()), Style::default().fg(value_color)));
        spans.push(Span::styled("c", Style::default().fg(shortcut_color)));
        spans.push(Span::styled("lear", Style::default().fg(text_color)));
    } else {
        spans.push(Span::styled("f", Style::default().fg(shortcut_color)));
        spans.push(Span::styled("ilter", Style::default().fg(text_color)));
    }

    // Separator
    spans.push(Span::styled("└┘", Style::default().fg(border_color)));

    // Platform
    spans.push(Span::styled("p", Style::default().fg(shortcut_color)));
    spans.push(Span::styled("latform", Style::default().fg(text_color)));

    // Separator
    spans.push(Span::styled("└┘", Style::default().fg(border_color)));

    // Sort order
    spans.push(Span::styled("s", Style::default().fg(shortcut_color)));
    spans.push(Span::styled("ort", Style::default().fg(text_color)));
    match app.sort_order.label() {
        Some(label) => spans.push(Span::styled(format!("[{}]", label), Style::default().fg(value_color))),
        None => spans.push(Span::styled("[—]", Style::default().fg(text_color))),
    }

    // Separator
    spans.push(Span::styled("└┘", Style::default().fg(border_color)));

    // Refresh
    spans.push(Span::styled("r", Style::default().fg(shortcut_color)));
    spans.push(Span::styled("efresh", Style::default().fg(text_color)));


    // Last element closing bracket
    spans.push(Span::styled("└", Style::default().fg(border_color)));

    Line::from(spans)
}

/// Build a title with btop-style brackets
fn build_title(text: &str, border_color: Color, title_color: Color) -> Line<'static> {
    Line::from(vec![
        Span::styled("┐", Style::default().fg(border_color)),
        Span::styled(text.to_string(), Style::default().fg(title_color)),
        Span::styled("┌", Style::default().fg(border_color)),
    ])
}

fn render_game_details(frame: &mut Frame, app: &App, area: Rect, dimmed: bool) {
    // When dimmed, ALL colors become TEXT_DIMMED
    let text_color = if dimmed { TEXT_DIMMED } else { TEXT_PRIMARY };
    let label_color = if dimmed { TEXT_DIMMED } else { PURPLE_LIGHT };
    let border_color = if dimmed { TEXT_DIMMED } else { PURPLE_ACCENT };
    let title_color = if dimmed { TEXT_DIMMED } else { TEXT_PRIMARY };
    let purple_color = if dimmed { TEXT_DIMMED } else { PURPLE_PRIMARY };
    let green_color = if dimmed { TEXT_DIMMED } else { ACCENT_GREEN };
    let yellow_color = if dimmed { TEXT_DIMMED } else { ACCENT_YELLOW };
    let secondary_color = if dimmed { TEXT_DIMMED } else { TEXT_SECONDARY };

    let title = build_title("Game Details", border_color, title_color);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(title);

    // Get selected deal for basic info
    let selected_deal = app.selected_deal();

    if selected_deal.is_none() {
        let padding = vertical_padding(area.height, 1);
        let empty = Paragraph::new(format!("{}Select a deal to view details", padding))
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
    let title_color = if dimmed { TEXT_DIMMED } else { TEXT_PRIMARY };

    let title = build_title("Price History", border_color, title_color);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(title);

    // Price history chart placeholder
    let content_lines = vec![
        "Price history chart coming soon...",
        "",
        "Visit IsThereAnyDeal.com for full",
        "price history information.",
    ];
    let padding = vertical_padding(area.height, content_lines.len() as u16);
    let content = format!("{}{}", padding, content_lines.join("\n"));

    let placeholder = Paragraph::new(content)
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
    let popup_width = 60u16;
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
        OptionsTab::Region => render_region_tab(frame, app, content_area),
        OptionsTab::Platforms => render_platforms_tab(frame, app, content_area),
        OptionsTab::Advanced => render_advanced_tab(frame, app, content_area),
    }
}

fn render_region_tab(frame: &mut Frame, app: &App, area: Rect) {
    // Split area: description + region list + help
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Description
            Constraint::Min(5),     // Region list
            Constraint::Length(2),  // Help text
        ])
        .split(area);

    // Description
    let desc = Paragraph::new(Line::from(Span::styled(
        "Select your region for local prices:",
        Style::default().fg(TEXT_SECONDARY),
    )));
    frame.render_widget(desc, chunks[0]);

    // Region list
    let mut region_lines: Vec<Line> = Vec::new();
    for (i, region) in Region::ALL.iter().enumerate() {
        let is_selected = app.options.region_list_index == i;
        let is_current = app.options.region == *region;

        let marker = if is_current { "●" } else { "○" };
        let line_style = if is_selected {
            Style::default().fg(TEXT_PRIMARY).bg(PURPLE_ACCENT)
        } else if is_current {
            Style::default().fg(PURPLE_LIGHT)
        } else {
            Style::default().fg(TEXT_PRIMARY)
        };

        region_lines.push(Line::from(Span::styled(
            format!(" {} {}", marker, region.name()),
            line_style,
        )));
    }

    let region_list = Paragraph::new(region_lines)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(PURPLE_ACCENT))
            .title(Span::styled(" Region ", Style::default().fg(PURPLE_LIGHT))));
    frame.render_widget(region_list, chunks[1]);

    // Help text
    let help = Paragraph::new(Line::from(Span::styled(
        "[Enter] Select  [Tab] Switch tab  [Esc] Close",
        Style::default().fg(TEXT_SECONDARY),
    )));
    frame.render_widget(help, chunks[2]);
}

fn render_platforms_tab(frame: &mut Frame, app: &App, area: Rect) {
    // Split area: default selector (2 lines) + platforms list (rest) + help (2 lines)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),  // Default platform selector
            Constraint::Min(5),     // Platforms list with border
            Constraint::Length(2),  // Help text
        ])
        .split(area);

    // Default platform selector (index 0)
    let is_default_selected = app.options.platform_list_index == 0;
    let default_style = if is_default_selected {
        Style::default().fg(TEXT_PRIMARY).bg(PURPLE_ACCENT)
    } else {
        Style::default().fg(TEXT_PRIMARY)
    };
    let default_line = Line::from(vec![
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
    ]);
    frame.render_widget(Paragraph::new(default_line), chunks[0]);

    // Platform list with checkboxes in a bordered block (skip Platform::All)
    let platforms_without_all: Vec<&Platform> = Platform::ALL
        .iter()
        .filter(|p| **p != Platform::All)
        .collect();

    let mut platform_lines: Vec<Line> = Vec::new();
    for (i, platform) in platforms_without_all.iter().enumerate() {
        let list_index = i + 1; // Index 0 is default selector
        let is_selected = app.options.platform_list_index == list_index;
        let is_enabled = app.options.enabled_platforms.contains(platform);

        let checkbox = if is_enabled { "[x]" } else { "[ ]" };

        let line_style = if is_selected {
            Style::default().fg(TEXT_PRIMARY).bg(PURPLE_ACCENT)
        } else if is_enabled {
            Style::default().fg(TEXT_PRIMARY)
        } else {
            Style::default().fg(TEXT_DIMMED)
        };

        platform_lines.push(Line::from(Span::styled(
            format!(" {} {}", checkbox, platform.name()),
            line_style,
        )));
    }

    // Calculate scroll offset to keep selection visible
    let visible_height = chunks[1].height.saturating_sub(2) as usize; // Account for borders
    let scroll_offset = if app.options.platform_list_index > 0 {
        let adjusted_index = app.options.platform_list_index - 1; // Adjust for default selector
        if adjusted_index >= visible_height {
            (adjusted_index - visible_height + 1) as u16
        } else {
            0
        }
    } else {
        0
    };

    let platforms_list = Paragraph::new(platform_lines)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(PURPLE_ACCENT))
            .title(Span::styled(" Enabled Platforms ", Style::default().fg(PURPLE_LIGHT))))
        .scroll((scroll_offset, 0));
    frame.render_widget(platforms_list, chunks[1]);

    // Help text
    let help = Paragraph::new(Line::from(Span::styled(
        "[Enter] Toggle  [Tab] Switch tab  [Esc] Close",
        Style::default().fg(TEXT_SECONDARY),
    )));
    frame.render_widget(help, chunks[2]);
}

fn render_advanced_tab(frame: &mut Frame, app: &App, area: Rect) {
    // Split area: description + settings list + help
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Description
            Constraint::Min(5),     // Settings list
            Constraint::Length(2),  // Help text
        ])
        .split(area);

    // Description
    let desc = Paragraph::new(Line::from(Span::styled(
        "Performance and pagination settings:",
        Style::default().fg(TEXT_SECONDARY),
    )));
    frame.render_widget(desc, chunks[0]);

    // Settings list
    let settings = [
        ("Page Size", format!("{}", app.options.deals_page_size), "Deals loaded per batch"),
        ("Info Delay", format!("{}ms", app.options.game_info_delay_ms), "Debounce for game info"),
    ];

    let mut setting_lines: Vec<Line> = Vec::new();
    for (i, (name, value, desc)) in settings.iter().enumerate() {
        let is_selected = app.options.advanced_list_index == i;

        let line_style = if is_selected {
            Style::default().fg(TEXT_PRIMARY).bg(PURPLE_ACCENT)
        } else {
            Style::default().fg(TEXT_PRIMARY)
        };

        let value_style = if is_selected {
            Style::default().fg(TEXT_PRIMARY).bg(PURPLE_ACCENT).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(PURPLE_LIGHT).add_modifier(Modifier::BOLD)
        };

        let desc_style = if is_selected {
            Style::default().fg(TEXT_PRIMARY).bg(PURPLE_ACCENT)
        } else {
            Style::default().fg(TEXT_SECONDARY)
        };

        setting_lines.push(Line::from(vec![
            Span::styled(format!(" {}: ", name), line_style),
            Span::styled(format!("{:<6}", value), value_style),
            Span::styled(format!(" ({})", desc), desc_style),
        ]));
    }

    let settings_list = Paragraph::new(setting_lines)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(PURPLE_ACCENT))
            .title(Span::styled(" Settings ", Style::default().fg(PURPLE_LIGHT))));
    frame.render_widget(settings_list, chunks[1]);

    // Help text
    let help = Paragraph::new(Line::from(Span::styled(
        "[Enter] Cycle value  [Tab] Switch tab  [Esc] Close",
        Style::default().fg(TEXT_SECONDARY),
    )));
    frame.render_widget(help, chunks[2]);
}

fn render_keybinds_popup(frame: &mut Frame) {
    let area = frame.area();
    let popup_width = 45u16;
    let popup_height = 15u16;
    let popup_x = area.width.saturating_sub(popup_width) / 2;
    let popup_y = area.height.saturating_sub(popup_height) / 2;
    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let content = vec![
        "",
        "  [Up/Down] or [j/k]  Navigate",
        "  [Enter]             Open deal / Select",
        "  [f]                 Filter by name",
        "  [c]                 Clear filter",
        "  [p]                 Change platform",
        "  [s]                 Cycle sort order",
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

fn render_platform_popup(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let enabled_platforms = app.enabled_platforms();

    // Calculate popup size based on content
    let popup_width = 35u16;
    let popup_height = (enabled_platforms.len() as u16 + 5).min(20); // +5 for title, borders, help
    let popup_x = area.width.saturating_sub(popup_width) / 2;
    let popup_y = area.height.saturating_sub(popup_height) / 2;
    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    // Main popup block
    let block = Block::default()
        .title(Span::styled(" Select Platform ", Style::default().fg(PURPLE_LIGHT)))
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

    // Split into list and help
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(inner);

    // Platform list
    let mut platform_lines: Vec<Line> = Vec::new();
    for (i, platform) in enabled_platforms.iter().enumerate() {
        let is_selected = app.platform_popup_index == i;
        let is_current = app.platform_filter == *platform;

        let marker = if is_current { "●" } else { "○" };
        let line_style = if is_selected {
            Style::default().fg(TEXT_PRIMARY).bg(PURPLE_ACCENT)
        } else if is_current {
            Style::default().fg(PURPLE_LIGHT)
        } else {
            Style::default().fg(TEXT_PRIMARY)
        };

        platform_lines.push(Line::from(Span::styled(
            format!(" {} {}", marker, platform.name()),
            line_style,
        )));
    }

    // Calculate scroll offset
    let visible_height = chunks[0].height as usize;
    let scroll_offset = if app.platform_popup_index >= visible_height {
        (app.platform_popup_index - visible_height + 1) as u16
    } else {
        0
    };

    let platform_list = Paragraph::new(platform_lines).scroll((scroll_offset, 0));
    frame.render_widget(platform_list, chunks[0]);

    // Help text
    let help = Paragraph::new(Line::from(Span::styled(
        "[Enter] Select  [Esc] Cancel",
        Style::default().fg(TEXT_SECONDARY),
    )));
    frame.render_widget(help, chunks[1]);
}

/// Calculate vertical padding to center text within an area
fn vertical_padding(area_height: u16, text_lines: u16) -> String {
    let inner_height = area_height.saturating_sub(2); // Account for borders
    let padding = inner_height.saturating_sub(text_lines) / 2;
    "\n".repeat(padding as usize)
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
