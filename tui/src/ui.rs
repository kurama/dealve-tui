use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(frame.area());

    let title = Paragraph::new("Dealve - Game Deals Browser")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);

    let items: Vec<ListItem> = app
        .deals
        .iter()
        .enumerate()
        .map(|(i, deal)| {
            let style = if i == app.selected_index {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default()
            };

            let content = Line::from(vec![
                Span::raw(&deal.title),
                Span::raw(" - "),
                Span::styled(
                    format!("{}% off", deal.price.discount),
                    Style::default().fg(Color::Green),
                ),
                Span::raw(" @ "),
                Span::styled(&deal.shop.name, Style::default().fg(Color::Yellow)),
            ]);

            ListItem::new(content).style(style)
        })
        .collect();

    let deals_list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Deals"));
    frame.render_widget(deals_list, chunks[1]);

    let help = Paragraph::new("↑/↓: Navigate | q: Quit | r: Refresh")
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(help, chunks[2]);
}
