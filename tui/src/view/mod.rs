pub mod deals_list;
pub mod game_details;
pub mod popups;
pub mod price_chart;
pub mod styles;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::Style,
    widgets::Block,
    Frame,
};

use crate::model::{Model, Popup};
use styles::BG_DARK;

pub fn view(frame: &mut Frame, model: &mut Model) {
    // Fill entire screen with dark purple background
    let bg_block = Block::default().style(Style::default().bg(BG_DARK));
    frame.render_widget(bg_block, frame.area());

    let dimmed = model.ui.show_menu;
    render_main(frame, model, dimmed);

    if model.ui.show_menu {
        popups::render_menu_overlay(frame, model);
    }

    match model.ui.popup {
        Popup::None => {}
        Popup::Options => popups::render_options_popup(frame, model),
        Popup::Keybinds => popups::render_keybinds_popup(frame),
        Popup::Platform => popups::render_platform_popup(frame, model),
        Popup::PriceFilter => popups::render_price_filter_popup(frame, model),
    }
}

fn render_main(frame: &mut Frame, model: &mut Model, dimmed: bool) {
    let area = frame.area();

    // Split horizontal: 55% left (deals), 45% right (details + chart)
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);

    let left_panel = main_chunks[0];

    // Right panel: split vertical - details (40%), chart (60%)
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(main_chunks[1]);

    let details_panel = right_chunks[0];
    let chart_panel = right_chunks[1];

    deals_list::render_deals_list(frame, model, left_panel, dimmed);
    game_details::render_game_details(frame, model, details_panel, dimmed);
    price_chart::render_price_chart(frame, model, chart_panel, dimmed);
}
