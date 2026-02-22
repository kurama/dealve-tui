use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use super::styles::*;
use crate::model::{MenuItem, Model, OptionsTab};
use dealve_core::models::{Platform, Region};

pub fn render_menu_overlay(frame: &mut Frame, model: &Model) {
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
        .map(|line| Line::from(Span::styled(*line, Style::default().fg(PURPLE_PRIMARY))))
        .collect();
    let logo = Paragraph::new(logo_lines).alignment(Alignment::Center);
    frame.render_widget(logo, logo_area);

    let menu_x = area.width.saturating_sub(menu_width) / 2;
    let menu_y = start_y + logo_height + 1;
    let menu_area = Rect::new(menu_x, menu_y, menu_width, menu_height);

    frame.render_widget(Clear, menu_area);

    let items: Vec<ListItem> = MenuItem::ALL
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if i == model.ui.menu_selected {
                Style::default()
                    .bg(BG_HIGHLIGHT)
                    .fg(PURPLE_LIGHT)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(TEXT_SECONDARY)
            };
            let prefix = if i == model.ui.menu_selected {
                "> "
            } else {
                "  "
            };
            ListItem::new(format!("{}{}", prefix, item.name())).style(style)
        })
        .collect();

    let menu = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(PURPLE_LIGHT)),
    );

    frame.render_widget(menu, menu_area);
}

pub fn render_options_popup(frame: &mut Frame, model: &Model) {
    let area = frame.area();
    let popup_width = 60u16;
    let popup_height = 26u16;
    let popup_x = area.width.saturating_sub(popup_width) / 2;
    let popup_y = area.height.saturating_sub(popup_height) / 2;
    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(Span::styled(" Options ", Style::default().fg(PURPLE_LIGHT)))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(PURPLE_ACCENT));
    frame.render_widget(block, popup_area);

    let inner = Rect::new(
        popup_area.x + 1,
        popup_area.y + 1,
        popup_area.width - 2,
        popup_area.height - 2,
    );

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(0)])
        .split(inner);

    // Render tabs bar
    let tabs: Vec<Span> = OptionsTab::ALL
        .iter()
        .enumerate()
        .map(|(i, tab)| {
            if i == model.options.current_tab {
                Span::styled(
                    format!(" {} ", tab.name()),
                    Style::default()
                        .fg(TEXT_PRIMARY)
                        .bg(PURPLE_ACCENT)
                        .add_modifier(Modifier::BOLD),
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

    let content_area = chunks[1];
    match OptionsTab::ALL[model.options.current_tab] {
        OptionsTab::Region => render_region_tab(frame, model, content_area),
        OptionsTab::Platforms => render_platforms_tab(frame, model, content_area),
        OptionsTab::Advanced => render_advanced_tab(frame, model, content_area),
    }
}

fn render_region_tab(frame: &mut Frame, model: &Model, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(2),
        ])
        .split(area);

    let desc = Paragraph::new(Line::from(Span::styled(
        "Select your region for local prices:",
        Style::default().fg(TEXT_SECONDARY),
    )));
    frame.render_widget(desc, chunks[0]);

    let mut region_lines: Vec<Line> = Vec::new();
    let mut current_continent = "";
    let mut selected_rendered_line: usize = 0;

    for (i, region) in Region::ALL.iter().enumerate() {
        // Insert continent header when group changes
        if region.continent() != current_continent {
            current_continent = region.continent();
            if !region_lines.is_empty() {
                region_lines.push(Line::from(""));
            }
            region_lines.push(Line::from(Span::styled(
                format!(" — {} —", current_continent),
                Style::default()
                    .fg(PURPLE_LIGHT)
                    .add_modifier(Modifier::BOLD),
            )));
        }

        if model.options.region_list_index == i {
            selected_rendered_line = region_lines.len();
        }

        let is_selected = model.options.region_list_index == i;
        let is_current = model.options.region == *region;

        let marker = if is_current { "●" } else { "○" };
        let line_style = if is_selected {
            Style::default().fg(TEXT_PRIMARY).bg(PURPLE_ACCENT)
        } else if is_current {
            Style::default().fg(PURPLE_LIGHT)
        } else {
            Style::default().fg(TEXT_PRIMARY)
        };

        region_lines.push(Line::from(Span::styled(
            format!(" {} {} ({})", marker, region.name(), region.code()),
            line_style,
        )));
    }

    // Calculate scroll offset to keep selected item visible
    let visible_height = chunks[1].height.saturating_sub(2) as usize;
    let scroll_offset = if selected_rendered_line >= visible_height {
        (selected_rendered_line - visible_height + 1) as u16
    } else {
        0
    };

    let region_list = Paragraph::new(region_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(PURPLE_ACCENT))
                .title(Span::styled(" Region ", Style::default().fg(PURPLE_LIGHT))),
        )
        .scroll((scroll_offset, 0));
    frame.render_widget(region_list, chunks[1]);

    let help = Paragraph::new(Line::from(Span::styled(
        "[Enter] Select  [Tab] Switch tab  [Esc] Close",
        Style::default().fg(TEXT_SECONDARY),
    )));
    frame.render_widget(help, chunks[2]);
}

