use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use super::styles::*;
use crate::model::{GroupEditorField, MenuItem, Model, OptionsTab};
use dealve_core::models::{PlatformFilter, Region};

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
        .map(|line| Line::from(Span::styled(*line, Style::default().fg(primary()))))
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
                    .bg(bg_highlight())
                    .fg(primary_light())
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(text_secondary())
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
            .border_style(Style::default().fg(primary_light())),
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
        .title(Span::styled(
            " Options ",
            Style::default().fg(primary_light()),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(accent()));
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
                        .fg(text_primary())
                        .bg(accent())
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(
                    format!(" {} ", tab.name()),
                    Style::default().fg(text_secondary()),
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
        OptionsTab::Theme => render_theme_tab(frame, model, content_area),
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
        Style::default().fg(text_secondary()),
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
                    .fg(primary_light())
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
            Style::default().fg(text_primary()).bg(accent())
        } else if is_current {
            Style::default().fg(primary_light())
        } else {
            Style::default().fg(text_primary())
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
                .border_style(Style::default().fg(accent()))
                .title(Span::styled(
                    " Region ",
                    Style::default().fg(primary_light()),
                )),
        )
        .scroll((scroll_offset, 0));
    frame.render_widget(region_list, chunks[1]);

    let help = Paragraph::new(Line::from(Span::styled(
        "[Enter] Select  [Tab] Switch tab  [Esc] Close",
        Style::default().fg(text_secondary()),
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
        Style::default().fg(text_primary()).bg(accent())
    } else {
        Style::default().fg(text_primary())
    };
    let default_line = Line::from(vec![
        Span::styled("Default: ", Style::default().fg(primary_light())),
        Span::styled(
            format!("{} ", &model.options.default_platform.name()),
            default_style,
        ),
        if is_default_selected {
            Span::styled("[Enter to change]", Style::default().fg(text_secondary()))
        } else {
            Span::raw("")
        },
    ]);
    frame.render_widget(Paragraph::new(default_line), chunks[0]);

    let groups_len = model.options.platform_groups.len();
    let platforms_without_all = Model::platforms_without_all();

    let mut list_lines: Vec<Line> = Vec::new();
    let mut selected_rendered_line: usize = 0;

    // Groups section
    for (i, group) in model.options.platform_groups.iter().enumerate() {
        let list_index = i + 1;
        let is_selected = model.options.platform_list_index == list_index;
        if is_selected {
            selected_rendered_line = list_lines.len();
        }

        let line_style = if is_selected {
            Style::default().fg(text_primary()).bg(accent())
        } else {
            Style::default().fg(primary())
        };

        let count = group.platforms.len();
        list_lines.push(Line::from(Span::styled(
            format!(" ▣ {} ({} platforms)", group.name, count),
            line_style,
        )));
    }

    // "+ New Group" button
    let new_group_index = groups_len + 1;
    let is_new_group_selected = model.options.platform_list_index == new_group_index;
    if is_new_group_selected {
        selected_rendered_line = list_lines.len();
    }
    let new_group_style = if is_new_group_selected {
        Style::default().fg(text_primary()).bg(accent())
    } else {
        Style::default().fg(text_secondary())
    };
    list_lines.push(Line::from(Span::styled(" + New Group", new_group_style)));

    // Separator
    list_lines.push(Line::from(Span::styled(
        " ───────────────────────────",
        Style::default().fg(text_dimmed()),
    )));

    // Platform list with checkboxes (skip Platform::All)
    for (i, platform) in platforms_without_all.iter().enumerate() {
        let list_index = i + groups_len + 2;
        let is_selected = model.options.platform_list_index == list_index;
        if is_selected {
            selected_rendered_line = list_lines.len();
        }
        let is_enabled = model.options.enabled_platforms.contains(platform);

        let checkbox = if is_enabled { "[x]" } else { "[ ]" };

        let line_style = if is_selected {
            Style::default().fg(text_primary()).bg(accent())
        } else if is_enabled {
            Style::default().fg(text_primary())
        } else {
            Style::default().fg(text_dimmed())
        };

        list_lines.push(Line::from(Span::styled(
            format!(" {} {}", checkbox, platform.name()),
            line_style,
        )));
    }

    // Calculate scroll offset based on actual rendered line position
    let visible_height = chunks[1].height.saturating_sub(2) as usize;
    let scroll_offset = if selected_rendered_line >= visible_height {
        (selected_rendered_line - visible_height + 1) as u16
    } else {
        0
    };

    let platforms_list = Paragraph::new(list_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(accent()))
                .title(Span::styled(
                    " Groups & Platforms ",
                    Style::default().fg(primary_light()),
                )),
        )
        .scroll((scroll_offset, 0));
    frame.render_widget(platforms_list, chunks[1]);

    let help = Paragraph::new(Line::from(Span::styled(
        "[Enter] Toggle/Edit  [d] Delete group  [Tab] Tab  [Esc] Close",
        Style::default().fg(text_secondary()),
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
        Style::default().fg(text_secondary()),
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
            Style::default().fg(text_primary()).bg(bg_highlight())
        } else {
            Style::default().fg(text_primary())
        };

        let value_style = if is_selected {
            Style::default()
                .fg(primary_light())
                .bg(bg_highlight())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(primary_light())
                .add_modifier(Modifier::BOLD)
        };

        let desc_style = if is_selected {
            Style::default().fg(text_secondary()).bg(bg_highlight())
        } else {
            Style::default().fg(text_secondary())
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
            .border_style(Style::default().fg(accent()))
            .title(Span::styled(
                " Settings ",
                Style::default().fg(primary_light()),
            )),
    );
    frame.render_widget(settings_list, chunks[1]);

    let help_lines = vec![
        Line::from(Span::styled(
            "[Enter] Cycle  [s] Direction  [Tab] Switch tab",
            Style::default().fg(text_secondary()),
        )),
        Line::from(Span::styled(
            "[Esc] Close",
            Style::default().fg(text_secondary()),
        )),
    ];
    let help = Paragraph::new(help_lines);
    frame.render_widget(help, chunks[2]);
}

fn render_theme_tab(frame: &mut Frame, model: &Model, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(2),
        ])
        .split(area);

    let desc = Paragraph::new(Line::from(Span::styled(
        "Select a color theme:",
        Style::default().fg(text_secondary()),
    )));
    frame.render_widget(desc, chunks[0]);

    let mut theme_lines: Vec<Line> = Vec::new();
    for (i, theme) in Theme::ALL.iter().enumerate() {
        let is_selected = model.options.theme_list_index == i;
        let is_current = model.options.theme == *theme;

        let marker = if is_current { "●" } else { "○" };

        let line_style = if is_selected {
            Style::default().fg(text_primary()).bg(accent())
        } else if is_current {
            Style::default().fg(primary_light())
        } else {
            Style::default().fg(text_primary())
        };

        // Show color preview swatches including background colors
        let p = theme.palette();
        let preview = Line::from(vec![
            Span::styled(format!(" {} {:<20}", marker, theme.name()), line_style),
            Span::styled("██", Style::default().fg(p.bg)),
            Span::styled("██", Style::default().fg(p.bg_highlight)),
            Span::styled("██", Style::default().fg(p.primary)),
            Span::styled("██", Style::default().fg(p.green)),
            Span::styled("██", Style::default().fg(p.yellow)),
            Span::styled("██", Style::default().fg(p.accent)),
        ]);
        theme_lines.push(preview);
    }

    // Calculate scroll offset
    let visible_height = chunks[1].height.saturating_sub(2) as usize;
    let scroll_offset = if model.options.theme_list_index >= visible_height {
        (model.options.theme_list_index - visible_height + 1) as u16
    } else {
        0
    };

    let theme_list = Paragraph::new(theme_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(accent()))
                .title(Span::styled(
                    " Theme ",
                    Style::default().fg(primary_light()),
                )),
        )
        .scroll((scroll_offset, 0));
    frame.render_widget(theme_list, chunks[1]);

    let help = Paragraph::new(Line::from(Span::styled(
        "[Enter] Select  [Tab] Switch tab  [Esc] Close",
        Style::default().fg(text_secondary()),
    )));
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
        .style(Style::default().fg(text_primary()))
        .block(
            Block::default()
                .title(Span::styled(
                    " Keybinds ",
                    Style::default().fg(primary_light()),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(primary())),
        );

    frame.render_widget(popup, popup_area);
}

pub fn render_platform_popup(frame: &mut Frame, model: &Model) {
    let area = frame.area();
    let filters = model.selectable_filters();

    let popup_width = 35u16;
    let popup_height = (filters.len() as u16 + 5).min(20);
    let popup_x = area.width.saturating_sub(popup_width) / 2;
    let popup_y = area.height.saturating_sub(popup_height) / 2;
    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(Span::styled(
            " Select Platform ",
            Style::default().fg(primary_light()),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(accent()));
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

    let mut lines: Vec<Line> = Vec::new();
    let mut prev_was_group = false;
    for (i, filter) in filters.iter().enumerate() {
        let is_selected = model.ui.platform_popup_index == i;
        let is_current = model.platform_filter == *filter;

        // Add separator between groups and individual platforms
        let is_group = matches!(filter, PlatformFilter::Group(_));
        let is_single = matches!(filter, PlatformFilter::Single(_));
        if is_single && prev_was_group {
            lines.push(Line::from(Span::styled(
                " ───────────────────",
                Style::default().fg(text_dimmed()),
            )));
        }
        prev_was_group = is_group;

        let marker = if is_current { "●" } else { "○" };
        let prefix = if is_group { "▣ " } else { "" };
        let line_style = if is_selected {
            Style::default().fg(text_primary()).bg(accent())
        } else if is_current {
            Style::default().fg(primary_light())
        } else if is_group {
            Style::default().fg(primary())
        } else {
            Style::default().fg(text_primary())
        };

        lines.push(Line::from(Span::styled(
            format!(" {} {}{}", marker, prefix, filter.name()),
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

    let platform_list = Paragraph::new(lines).scroll((scroll_offset, 0));
    frame.render_widget(platform_list, chunks[0]);

    let help = Paragraph::new(Line::from(Span::styled(
        "[Enter] Select  [Esc] Cancel",
        Style::default().fg(text_secondary()),
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
            Style::default().fg(primary_light()),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(accent()));
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
        Style::default().fg(text_primary()).bg(accent())
    } else {
        Style::default().fg(text_primary())
    };

    let max_style = if max_selected {
        Style::default().fg(text_primary()).bg(accent())
    } else {
        Style::default().fg(text_primary())
    };

    let min_cursor = if min_selected { "▋" } else { "" };
    let max_cursor = if max_selected { "▋" } else { "" };

    let min_display = format!("{}{}", model.price_filter.min_input, min_cursor);
    let max_display = format!("{}{}", model.price_filter.max_input, max_cursor);

    let content = vec![
        Line::from(vec![
            Span::styled("Min: ", Style::default().fg(primary_light())),
            Span::styled(format!("{:<10}", min_display), min_style),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Max: ", Style::default().fg(primary_light())),
            Span::styled(format!("{:<10}", max_display), max_style),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "[Tab] Switch  [Enter] Apply",
            Style::default().fg(text_secondary()),
        )),
        Line::from(Span::styled(
            "[c] Clear  [Esc] Cancel",
            Style::default().fg(text_secondary()),
        )),
    ];

    let paragraph = Paragraph::new(content);
    frame.render_widget(paragraph, inner);
}

pub fn render_group_editor_popup(frame: &mut Frame, model: &Model) {
    let area = frame.area();
    let popup_width = 45u16;
    let popup_height = 22u16;
    let popup_x = area.width.saturating_sub(popup_width) / 2;
    let popup_y = area.height.saturating_sub(popup_height) / 2;
    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let title = if model.group_editor.editing_index.is_some() {
        " Edit Group "
    } else {
        " New Group "
    };

    let block = Block::default()
        .title(Span::styled(title, Style::default().fg(primary_light())))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(accent()));
    frame.render_widget(block, popup_area);

    let inner = Rect::new(
        popup_area.x + 2,
        popup_area.y + 2,
        popup_area.width - 4,
        popup_area.height - 4,
    );

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(inner);

    // Name field
    let name_active = model.group_editor.field == GroupEditorField::Name;
    let name_style = if name_active {
        Style::default().fg(text_primary()).bg(accent())
    } else {
        Style::default().fg(text_primary())
    };
    let cursor = if name_active { "▋" } else { "" };
    let name_line = Line::from(vec![
        Span::styled("Name: ", Style::default().fg(primary_light())),
        Span::styled(format!("{}{}", model.group_editor.name, cursor), name_style),
    ]);
    frame.render_widget(Paragraph::new(name_line), chunks[0]);

    // Platforms checklist
    let platforms = Model::platforms_without_all();
    let plat_active = model.group_editor.field == GroupEditorField::Platforms;

    let mut plat_lines: Vec<Line> = Vec::new();
    for (i, platform) in platforms.iter().enumerate() {
        let is_selected = plat_active && model.group_editor.platform_index == i;
        let is_checked = model.group_editor.selected_platforms.contains(platform);

        let checkbox = if is_checked { "[x]" } else { "[ ]" };

        let line_style = if is_selected {
            Style::default().fg(text_primary()).bg(accent())
        } else if is_checked {
            Style::default().fg(primary_light())
        } else {
            Style::default().fg(text_primary())
        };

        plat_lines.push(Line::from(Span::styled(
            format!(" {} {}", checkbox, platform.name()),
            line_style,
        )));
    }

    let visible_height = chunks[1].height.saturating_sub(2) as usize;
    let scroll_offset = if model.group_editor.platform_index >= visible_height {
        (model.group_editor.platform_index - visible_height + 1) as u16
    } else {
        0
    };

    let plat_block_style = if plat_active {
        Style::default().fg(accent())
    } else {
        Style::default().fg(text_dimmed())
    };

    let plat_list = Paragraph::new(plat_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(plat_block_style)
                .title(Span::styled(
                    " Platforms ",
                    Style::default().fg(primary_light()),
                )),
        )
        .scroll((scroll_offset, 0));
    frame.render_widget(plat_list, chunks[1]);

    let selected_count = model.group_editor.selected_platforms.len();
    let help_lines = vec![
        Line::from(Span::styled(
            format!("{} selected  [Tab] Switch  [s] Save", selected_count),
            Style::default().fg(text_secondary()),
        )),
        Line::from(Span::styled(
            "[Enter/Space] Toggle  [Esc] Cancel",
            Style::default().fg(text_secondary()),
        )),
    ];
    let help = Paragraph::new(help_lines);
    frame.render_widget(help, chunks[2]);
}
