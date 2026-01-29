use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &App) {
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

    let title = Paragraph::new("🔥 Top Deals")
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
    } else if app.deals.is_empty() {
        let empty = Paragraph::new("No deals found")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(empty, chunks[1]);
    } else {
        let items: Vec<ListItem> = app
            .deals
            .iter()
            .enumerate()
            .map(|(i, deal)| {
                let is_selected = i == app.selected_index;
                let symbol = if is_selected { ">" } else { " " };

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
                    Span::raw(format!("{} ", symbol)),
                    Span::raw(format!("{:<50}", truncate(&deal.title, 50))),
                    Span::raw(format!("{:>8}", price_str)),
                    Span::raw("  "),
                    Span::raw(format!("{:>4}", discount_str)),
                    Span::raw("  "),
                    Span::raw(discount_bar),
                    Span::raw(" "),
                    Span::raw(format!("{:<13}", low_info)),
                ]);

                let mut item = ListItem::new(content);
                if is_selected {
                    item = item.style(Style::default().bg(Color::DarkGray));
                }
                item
            })
            .collect();

        let deals_list = List::new(items)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(deals_list, chunks[1]);
    }

    let help = Paragraph::new("[↑/↓] Navigate  [Enter] Open  [r] Refresh  [q] Quit")
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(help, chunks[2]);
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