fn render_platforms_tab(frame: &mut Frame, model: &Model, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(5),
            Constraint::Length(2),
        ])
        .split(area);

    // Default platform selector (index 0)
    let is_default_selected = model.options.platform_list_index == 0;
    let default_style = if is_default_selected {
        Style::default().fg(TEXT_PRIMARY).bg(PURPLE_ACCENT)
    } else {
        Style::default().fg(TEXT_PRIMARY)
    };
    let default_line = Line::from(vec![
        Span::styled("Default: ", Style::default().fg(PURPLE_LIGHT)),
        Span::styled(
            format!("{} ", model.options.default_platform.name()),
            default_style,
        ),
        if is_default_selected {
            Span::styled("[Enter to change]", Style::default().fg(TEXT_SECONDARY))
        } else {
            Span::raw("")
        },
    ]);
    frame.render_widget(Paragraph::new(default_line), chunks[0]);

    // Platform list with checkboxes (skip Platform::All)
    let platforms_without_all: Vec<&Platform> = Platform::ALL
        .iter()
        .filter(|p| **p != Platform::All)
        .collect();

    let mut platform_lines: Vec<Line> = Vec::new();
    for (i, platform) in platforms_without_all.iter().enumerate() {
        let list_index = i + 1;
        let is_selected = model.options.platform_list_index == list_index;
        let is_enabled = model.options.enabled_platforms.contains(platform);

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

    // Calculate scroll offset
    let visible_height = chunks[1].height.saturating_sub(2) as usize;
    let scroll_offset = if model.options.platform_list_index > 0 {
        let adjusted_index = model.options.platform_list_index - 1;
        if adjusted_index >= visible_height {
            (adjusted_index - visible_height + 1) as u16
        } else {
            0
        }
    } else {
        0
    };

    let platforms_list = Paragraph::new(platform_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(PURPLE_ACCENT))
                .title(Span::styled(
                    " Enabled Platforms ",
                    Style::default().fg(PURPLE_LIGHT),
                )),
        )
        .scroll((scroll_offset, 0));
    frame.render_widget(platforms_list, chunks[1]);

    let help = Paragraph::new(Line::from(Span::styled(
        "[Enter] Toggle  [Tab] Switch tab  [Esc] Close",
        Style::default().fg(TEXT_SECONDARY),
    )));
    frame.render_widget(help, chunks[2]);
}

fn render_advanced_tab(frame: &mut Frame, model: &Model, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(2),
        ])
        .split(area);

    let desc = Paragraph::new(Line::from(Span::styled(
        "Default sort and performance settings:",
        Style::default().fg(TEXT_SECONDARY),
    )));
    frame.render_widget(desc, chunks[0]);

    let sort_value = format!(
        "{} {}",
        model.options.default_sort.criteria.name(),
        model.options.default_sort.direction.arrow()
    );
    let settings = [
        ("Default Sort", sort_value, "Sort on startup"),
        (
            "Page Size",
            format!("{}", model.options.deals_page_size),
            "Deals per batch",
        ),
        (
            "Info Delay",
            format!("{}ms", model.options.game_info_delay_ms),
            "Debounce delay",
        ),
    ];

    let mut setting_lines: Vec<Line> = Vec::new();
    for (i, (name, value, desc)) in settings.iter().enumerate() {
        let is_selected = model.options.advanced_list_index == i;

        let line_style = if is_selected {
            Style::default().fg(TEXT_PRIMARY).bg(BG_HIGHLIGHT)
        } else {
            Style::default().fg(TEXT_PRIMARY)
        };

        let value_style = if is_selected {
            Style::default()
                .fg(PURPLE_LIGHT)
                .bg(BG_HIGHLIGHT)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(PURPLE_LIGHT)
                .add_modifier(Modifier::BOLD)
        };

        let desc_style = if is_selected {
            Style::default().fg(TEXT_SECONDARY).bg(BG_HIGHLIGHT)
        } else {
            Style::default().fg(TEXT_SECONDARY)
        };

        setting_lines.push(Line::from(vec![
            Span::styled(format!(" {}: ", name), line_style),
            Span::styled(format!("{:<12}", value), value_style),
            Span::styled(format!(" ({})", desc), desc_style),
        ]));
    }

    let settings_list = Paragraph::new(setting_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(PURPLE_ACCENT))
            .title(Span::styled(
                " Settings ",
                Style::default().fg(PURPLE_LIGHT),
            )),
    );
    frame.render_widget(settings_list, chunks[1]);

    let help_lines = vec![
        Line::from(Span::styled(
            "[Enter] Cycle  [s] Direction  [Tab] Switch tab",
            Style::default().fg(TEXT_SECONDARY),
        )),
        Line::from(Span::styled(
            "[Esc] Close",
            Style::default().fg(TEXT_SECONDARY),
        )),
    ];
    let help = Paragraph::new(help_lines);
    frame.render_widget(help, chunks[2]);
}

