use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &mut App) {
    let area = centered_rect(85, 90, frame.area());

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
        .title("─ Dealve TUI ─");

    let filter_text = format!("🔥 Top Deals  [Filter: {}]", app.platform_filter.name());
    let title = Paragraph::new(filter_text)
        .block(title_block);
    frame.render_widget(title, chunks[0]);

    if app.loading {
        let loading = Paragraph::new("Loading deals...")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(loading, chunks[1]);
    } else if let Some(error) = &app.error {
        let error_msg = Paragraph::new(format!("Error: {}", error))
            .block(Block::default().borders(Borders::ALL).title("Error"));
        frame.render_widget(error_msg, chunks[1]);
    } else {
        let filtered_deals = app.filtered_deals();

        if filtered_deals.is_empty() {
            let empty = Paragraph::new("No deals found for this platform")
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL));
            frame.render_widget(empty, chunks[1]);
        } else {
            let items: Vec<ListItem> = filtered_deals
                .iter()
                .map(|deal| {

                let discount_bar = create_discount_bar(deal.price.discount);

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
                    Span::raw(format!("{:<50}", truncate(&deal.title, 50))),
                    Span::raw(format!("{:>8}", price_str)),
                    Span::raw("  "),
                    Span::raw(format!("{:>4}", discount_str)),
                    Span::raw("  "),
                    Span::raw(discount_bar),
                    Span::raw(" "),
                    Span::raw(format!("{:<13}", low_info)),
                ]);

                ListItem::new(content)
                })
                .collect();

            let deals_list = List::new(items)
                .block(Block::default().borders(Borders::ALL))
                .highlight_style(Style::default().bg(Color::DarkGray))
                .highlight_symbol("> ");

            frame.render_stateful_widget(deals_list, chunks[1], &mut app.list_state);
        }
    }

    let help = Paragraph::new("[↑/↓] Navigate  [f] Filter  [Enter] Open  [r] Refresh  [q] Quit")
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
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

fn create_discount_bar(discount: u8) -> String {
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
