use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use crate::app::{App, MenuItem, Popup};

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
    render_deals(frame, app, dimmed);

    if app.show_menu {
        render_menu_overlay(frame, app);
    }

    match app.popup {
        Popup::None => {}
        Popup::Options => render_options_popup(frame),
        Popup::Keybinds => render_keybinds_popup(frame),
    }
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

    let logo_lines: Vec<Line> = ASCII_LOGO
        .iter()
        .map(|line| Line::from(*line))
        .collect();
    let logo = Paragraph::new(logo_lines)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::White));
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
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default().fg(Color::Gray)
            };
            let prefix = if i == app.menu_selected { "> " } else { "  " };
            ListItem::new(format!("{}{}", prefix, item.name())).style(style)
        })
        .collect();

    let menu = List::new(items)
        .block(Block::default().borders(Borders::ALL).style(Style::default()));

    frame.render_widget(menu, menu_area);
}

fn render_options_popup(frame: &mut Frame) {
    let area = frame.area();
    let popup_width = 40u16;
    let popup_height = 10u16;
    let popup_x = area.width.saturating_sub(popup_width) / 2;
    let popup_y = area.height.saturating_sub(popup_height) / 2;
    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let content = vec![
        "",
        "  Coming soon...",
        "",
        "  [Esc] Close",
    ];

    let popup = Paragraph::new(content.join("\n"))
        .block(Block::default()
            .title(" Options ")
            .borders(Borders::ALL)
            .style(Style::default()));

    frame.render_widget(popup, popup_area);
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
        "  [↑/↓] or [j/k]  Navigate",
        "  [Enter]         Open deal / Select",
        "  [f]             Open platform filter",
        "  [r]             Refresh deals",
        "  [Esc]           Menu / Close popup",
        "  [q]             Quit (from menu)",
        "",
        "  [Esc] Close",
    ];

    let popup = Paragraph::new(content.join("\n"))
        .block(Block::default()
            .title(" Keybinds ")
            .borders(Borders::ALL)
            .style(Style::default()));

    frame.render_widget(popup, popup_area);
}

fn render_deals(frame: &mut Frame, app: &mut App, dimmed: bool) {
    let area = centered_rect(85, 90, frame.area());
    let text_color = if dimmed { Color::DarkGray } else { Color::White };
    let bar_color = if dimmed { Color::Rgb(60, 60, 60) } else { Color::White };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

    let title_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(text_color))
        .title("─ Dealve TUI ─");

    let filter_text = format!("🔥 Top Deals  [Filter: {}]", app.platform_filter.name());
    let title = Paragraph::new(filter_text)
        .style(Style::default().fg(text_color))
        .block(title_block);
    frame.render_widget(title, chunks[0]);

    if app.loading {
        let loading = Paragraph::new("Loading deals...")
            .alignment(Alignment::Center)
            .style(Style::default().fg(text_color))
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(text_color)));
        frame.render_widget(loading, chunks[1]);
    } else if let Some(error) = &app.error {
        let error_msg = Paragraph::new(format!("Error: {}", error))
            .style(Style::default().fg(text_color))
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(text_color)).title("Error"));
        frame.render_widget(error_msg, chunks[1]);
    } else {
        let filtered_deals = app.filtered_deals();

        if filtered_deals.is_empty() {
            let empty = Paragraph::new("No deals found for this platform")
                .alignment(Alignment::Center)
                .style(Style::default().fg(text_color))
                .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(text_color)));
            frame.render_widget(empty, chunks[1]);
        } else {
            let items: Vec<ListItem> = filtered_deals
                .iter()
                .map(|deal| {

                let discount_bar = create_discount_bar(deal.price.discount, bar_color);

                let low_info = if let Some(low) = deal.history_low {
                    if (low - deal.price.amount).abs() < 0.01 {
                        "ATL! 🏆".to_string()
                    } else {
                        format!("Low: {}{:.2}", deal.price.currency_symbol(), low)
                    }
                } else {
                    String::new()
                };

                let price_str = format!("{}{:.2}", deal.price.currency_symbol(), deal.price.amount);
                let discount_str = format!("-{}%", deal.price.discount);

                let content = Line::from(vec![
                    Span::styled(format!("{:<50}", truncate(&deal.title, 50)), Style::default().fg(text_color)),
                    Span::styled(format!("{:>8}", price_str), Style::default().fg(text_color)),
                    Span::styled("  ", Style::default().fg(text_color)),
                    Span::styled(format!("{:>4}", discount_str), Style::default().fg(text_color)),
                    Span::styled("  ", Style::default().fg(text_color)),
                    Span::styled(discount_bar, Style::default().fg(bar_color)),
                    Span::styled(" ", Style::default().fg(text_color)),
                    Span::styled(format!("{:<13}", low_info), Style::default().fg(text_color)),
                ]);

                ListItem::new(content)
                })
                .collect();

            let deals_list = List::new(items)
                .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(text_color)))
                .highlight_style(Style::default().bg(Color::DarkGray))
                .highlight_symbol("> ");

            frame.render_stateful_widget(deals_list, chunks[1], &mut app.list_state);
        }
    }

    let help = Paragraph::new("[↑/↓] Navigate  [f] Filter  [Enter] Open  [r] Refresh  [Esc] Menu")
        .alignment(Alignment::Center)
        .style(Style::default().fg(text_color))
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(text_color)));
    frame.render_widget(help, chunks[2]);

    if app.show_platform_dropdown {
        render_dropdown(frame, app, chunks[0]);
    }
}

fn render_dropdown(frame: &mut Frame, app: &mut App, title_area: Rect) {
    let dropdown_width = 20u16;
    let max_visible = 15u16;
    let dropdown_height = max_visible + 2;

    let dropdown_x = title_area.x + 20;
    let dropdown_y = title_area.y + title_area.height;

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

    let items: Vec<ListItem> = App::platforms()
        .iter()
        .map(|platform| {
            ListItem::new(format!("  {}", platform.name()))
        })
        .collect();

    let mut list_state = ratatui::widgets::ListState::default();
    list_state.select(Some(app.dropdown_selected));

    let dropdown = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Platform"))
        .highlight_style(Style::default().bg(Color::DarkGray))
        .highlight_symbol("> ");

    frame.render_stateful_widget(dropdown, dropdown_area, &mut list_state);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn create_discount_bar(discount: u8, _color: Color) -> String {
    let total = 8;
    let savings = (((discount as f64 / 100.0) * total as f64).round() as usize).min(total);
    let paying = total - savings;
    format!("{}{}", "█".repeat(paying), "░".repeat(savings))
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
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