pub fn render_keybinds_popup(frame: &mut Frame) {
    let area = frame.area();
    let popup_width = 45u16;
    let popup_height = 17u16;
    let popup_x = area.width.saturating_sub(popup_width) / 2;
    let popup_y = area.height.saturating_sub(popup_height) / 2;
    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let content = vec![
        "",
        "  [Up/Down] or [j/k]  Navigate",
        "  [PgUp/PgDown]       Page scroll",
        "  [Home/End]          First/Last deal",
        "  [Enter]             Open deal / Select",
        "  [f]                 Filter by name",
        "  [c]                 Clear filter",
        "  [$]                 Price filter",
        "  [p]                 Change platform",
        "  [s]                 Toggle sort direction",
        "  [Left/Right]        Change sort criteria",
        "  [r]                 Refresh deals",
        "  [Esc]               Menu / Close popup",
        "  [q]                 Quit (from menu)",
        "",
        "  [Esc] Close",
    ];

    let popup = Paragraph::new(content.join("\n"))
        .style(Style::default().fg(TEXT_PRIMARY))
        .block(
            Block::default()
                .title(Span::styled(
                    " Keybinds ",
                    Style::default().fg(PURPLE_LIGHT),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(PURPLE_PRIMARY)),
        );

    frame.render_widget(popup, popup_area);
}

pub fn render_platform_popup(frame: &mut Frame, model: &Model) {
    let area = frame.area();
    let enabled_platforms = model.enabled_platforms();

    let popup_width = 35u16;
    let popup_height = (enabled_platforms.len() as u16 + 5).min(20);
    let popup_x = area.width.saturating_sub(popup_width) / 2;
    let popup_y = area.height.saturating_sub(popup_height) / 2;
    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(Span::styled(
            " Select Platform ",
            Style::default().fg(PURPLE_LIGHT),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(PURPLE_ACCENT));
    frame.render_widget(block, popup_area);

    let inner = Rect::new(
        popup_area.x + 1,
        popup_area.y + 1,
        popup_area.width - 2,
        popup_area.height - 2,
    );

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(inner);

    let mut platform_lines: Vec<Line> = Vec::new();
    for (i, platform) in enabled_platforms.iter().enumerate() {
        let is_selected = model.ui.platform_popup_index == i;
        let is_current = model.platform_filter == *platform;

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
    let scroll_offset = if model.ui.platform_popup_index >= visible_height {
        (model.ui.platform_popup_index - visible_height + 1) as u16
    } else {
        0
    };

    let platform_list = Paragraph::new(platform_lines).scroll((scroll_offset, 0));
    frame.render_widget(platform_list, chunks[0]);

    let help = Paragraph::new(Line::from(Span::styled(
        "[Enter] Select  [Esc] Cancel",
        Style::default().fg(TEXT_SECONDARY),
    )));
    frame.render_widget(help, chunks[1]);
}

pub fn render_price_filter_popup(frame: &mut Frame, model: &Model) {
    let area = frame.area();
    let popup_width = 32u16;
    let popup_height = 10u16;
    let popup_x = area.width.saturating_sub(popup_width) / 2;
    let popup_y = area.height.saturating_sub(popup_height) / 2;
    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(Span::styled(
            " Price Filter ",
            Style::default().fg(PURPLE_LIGHT),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(PURPLE_ACCENT));
    frame.render_widget(block, popup_area);

    let inner = Rect::new(
        popup_area.x + 2,
        popup_area.y + 2,
        popup_area.width - 4,
        popup_area.height - 4,
    );

    let min_selected = model.price_filter.selected_field == 0;
    let max_selected = model.price_filter.selected_field == 1;

    let min_style = if min_selected {
        Style::default().fg(TEXT_PRIMARY).bg(PURPLE_ACCENT)
    } else {
        Style::default().fg(TEXT_PRIMARY)
    };

    let max_style = if max_selected {
        Style::default().fg(TEXT_PRIMARY).bg(PURPLE_ACCENT)
    } else {
        Style::default().fg(TEXT_PRIMARY)
    };

    let min_cursor = if min_selected { "▋" } else { "" };
    let max_cursor = if max_selected { "▋" } else { "" };

    let min_display = format!("{}{}", model.price_filter.min_input, min_cursor);
    let max_display = format!("{}{}", model.price_filter.max_input, max_cursor);

    let content = vec![
        Line::from(vec![
            Span::styled("Min: ", Style::default().fg(PURPLE_LIGHT)),
            Span::styled(format!("{:<10}", min_display), min_style),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Max: ", Style::default().fg(PURPLE_LIGHT)),
            Span::styled(format!("{:<10}", max_display), max_style),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "[Tab] Switch  [Enter] Apply",
            Style::default().fg(TEXT_SECONDARY),
        )),
        Line::from(Span::styled(
            "[c] Clear  [Esc] Cancel",
            Style::default().fg(TEXT_SECONDARY),
        )),
    ];

    let paragraph = Paragraph::new(content);
    frame.render_widget(paragraph, inner);
}
