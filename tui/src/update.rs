use dealve_core::models::{Platform, PlatformFilter, PlatformGroup};

use crate::message::Message;
use crate::model::{MenuItem, Model, OptionsTab, Popup, SortCriteria};

// Number of rows to scroll for PageUp/PageDown navigation
const PAGE_SCROLL_SIZE: usize = 20;

/// Flags returned by update to signal side effects needed
pub struct UpdateResult {
    pub msg: Option<Message>,
    pub needs_reload: bool,
    pub selection_changed: bool,
}

impl UpdateResult {
    fn none() -> Self {
        Self {
            msg: None,
            needs_reload: false,
            selection_changed: false,
        }
    }

    fn with_selection_changed() -> Self {
        Self {
            msg: None,
            needs_reload: false,
            selection_changed: true,
        }
    }

    fn with_reload() -> Self {
        Self {
            msg: None,
            needs_reload: true,
            selection_changed: true,
        }
    }

    fn with_msg(msg: Message) -> Self {
        Self {
            msg: Some(msg),
            needs_reload: false,
            selection_changed: false,
        }
    }
}

pub fn update(model: &mut Model, msg: Message) -> UpdateResult {
    match msg {
        // Navigation
        Message::SelectNext => {
            let filtered_count = model.filtered_deals().len();
            if filtered_count > 0 {
                let i = match model.ui.table_state.selected() {
                    Some(i) => {
                        if i >= filtered_count - 1 {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                model.select(Some(i));
            }
            UpdateResult::with_selection_changed()
        }
        Message::SelectPrevious => {
            let filtered_count = model.filtered_deals().len();
            if filtered_count > 0 {
                let i = match model.ui.table_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            filtered_count - 1
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                model.select(Some(i));
            }
            UpdateResult::with_selection_changed()
        }
        Message::SelectPageUp => {
            let filtered_count = model.filtered_deals().len();
            if filtered_count > 0 {
                let i = match model.ui.table_state.selected() {
                    Some(i) => i.saturating_sub(PAGE_SCROLL_SIZE),
                    None => 0,
                };
                model.select(Some(i));
            }
            UpdateResult::with_selection_changed()
        }
        Message::SelectPageDown => {
            let filtered_count = model.filtered_deals().len();
            if filtered_count > 0 {
                let i = match model.ui.table_state.selected() {
                    Some(i) => (i + PAGE_SCROLL_SIZE).min(filtered_count - 1),
                    None => 0,
                };
                model.select(Some(i));
            }
            UpdateResult::with_selection_changed()
        }
        Message::SelectTop => {
            let filtered_count = model.filtered_deals().len();
            if filtered_count > 0 {
                model.select(Some(0));
            }
            UpdateResult::with_selection_changed()
        }
        Message::SelectBottom => {
            let filtered_count = model.filtered_deals().len();
            if filtered_count > 0 {
                model.select(Some(filtered_count - 1));
            }
            UpdateResult::with_selection_changed()
        }
        Message::OpenSelectedDeal => {
            if let Some(i) = model.ui.table_state.selected() {
                let filtered = model.filtered_deals();
                if let Some(deal) = filtered.get(i) {
                    let _ = webbrowser::open(&deal.url);
                }
            }
            UpdateResult::none()
        }

        // Menu
        Message::ToggleMenu => {
            model.ui.show_menu = !model.ui.show_menu;
            if model.ui.show_menu {
                model.ui.menu_selected = 0;
            }
            UpdateResult::none()
        }
        Message::MenuNext => {
            model.ui.menu_selected = (model.ui.menu_selected + 1) % MenuItem::ALL.len();
            UpdateResult::none()
        }
        Message::MenuPrevious => {
            if model.ui.menu_selected == 0 {
                model.ui.menu_selected = MenuItem::ALL.len() - 1;
            } else {
                model.ui.menu_selected -= 1;
            }
            UpdateResult::none()
        }
        Message::MenuSelect => {
            match MenuItem::ALL[model.ui.menu_selected] {
                MenuItem::Browse => {
                    model.ui.show_menu = false;
                }
                MenuItem::Options => {
                    model.ui.popup = Popup::Options;
                }
                MenuItem::Keybinds => {
                    model.ui.popup = Popup::Keybinds;
                }
                MenuItem::Quit => {
                    return UpdateResult::with_msg(Message::Quit);
                }
            }
            UpdateResult::none()
        }

        // Filtering
        Message::StartFilter => {
            model.filter.active = true;
            model.filter.text = model.active_search_query.clone().unwrap_or_default();
            UpdateResult::none()
        }
        Message::CancelFilter => {
            model.filter.active = false;
            model.filter.text = model.active_search_query.clone().unwrap_or_default();
            model.select(Some(0));
            UpdateResult::with_selection_changed()
        }
        Message::ConfirmFilter => {
            model.filter.active = false;
            let normalized = model.filter.text.trim().to_string();
            let next_query = if normalized.is_empty() {
                None
            } else {
                Some(normalized)
            };
            let has_changed = model.active_search_query != next_query;
            model.active_search_query = next_query;
            if model.is_search_mode() && !model.is_search_sort_supported() {
                model.sort_state.criteria = SortCriteria::Price;
            }
            model.filter.text = model.active_search_query.clone().unwrap_or_default();
            model.select(Some(0));
            if has_changed {
                UpdateResult::with_reload()
            } else {
                UpdateResult::with_selection_changed()
            }
        }
        Message::FilterPush(c) => {
            model.filter.text.push(c);
            model.select(Some(0));
            UpdateResult::with_selection_changed()
        }
        Message::FilterPop => {
            model.filter.text.pop();
            model.select(Some(0));
            UpdateResult::with_selection_changed()
        }
        Message::ClearFilters => {
            if !model.filter.text.is_empty()
                || model.price_filter.is_active()
                || model.active_search_query.is_some()
            {
                let had_search_query = model.active_search_query.take().is_some();
                model.filter.text.clear();
                model.filter.active = false;
                model.price_filter.clear();
                model.select(Some(0));
                return if had_search_query {
                    UpdateResult::with_reload()
                } else {
                    UpdateResult::with_selection_changed()
                };
            }
            UpdateResult::none()
        }

        // Price filter
        Message::OpenPriceFilter => {
            model.price_filter.min_input = model
                .price_filter
                .active_min
                .map(|v| format!("{:.0}", v))
                .unwrap_or_default();
            model.price_filter.max_input = model
                .price_filter
                .active_max
                .map(|v| format!("{:.0}", v))
                .unwrap_or_default();
            model.price_filter.selected_field = 0;
            model.ui.popup = Popup::PriceFilter;
            UpdateResult::none()
        }
        Message::PriceFilterSwitchField => {
            model.price_filter.selected_field = 1 - model.price_filter.selected_field;
            UpdateResult::none()
        }
        Message::PriceFilterPush(c) => {
            if c.is_ascii_digit() || c == '.' {
                let input = if model.price_filter.selected_field == 0 {
                    &mut model.price_filter.min_input
                } else {
                    &mut model.price_filter.max_input
                };
                if input.len() < 8 {
                    input.push(c);
                }
            }
            UpdateResult::none()
        }
        Message::PriceFilterPop => {
            let input = if model.price_filter.selected_field == 0 {
                &mut model.price_filter.min_input
            } else {
                &mut model.price_filter.max_input
            };
            input.pop();
            UpdateResult::none()
        }
        Message::PriceFilterApply => {
            model.price_filter.apply();
            model.ui.popup = Popup::None;
            model.select(Some(0));
            UpdateResult::with_selection_changed()
        }
        Message::PriceFilterClear => {
            model.price_filter.clear();
            model.ui.popup = Popup::None;
            model.select(Some(0));
            UpdateResult::with_selection_changed()
        }

        // Platform popup
        Message::OpenPlatformPopup => {
            let filters = model.selectable_filters();
            model.ui.platform_popup_index = filters
                .iter()
                .position(|f| *f == model.platform_filter)
                .unwrap_or(0);
            model.ui.popup = Popup::Platform;
            UpdateResult::none()
        }
        Message::PlatformPopupNext => {
            let filters = model.selectable_filters();
            if !filters.is_empty() {
                model.ui.platform_popup_index = (model.ui.platform_popup_index + 1) % filters.len();
            }
            UpdateResult::none()
        }
        Message::PlatformPopupPrev => {
            let filters = model.selectable_filters();
            if !filters.is_empty() {
                if model.ui.platform_popup_index == 0 {
                    model.ui.platform_popup_index = filters.len() - 1;
                } else {
                    model.ui.platform_popup_index -= 1;
                }
            }
            UpdateResult::none()
        }
        Message::PlatformPopupSelect => {
            let filters = model.selectable_filters();
            if let Some(filter) = filters.get(model.ui.platform_popup_index) {
                let changed = model.platform_filter != *filter;
                model.platform_filter = filter.clone();
                model.ui.popup = Popup::None;
                if changed {
                    model.select(Some(0));
                    return UpdateResult::with_reload();
                }
            } else {
                model.ui.popup = Popup::None;
            }
            UpdateResult::none()
        }

        // Sort
        Message::ToggleSortDirection => {
            model.sort_state.direction = model.sort_state.direction.toggle();
            model.select(Some(0));
            if model.is_search_mode() {
                UpdateResult::with_selection_changed()
            } else {
                UpdateResult::with_reload()
            }
        }
        Message::NextSortCriteria => {
            model.sort_state.criteria = if model.is_search_mode() {
                model.sort_state.criteria.toggle_search()
            } else {
                model.sort_state.criteria.next()
            };
            model.select(Some(0));
            if model.is_search_mode() {
                UpdateResult::with_selection_changed()
            } else {
                UpdateResult::with_reload()
            }
        }
        Message::PrevSortCriteria => {
            model.sort_state.criteria = if model.is_search_mode() {
                model.sort_state.criteria.toggle_search()
            } else {
                model.sort_state.criteria.prev()
            };
            model.select(Some(0));
            if model.is_search_mode() {
                UpdateResult::with_selection_changed()
            } else {
                UpdateResult::with_reload()
            }
        }

        // Popups
        Message::ClosePopup => {
            model.ui.popup = Popup::None;
            model.options.platform_list_index = 0;
            model.options.region_list_index = 0;
            model.options.advanced_list_index = 0;
            model.options.theme_list_index = 0;
            UpdateResult::none()
        }

        // Options
        Message::OptionsNextTab => {
            model.options.current_tab = (model.options.current_tab + 1) % OptionsTab::ALL.len();
            model.options.platform_list_index = 0;
            model.options.region_list_index = 0;
            model.options.advanced_list_index = 0;
            model.options.theme_list_index = 0;
            UpdateResult::none()
        }
        Message::OptionsPrevTab => {
            if model.options.current_tab == 0 {
                model.options.current_tab = OptionsTab::ALL.len() - 1;
            } else {
                model.options.current_tab -= 1;
            }
            model.options.platform_list_index = 0;
            model.options.region_list_index = 0;
            model.options.advanced_list_index = 0;
            model.options.theme_list_index = 0;
            UpdateResult::none()
        }
        Message::OptionsNextItem => {
            match OptionsTab::ALL[model.options.current_tab] {
                OptionsTab::Region => {
                    model.options.region_list_index = (model.options.region_list_index + 1)
                        % dealve_core::models::Region::ALL.len();
                }
                OptionsTab::Platforms => {
                    // 1 (default) + groups count + 1 (separator/"+ New Group") + platforms count
                    let total_items = 1 + model.options.platform_groups.len() + 1 + Model::platforms_without_all().len();
                    model.options.platform_list_index =
                        (model.options.platform_list_index + 1) % total_items;
                }
                OptionsTab::Advanced => {
                    model.options.advanced_list_index = (model.options.advanced_list_index + 1) % 3;
                }
                OptionsTab::Theme => {
                    model.options.theme_list_index = (model.options.theme_list_index + 1)
                        % crate::view::styles::Theme::ALL.len();
                }
            }
            UpdateResult::none()
        }
        Message::OptionsPrevItem => {
            match OptionsTab::ALL[model.options.current_tab] {
                OptionsTab::Region => {
                    if model.options.region_list_index == 0 {
                        model.options.region_list_index =
                            dealve_core::models::Region::ALL.len() - 1;
                    } else {
                        model.options.region_list_index -= 1;
                    }
                }
                OptionsTab::Platforms => {
                    let total_items = 1 + model.options.platform_groups.len() + 1 + Model::platforms_without_all().len();
                    if model.options.platform_list_index == 0 {
                        model.options.platform_list_index = total_items - 1;
                    } else {
                        model.options.platform_list_index -= 1;
                    }
                }
                OptionsTab::Advanced => {
                    if model.options.advanced_list_index == 0 {
                        model.options.advanced_list_index = 2;
                    } else {
                        model.options.advanced_list_index -= 1;
                    }
                }
                OptionsTab::Theme => {
                    if model.options.theme_list_index == 0 {
                        model.options.theme_list_index = crate::view::styles::Theme::ALL.len() - 1;
                    } else {
                        model.options.theme_list_index -= 1;
                    }
                }
            }
            UpdateResult::none()
        }
        Message::OptionsToggleItem => {
            let mut needs_reload = false;
            match OptionsTab::ALL[model.options.current_tab] {
                OptionsTab::Region => {
                    if let Some(&region) =
                        dealve_core::models::Region::ALL.get(model.options.region_list_index)
                    {
                        if model.options.region != region {
                            model.options.region = region;
                            model.region = region;
                            needs_reload = true;
                        }
                    }
                    model.options.save_to_config();
                }
                OptionsTab::Platforms => {
                    let groups_len = model.options.platform_groups.len();
                    let idx = model.options.platform_list_index;
                    if idx == 0 {
                        // Default platform
                        cycle_default_platform(model);
                    } else if idx <= groups_len {
                        // Edit existing group
                        return UpdateResult::with_msg(Message::EditGroup(idx - 1));
                    } else if idx == groups_len + 1 {
                        // "+ New Group"
                        return UpdateResult::with_msg(Message::OpenGroupEditor);
                    } else {
                        // Individual platform checkbox
                        let platform_idx = idx - groups_len - 2;
                        let platforms = Model::platforms_without_all();
                        if let Some(&platform) = platforms.get(platform_idx) {
                            if model.options.enabled_platforms.contains(&platform) {
                                model.options.enabled_platforms.remove(&platform);
                            } else {
                                model.options.enabled_platforms.insert(platform);
                            }
                        }
                    }
                    model.options.save_to_config();
                }
                OptionsTab::Advanced => {
                    match model.options.advanced_list_index {
                        0 => {
                            model.options.default_sort.criteria =
                                model.options.default_sort.criteria.next();
                        }
                        1 => {
                            model.options.deals_page_size = match model.options.deals_page_size {
                                25 => 50,
                                50 => 100,
                                100 => 200,
                                _ => 25,
                            };
                            model.deals_page_size = model.options.deals_page_size;
                        }
                        2 => {
                            model.options.game_info_delay_ms =
                                match model.options.game_info_delay_ms {
                                    100 => 200,
                                    200 => 300,
                                    300 => 500,
                                    _ => 100,
                                };
                            model.game_info_delay_ms = model.options.game_info_delay_ms;
                        }
                        _ => {}
                    }
                    model.options.save_to_config();
                }
                OptionsTab::Theme => {
                    if let Some(&theme) =
                        crate::view::styles::Theme::ALL.get(model.options.theme_list_index)
                    {
                        model.options.theme = theme;
                        crate::view::styles::set_active_theme(theme);
                    }
                    model.options.save_to_config();
                }
            }
            if needs_reload {
                model.ui.popup = Popup::None;
                UpdateResult::with_reload()
            } else {
                UpdateResult::none()
            }
        }
        Message::OptionsToggleSortDirection => {
            if OptionsTab::ALL[model.options.current_tab] == OptionsTab::Advanced
                && model.options.advanced_list_index == 0
            {
                model.options.default_sort.direction =
                    model.options.default_sort.direction.toggle();
                model.options.save_to_config();
            }
            UpdateResult::none()
        }

        // Group editor
        Message::OpenGroupEditor => {
            model.group_editor = crate::model::GroupEditorState::default();
            model.ui.popup = Popup::GroupEditor;
            UpdateResult::none()
        }
        Message::EditGroup(idx) => {
            if let Some(group) = model.options.platform_groups.get(idx) {
                model.group_editor.name = group.name.clone();
                model.group_editor.selected_platforms =
                    group.platforms.iter().copied().collect();
                model.group_editor.field = crate::model::GroupEditorField::Name;
                model.group_editor.platform_index = 0;
                model.group_editor.editing_index = Some(idx);
                model.ui.popup = Popup::GroupEditor;
            }
            UpdateResult::none()
        }
        Message::DeleteGroup(idx) => {
            if idx < model.options.platform_groups.len() {
                model.options.platform_groups.remove(idx);
                model.options.save_to_config();
            }
            UpdateResult::none()
        }
        Message::GroupEditorSwitchField => {
            model.group_editor.field = match model.group_editor.field {
                crate::model::GroupEditorField::Name => crate::model::GroupEditorField::Platforms,
                crate::model::GroupEditorField::Platforms => crate::model::GroupEditorField::Name,
            };
            UpdateResult::none()
        }
        Message::GroupEditorPush(c) => {
            if model.group_editor.field == crate::model::GroupEditorField::Name
                && model.group_editor.name.len() < 20
            {
                model.group_editor.name.push(c);
            }
            UpdateResult::none()
        }
        Message::GroupEditorPop => {
            if model.group_editor.field == crate::model::GroupEditorField::Name {
                model.group_editor.name.pop();
            }
            UpdateResult::none()
        }
        Message::GroupEditorNext => {
            if model.group_editor.field == crate::model::GroupEditorField::Platforms {
                let platforms = Model::platforms_without_all();
                if !platforms.is_empty() {
                    model.group_editor.platform_index =
                        (model.group_editor.platform_index + 1) % platforms.len();
                }
            }
            UpdateResult::none()
        }
        Message::GroupEditorPrev => {
            if model.group_editor.field == crate::model::GroupEditorField::Platforms {
                let platforms = Model::platforms_without_all();
                if !platforms.is_empty() {
                    if model.group_editor.platform_index == 0 {
                        model.group_editor.platform_index = platforms.len() - 1;
                    } else {
                        model.group_editor.platform_index -= 1;
                    }
                }
            }
            UpdateResult::none()
        }
        Message::GroupEditorToggle => {
            if model.group_editor.field == crate::model::GroupEditorField::Platforms {
                let platforms = Model::platforms_without_all();
                if let Some(&platform) = platforms.get(model.group_editor.platform_index) {
                    if model.group_editor.selected_platforms.contains(&platform) {
                        model.group_editor.selected_platforms.remove(&platform);
                    } else {
                        model.group_editor.selected_platforms.insert(platform);
                    }
                }
            }
            UpdateResult::none()
        }
        Message::GroupEditorSave => {
            let name = model.group_editor.name.trim().to_string();
            if !name.is_empty() && !model.group_editor.selected_platforms.is_empty() {
                let platforms: Vec<Platform> = Platform::ALL
                    .iter()
                    .copied()
                    .filter(|p| model.group_editor.selected_platforms.contains(p))
                    .collect();
                let group = PlatformGroup {
                    name,
                    platforms,
                };
                if let Some(idx) = model.group_editor.editing_index {
                    if idx < model.options.platform_groups.len() {
                        model.options.platform_groups[idx] = group;
                    }
                } else {
                    model.options.platform_groups.push(group);
                }
                model.options.save_to_config();
            }
            model.ui.popup = Popup::Options;
            UpdateResult::none()
        }
        Message::GroupEditorCancel => {
            model.ui.popup = Popup::Options;
            UpdateResult::none()
        }

        // Data loading results
        Message::DealsLoaded {
            deals,
            is_more,
            page_size,
        } => {
            if !is_more {
                model.pagination.has_more = false;
            }
            model.deals = deals;
            model.pagination.offset = page_size;
            model.select(Some(0));
            model.loading.deals = false;
            model.error_clear();
            UpdateResult::with_selection_changed()
        }
        Message::MoreDealsLoaded {
            deals,
            is_more,
            page_size,
        } => {
            if !is_more {
                model.pagination.has_more = false;
            }
            model.deals.extend(deals);
            model.pagination.offset += page_size;
            model.pagination.loading_more = false;
            model.error_clear();
            UpdateResult::none()
        }
        Message::DealsLoadFailed(error) => {
            model.error = Some(error);
            model.loading.deals = false;
            model.pagination.loading_more = false;
            UpdateResult::none()
        }
        Message::PriceHistoryLoaded { game_id, history } => {
            model.price_history_cache.insert(game_id.clone(), history);
            if model.loading.price_history.as_ref() == Some(&game_id) {
                model.loading.price_history = None;
            }
            UpdateResult::none()
        }

        // System
        Message::RequestRefresh => UpdateResult::with_reload(),

        Message::Tick => {
            if model.loading.deals || model.pagination.loading_more {
                model.ui.spinner_frame = (model.ui.spinner_frame + 1) % 10;
            }
            UpdateResult::none()
        }

        Message::Quit => {
            model.should_quit = true;
            UpdateResult::none()
        }
    }
}

fn cycle_default_platform(model: &mut Model) {
    let current_idx = Platform::ALL
        .iter()
        .position(|&p| p == model.options.default_platform)
        .unwrap_or(0);

    let len = Platform::ALL.len();
    for i in 1..=len {
        let next_idx = (current_idx + i) % len;
        let next_platform = Platform::ALL[next_idx];
        if model.options.enabled_platforms.contains(&next_platform) {
            model.options.default_platform = next_platform;
            model.platform_filter = if next_platform == Platform::All {
                PlatformFilter::All
            } else {
                PlatformFilter::Single(next_platform)
            };
            return;
        }
    }
}
