use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Sparkline},
    Frame,
};

use super::styles::*;
use crate::model::Model;

pub fn render_price_chart(frame: &mut Frame, model: &Model, area: Rect, dimmed: bool) {
    let text_color = if dimmed { TEXT_DIMMED } else { TEXT_SECONDARY };
    let border_color = if dimmed { TEXT_DIMMED } else { PURPLE_ACCENT };
    let title_color = if dimmed { TEXT_DIMMED } else { TEXT_PRIMARY };
    let chart_color = if dimmed { TEXT_DIMMED } else { ACCENT_GREEN };

    let title = build_title("Price History (1 year)", border_color, title_color);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(title);

    let history = model.selected_price_history();

    if let Some(points) = history {
        if points.is_empty() {
            render_empty(frame, area, block, text_color, "No price history available");
            return;
        }

        // Convert prices to u64 (cents) for Sparkline
        let data: Vec<u64> = points.iter().map(|p| (p.price * 100.0) as u64).collect();

        let min_price = points.iter().map(|p| p.price).fold(f64::INFINITY, f64::min);
        let max_price = points
            .iter()
            .map(|p| p.price)
            .fold(f64::NEG_INFINITY, f64::max);
        let current_price = points.last().map(|p| p.price).unwrap_or(0.0);

        let currency = model
            .selected_deal()
            .map(|d| d.price.currency_symbol())
            .unwrap_or("€");

        // Render block and get inner area
        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Layout: info line + sparkline
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(1)])
            .split(inner);

        // Summary line
        let summary = Line::from(vec![
            Span::styled(
                format!("Low: {}{:.2}", currency, min_price),
                Style::default().fg(ACCENT_GREEN),
            ),
            Span::styled("  ", Style::default()),
            Span::styled(
                format!("High: {}{:.2}", currency, max_price),
                Style::default().fg(ACCENT_YELLOW),
            ),
            Span::styled("  ", Style::default()),
            Span::styled(
                format!("Now: {}{:.2}", currency, current_price),
                Style::default().fg(TEXT_PRIMARY),
            ),
        ]);
        frame.render_widget(Paragraph::new(summary), chunks[0]);

        // Sparkline
        let sparkline = Sparkline::default()
            .data(&data)
            .style(Style::default().fg(chart_color));
        frame.render_widget(sparkline, chunks[1]);
    } else if model.loading.price_history.is_some() {
        let spinner = model.spinner_char();
        render_empty(
            frame,
            area,
            block,
            text_color,
            &format!("{} Loading price history...", spinner),
        );
    } else {
        render_empty(
            frame,
            area,
            block,
            text_color,
            "Select a deal to view price history",
        );
    }
}

fn render_empty(
    frame: &mut Frame,
    area: Rect,
    block: Block,
    text_color: ratatui::style::Color,
    message: &str,
) {
    let padding = vertical_padding(area.height, 1);
    let content = format!("{}{}", padding, message);

    let placeholder = Paragraph::new(content)
        .alignment(Alignment::Center)
        .style(Style::default().fg(text_color))
        .block(block);

    frame.render_widget(placeholder, area);
}
