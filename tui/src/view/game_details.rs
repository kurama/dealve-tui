use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::styles::*;
use crate::model::Model;

pub fn render_game_details(frame: &mut Frame, model: &Model, area: Rect, dimmed: bool) {
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

    let selected_deal = model.selected_deal();

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
    let game_info = model.selected_game_info();
    let is_loading = model.loading.game_info.as_ref() == Some(&deal.id);
    let mut lines: Vec<Line> = Vec::new();

    let is_atl = deal
        .history_low
        .map(|low| (low - deal.price.amount).abs() < 0.01)
        .unwrap_or(false);

    // ATL badge
    if is_atl {
        lines.push(Line::from(vec![Span::styled(
            ">> ALL-TIME LOW <<",
            Style::default()
                .fg(purple_color)
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(""));
    }

    // Title
    lines.push(Line::from(vec![Span::styled(
        &deal.title,
        Style::default().fg(text_color).add_modifier(Modifier::BOLD),
    )]));

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
                Span::styled(
                    info.developers.join(", "),
                    Style::default().fg(secondary_color),
                ),
            ]));
        }
        if !info.publishers.is_empty() && info.publishers != info.developers {
            lines.push(Line::from(vec![
                Span::styled("Publisher: ", Style::default().fg(label_color)),
                Span::styled(
                    info.publishers.join(", "),
                    Style::default().fg(secondary_color),
                ),
            ]));
        }
    } else if is_loading {
        lines.push(Line::from(vec![Span::styled(
            "Loading game info...",
            Style::default().fg(secondary_color),
        )]));
    }

    lines.push(Line::from(""));

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
        Span::styled(
            regular_str,
            Style::default()
                .fg(secondary_color)
                .add_modifier(Modifier::CROSSED_OUT),
        ),
        Span::styled(" -> ", Style::default().fg(secondary_color)),
        Span::styled(
            price_str,
            Style::default()
                .fg(price_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" ({})", discount_str),
            Style::default().fg(yellow_color),
        ),
    ]));

    // Savings
    let savings = deal.regular_price - deal.price.amount;
    if savings > 0.0 {
        let savings_str = format!("{}{:.2}", deal.price.currency_symbol(), savings);
        lines.push(Line::from(vec![
            Span::styled("You save ", Style::default().fg(secondary_color)),
            Span::styled(
                savings_str,
                Style::default()
                    .fg(green_color)
                    .add_modifier(Modifier::BOLD),
            ),
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
            lines.push(Line::from(""));
            let tags_str = info
                .tags
                .iter()
                .take(5)
                .cloned()
                .collect::<Vec<_>>()
                .join(" | ");
            lines.push(Line::from(vec![Span::styled(
                tags_str,
                Style::default().fg(secondary_color),
            )]));
        }
    }

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}
