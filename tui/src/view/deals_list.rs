use ratatui::{
    layout::{Alignment, Constraint, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Cell, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState,
        Table,
    },
    Frame,
};

use super::styles::*;
use crate::model::Model;

pub fn render_deals_list(frame: &mut Frame, model: &mut Model, area: Rect, dimmed: bool) {
    let text_color = if dimmed { TEXT_DIMMED } else { TEXT_PRIMARY };
    let border_color = if dimmed { TEXT_DIMMED } else { PURPLE_ACCENT };
    let title_color = if dimmed { TEXT_DIMMED } else { TEXT_PRIMARY };

    let title_text = format!("Deals [{}]", model.platform_filter.name());
    let title = build_title(&title_text, border_color, title_color);

    let status_line = build_status_line(model, dimmed);

    if model.loading.deals {
        let spinner = model.spinner_char();
        let padding = vertical_padding(area.height, 1);
        let loading = Paragraph::new(format!("{}{} Loading deals...", padding, spinner))
            .alignment(Alignment::Center)
            .style(Style::default().fg(text_color))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color))
                    .title(title)
                    .title_bottom(status_line),
            );
        frame.render_widget(loading, area);
        return;
    }

    if let Some(error) = &model.error {
        let error_title = build_title("Error", border_color, title_color);
        let error_msg = Paragraph::new(format!("Error: {}", error))
            .style(Style::default().fg(ratatui::style::Color::Red))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color))
                    .title(error_title)
                    .title_bottom(status_line),
            );
        frame.render_widget(error_msg, area);
        return;
    }

    let filtered_deals = model.filtered_deals();

    if filtered_deals.is_empty() {
        let padding = vertical_padding(area.height, 1);
        let empty = Paragraph::new(format!("{}No deals found", padding))
            .alignment(Alignment::Center)
            .style(Style::default().fg(text_color))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color))
                    .title(title)
                    .title_bottom(status_line),
            );
        frame.render_widget(empty, area);
        return;
    }

    // Build table header
    let header_color = if dimmed { TEXT_DIMMED } else { TEXT_PRIMARY };
    let header = Row::new(vec![
        Cell::from("Title").style(Style::default().fg(header_color)),
        Cell::from("Price").style(Style::default().fg(header_color)),
        Cell::from("Deal").style(Style::default().fg(header_color)),
        Cell::from("").style(Style::default().fg(header_color)),
    ]);

    // Build table rows
    let rows: Vec<Row> = filtered_deals
        .iter()
        .map(|deal| {
            let price_str = format!("{}{:.2}", deal.price.currency_symbol(), deal.price.amount);
            let discount_str = format!("-{}%", deal.price.discount);

            let is_atl = deal
                .history_low
                .map(|low| (low - deal.price.amount).abs() < 0.01)
                .unwrap_or(false);

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
        Style::default().bg(BG_HIGHLIGHT)
    };

    let total_items = filtered_deals.len();
    let selected = model.ui.table_state.selected().unwrap_or(0);

    // Counter for bottom right corner
    // Use spinner in place of "+" to avoid width changes during loading
    let counter_color = if dimmed { TEXT_DIMMED } else { TEXT_PRIMARY };
    let suffix = if model.pagination.loading_more {
        format!("{}", model.spinner_char())
    } else if model.pagination.has_more {
        "+".to_string()
    } else {
        " ".to_string()
    };
    let counter = Span::styled(
        format!(" {}/{} {} ", selected + 1, total_items, suffix),
        Style::default()
            .fg(counter_color)
            .add_modifier(Modifier::BOLD),
    );

    let widths = [
        Constraint::Min(20),
        Constraint::Length(10),
        Constraint::Length(7),
        Constraint::Length(4),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .title(title)
                .title_bottom(status_line)
                .title_bottom(Line::from(counter).alignment(Alignment::Right)),
        )
        .row_highlight_style(highlight_style)
        .highlight_symbol("> ");

    frame.render_stateful_widget(table, area, &mut model.ui.table_state);

    // Render scrollbar
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

    let scrollbar_area = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: area.height.saturating_sub(1),
    };
    frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
}

/// Build status bar line with btop-style highlighted shortcut keys and separators
fn build_status_line(model: &Model, dimmed: bool) -> Line<'static> {
    let text_color = if dimmed { TEXT_DIMMED } else { TEXT_PRIMARY };
    let shortcut_color = if dimmed { TEXT_DIMMED } else { SHORTCUT_KEY };
    let value_color = if dimmed { TEXT_DIMMED } else { TEXT_PRIMARY };
    let border_color = if dimmed { TEXT_DIMMED } else { PURPLE_ACCENT };

    let mut spans: Vec<Span> = Vec::new();

    spans.push(Span::styled("┘", Style::default().fg(border_color)));

    // Filter
    if model.filter.active {
        spans.push(Span::styled("f ", Style::default().fg(shortcut_color)));
        spans.push(Span::styled(
            model.filter.text.clone(),
            Style::default().fg(text_color),
        ));
        spans.push(Span::styled("_", Style::default().fg(text_color)));
        spans.push(Span::styled(" ⏎", Style::default().fg(shortcut_color)));
    } else if !model.filter.text.is_empty() {
        spans.push(Span::styled("f", Style::default().fg(shortcut_color)));
        spans.push(Span::styled(
            format!("[{}] ", model.filter.text.clone()),
            Style::default().fg(value_color),
        ));
        spans.push(Span::styled("c", Style::default().fg(shortcut_color)));
        spans.push(Span::styled("lear", Style::default().fg(text_color)));
    } else {
        spans.push(Span::styled("f", Style::default().fg(shortcut_color)));
        spans.push(Span::styled("ilter", Style::default().fg(text_color)));
    }

    spans.push(Span::styled("└┘", Style::default().fg(border_color)));

    // Platform
    spans.push(Span::styled("p", Style::default().fg(shortcut_color)));
    spans.push(Span::styled("latform", Style::default().fg(text_color)));

    spans.push(Span::styled("└┘", Style::default().fg(border_color)));

    // Price filter
    spans.push(Span::styled("$", Style::default().fg(shortcut_color)));
    if model.price_filter.is_active() {
        spans.push(Span::styled(
            format!("[{}]", model.price_filter.label()),
            Style::default().fg(value_color),
        ));
    }

    spans.push(Span::styled("└┘", Style::default().fg(border_color)));

    // Sort
    spans.push(Span::styled("s", Style::default().fg(shortcut_color)));
    spans.push(Span::styled("ort[", Style::default().fg(text_color)));
    spans.push(Span::styled("←", Style::default().fg(shortcut_color)));
    spans.push(Span::styled(
        model.sort_state.criteria.name().to_string(),
        Style::default().fg(value_color),
    ));
    spans.push(Span::styled(
        model.sort_state.direction.arrow().to_string(),
        Style::default().fg(value_color),
    ));
    spans.push(Span::styled("→", Style::default().fg(shortcut_color)));
    spans.push(Span::styled("]", Style::default().fg(text_color)));

    spans.push(Span::styled("└┘", Style::default().fg(border_color)));

    // Search mode status
    if model.is_search_mode() {
        spans.push(Span::styled("A", Style::default().fg(shortcut_color)));
        spans.push(Span::styled("PI-search ", Style::default().fg(text_color)));
        spans.push(Span::styled("└┘", Style::default().fg(border_color)));
    }

    // Refresh
    spans.push(Span::styled("r", Style::default().fg(shortcut_color)));
    spans.push(Span::styled("efresh", Style::default().fg(text_color)));

    spans.push(Span::styled("└", Style::default().fg(border_color)));

    Line::from(spans)
}
