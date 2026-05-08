use dealve_core::models::{Platform, PlatformGroup};

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
                    let total_items = 1
                        + model.options.platform_groups.len()
                        + 1
                        + Model::platforms_without_all().len();
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
                    let total_items = 1
                        + model.options.platform_groups.len()
                        + 1
                        + Model::platforms_without_all().len();
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
                model.group_editor.selected_platforms = group.platforms.iter().copied().collect();
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
                let group = PlatformGroup { name, platforms };
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
    let filters = model.selectable_filters();
    if filters.is_empty() {
        return;
    }
    let current_idx = filters
        .iter()
        .position(|f| *f == model.options.default_platform)
        .unwrap_or(0);
    let next_idx = (current_idx + 1) % filters.len();
    model.options.default_platform = filters[next_idx].clone();
    model.platform_filter = filters[next_idx].clone();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        FilterState, LoadingState, OptionsState, PaginationState, PriceFilterState, SortDirection,
        SortState, UiState,
    };
    use dealve_core::models::{Deal, Price, Region, Shop};
    use std::collections::HashMap;

    fn make_model() -> Model {
        Model {
            deals: vec![],
            game_info_cache: HashMap::new(),
            price_history_cache: HashMap::new(),
            ui: UiState::default(),
            filter: FilterState::default(),
            active_search_query: None,
            price_filter: PriceFilterState::default(),
            sort_state: SortState::default(),
            platform_filter: Platform::All,
            region: Region::default(),
            pagination: PaginationState::default(),
            loading: LoadingState::default(),
            options: OptionsState::default(),
            api_key: None,
            deals_page_size: 50,
            game_info_delay_ms: 200,
            error: None,
            should_quit: false,
        }
    }

    fn make_deal(id: &str, title: &str, shop_id: u32, price: f64, discount: u8) -> Deal {
        Deal {
            id: id.to_string(),
            title: title.to_string(),
            shop: Shop {
                id: shop_id.to_string(),
                name: "Shop".to_string(),
            },
            price: Price {
                amount: price,
                currency: "USD".to_string(),
                discount,
            },
            regular_price: price,
            url: "https://example.com".to_string(),
            history_low: None,
        }
    }

    fn make_model_with_n_deals(n: usize) -> Model {
        let mut m = make_model();
        m.deals = (0..n)
            .map(|i| make_deal(&format!("g{i}"), &format!("Game {i}"), 61, i as f64, 10))
            .collect();
        m
    }

    // --- Navigation ---

    #[test]
    fn select_next_does_nothing_on_empty_list() {
        let mut m = make_model();
        m.select(None);
        let r = update(&mut m, Message::SelectNext);
        assert!(r.selection_changed);
        assert_eq!(m.ui.table_state.selected(), None);
    }

    #[test]
    fn select_next_advances_then_wraps() {
        let mut m = make_model_with_n_deals(3);
        m.select(Some(0));
        update(&mut m, Message::SelectNext);
        assert_eq!(m.ui.table_state.selected(), Some(1));
        update(&mut m, Message::SelectNext);
        update(&mut m, Message::SelectNext);
        assert_eq!(m.ui.table_state.selected(), Some(0));
    }

    #[test]
    fn select_previous_wraps_to_last() {
        let mut m = make_model_with_n_deals(3);
        m.select(Some(0));
        update(&mut m, Message::SelectPrevious);
        assert_eq!(m.ui.table_state.selected(), Some(2));
    }

    #[test]
    fn page_up_saturates_at_zero() {
        let mut m = make_model_with_n_deals(5);
        m.select(Some(2));
        update(&mut m, Message::SelectPageUp);
        assert_eq!(m.ui.table_state.selected(), Some(0));
    }

    #[test]
    fn page_down_clamps_to_last() {
        let mut m = make_model_with_n_deals(5);
        m.select(Some(2));
        update(&mut m, Message::SelectPageDown);
        assert_eq!(m.ui.table_state.selected(), Some(4));
    }

    #[test]
    fn select_top_and_bottom() {
        let mut m = make_model_with_n_deals(10);
        m.select(Some(5));
        update(&mut m, Message::SelectTop);
        assert_eq!(m.ui.table_state.selected(), Some(0));
        update(&mut m, Message::SelectBottom);
        assert_eq!(m.ui.table_state.selected(), Some(9));
    }

    // --- Menu ---

    #[test]
    fn toggle_menu_opens_and_resets_selection() {
        let mut m = make_model();
        m.ui.menu_selected = 2;
        update(&mut m, Message::ToggleMenu);
        assert!(m.ui.show_menu);
        assert_eq!(m.ui.menu_selected, 0);
        update(&mut m, Message::ToggleMenu);
        assert!(!m.ui.show_menu);
    }

    #[test]
    fn menu_next_and_previous_wrap() {
        let mut m = make_model();
        m.ui.menu_selected = 0;
        update(&mut m, Message::MenuPrevious);
        assert_eq!(m.ui.menu_selected, MenuItem::ALL.len() - 1);
        update(&mut m, Message::MenuNext);
        assert_eq!(m.ui.menu_selected, 0);
    }

    #[test]
    fn menu_select_quit_emits_quit() {
        let mut m = make_model();
        m.ui.menu_selected = MenuItem::ALL
            .iter()
            .position(|i| *i == MenuItem::Quit)
            .unwrap();
        let r = update(&mut m, Message::MenuSelect);
        assert!(matches!(r.msg, Some(Message::Quit)));
    }

    // --- Filter (text search) ---

    #[test]
    fn start_filter_activates_and_seeds_text_from_active_query() {
        let mut m = make_model();
        m.active_search_query = Some("zelda".to_string());
        update(&mut m, Message::StartFilter);
        assert!(m.filter.active);
        assert_eq!(m.filter.text, "zelda");
    }

    #[test]
    fn filter_push_pop_resets_selection() {
        let mut m = make_model_with_n_deals(3);
        m.select(Some(2));
        update(&mut m, Message::FilterPush('a'));
        assert_eq!(m.filter.text, "a");
        assert_eq!(m.ui.table_state.selected(), Some(0));
        update(&mut m, Message::FilterPop);
        assert_eq!(m.filter.text, "");
    }

    #[test]
    fn confirm_filter_empty_clears_search_query() {
        let mut m = make_model();
        m.active_search_query = Some("old".to_string());
        m.filter.text = "   ".to_string();
        let r = update(&mut m, Message::ConfirmFilter);
        assert!(!m.filter.active);
        assert_eq!(m.active_search_query, None);
        assert!(r.needs_reload, "search query change must trigger reload");
    }

    #[test]
    fn confirm_filter_with_new_query_triggers_reload() {
        let mut m = make_model();
        m.filter.text = "celeste".to_string();
        let r = update(&mut m, Message::ConfirmFilter);
        assert_eq!(m.active_search_query.as_deref(), Some("celeste"));
        assert!(r.needs_reload);
    }

    #[test]
    fn confirm_filter_same_query_no_reload() {
        let mut m = make_model();
        m.active_search_query = Some("celeste".to_string());
        m.filter.text = "celeste".to_string();
        let r = update(&mut m, Message::ConfirmFilter);
        assert!(!r.needs_reload);
        assert!(r.selection_changed);
    }

    #[test]
    fn confirm_filter_forces_price_sort_when_search_sort_unsupported() {
        let mut m = make_model();
        m.sort_state.criteria = SortCriteria::Hottest;
        m.filter.text = "kirby".to_string();
        update(&mut m, Message::ConfirmFilter);
        assert_eq!(m.sort_state.criteria, SortCriteria::Price);
    }

    #[test]
    fn confirm_filter_keeps_supported_sort_in_search_mode() {
        let mut m = make_model();
        m.sort_state.criteria = SortCriteria::Cut;
        m.filter.text = "kirby".to_string();
        update(&mut m, Message::ConfirmFilter);
        assert_eq!(m.sort_state.criteria, SortCriteria::Cut);
    }

    #[test]
    fn cancel_filter_restores_text_from_active_query() {
        let mut m = make_model();
        m.active_search_query = Some("mario".to_string());
        m.filter.active = true;
        m.filter.text = "wario".to_string();
        update(&mut m, Message::CancelFilter);
        assert!(!m.filter.active);
        assert_eq!(m.filter.text, "mario");
    }

    #[test]
    fn clear_filters_with_search_query_triggers_reload() {
        let mut m = make_model();
        m.active_search_query = Some("foo".to_string());
        let r = update(&mut m, Message::ClearFilters);
        assert!(r.needs_reload);
        assert_eq!(m.active_search_query, None);
    }

    #[test]
    fn clear_filters_no_state_returns_none() {
        let mut m = make_model();
        let r = update(&mut m, Message::ClearFilters);
        assert!(!r.needs_reload);
        assert!(!r.selection_changed);
    }

    #[test]
    fn clear_filters_with_only_price_filter_does_not_reload() {
        let mut m = make_model();
        m.price_filter.active_min = Some(5.0);
        let r = update(&mut m, Message::ClearFilters);
        assert!(!r.needs_reload);
        assert!(r.selection_changed);
        assert!(!m.price_filter.is_active());
    }

    // --- Price filter ---

    #[test]
    fn price_filter_push_accepts_only_digits_and_dot() {
        let mut m = make_model();
        update(&mut m, Message::PriceFilterPush('1'));
        update(&mut m, Message::PriceFilterPush('a'));
        update(&mut m, Message::PriceFilterPush('.'));
        update(&mut m, Message::PriceFilterPush('5'));
        assert_eq!(m.price_filter.min_input, "1.5");
    }

    #[test]
    fn price_filter_push_respects_max_length() {
        let mut m = make_model();
        for _ in 0..12 {
            update(&mut m, Message::PriceFilterPush('9'));
        }
        assert_eq!(m.price_filter.min_input.len(), 8);
    }

    #[test]
    fn price_filter_switch_field_toggles_target() {
        let mut m = make_model();
        update(&mut m, Message::PriceFilterPush('1'));
        update(&mut m, Message::PriceFilterSwitchField);
        update(&mut m, Message::PriceFilterPush('2'));
        assert_eq!(m.price_filter.min_input, "1");
        assert_eq!(m.price_filter.max_input, "2");
    }

    #[test]
    fn price_filter_apply_parses_and_clear_resets() {
        let mut m = make_model();
        m.price_filter.min_input = "10".to_string();
        m.price_filter.max_input = "20".to_string();
        update(&mut m, Message::PriceFilterApply);
        assert_eq!(m.price_filter.active_min, Some(10.0));
        assert_eq!(m.price_filter.active_max, Some(20.0));
        update(&mut m, Message::PriceFilterClear);
        assert!(!m.price_filter.is_active());
    }

    // --- Sort ---

    #[test]
    fn toggle_sort_direction_browse_mode_reloads() {
        let mut m = make_model();
        let r = update(&mut m, Message::ToggleSortDirection);
        assert_eq!(m.sort_state.direction, SortDirection::Descending);
        assert!(r.needs_reload);
    }

    #[test]
    fn toggle_sort_direction_search_mode_does_not_reload() {
        let mut m = make_model();
        m.active_search_query = Some("q".to_string());
        let r = update(&mut m, Message::ToggleSortDirection);
        assert!(!r.needs_reload);
        assert!(r.selection_changed);
    }

    #[test]
    fn next_sort_criteria_browse_mode_cycles_full() {
        let mut m = make_model();
        m.sort_state.criteria = SortCriteria::Price;
        update(&mut m, Message::NextSortCriteria);
        assert_eq!(m.sort_state.criteria, SortCriteria::Cut);
    }

    #[test]
    fn next_sort_criteria_in_search_mode_only_toggles_price_cut() {
        let mut m = make_model();
        m.active_search_query = Some("q".to_string());
        m.sort_state.criteria = SortCriteria::Price;
        update(&mut m, Message::NextSortCriteria);
        assert_eq!(m.sort_state.criteria, SortCriteria::Cut);
        update(&mut m, Message::NextSortCriteria);
        assert_eq!(m.sort_state.criteria, SortCriteria::Price);
    }

    // --- Platform popup ---

    #[test]
    fn platform_popup_select_changing_triggers_reload() {
        let mut m = make_model();
        m.platform_filter = Platform::All;
        update(&mut m, Message::OpenPlatformPopup);
        let enabled = m.enabled_platforms();
        let target_idx = enabled.iter().position(|p| *p == Platform::Steam).unwrap();
        m.ui.platform_popup_index = target_idx;
        let r = update(&mut m, Message::PlatformPopupSelect);
        assert_eq!(m.platform_filter, Platform::Steam);
        assert_eq!(m.ui.popup, Popup::None);
        assert!(r.needs_reload);
    }

    #[test]
    fn platform_popup_select_same_no_reload() {
        let mut m = make_model();
        m.platform_filter = Platform::All;
        update(&mut m, Message::OpenPlatformPopup);
        let r = update(&mut m, Message::PlatformPopupSelect);
        assert!(!r.needs_reload);
    }

    // --- Data loading ---

    #[test]
    fn deals_loaded_replaces_deals_resets_selection_and_clears_error() {
        let mut m = make_model();
        m.error = Some("boom".to_string());
        m.loading.deals = true;
        m.pagination.has_more = true;
        let new_deals = vec![make_deal("a", "A", 61, 1.0, 0)];
        update(
            &mut m,
            Message::DealsLoaded {
                deals: new_deals,
                is_more: false,
                page_size: 50,
            },
        );
        assert_eq!(m.deals.len(), 1);
        assert!(!m.pagination.has_more);
        assert_eq!(m.pagination.offset, 50);
        assert_eq!(m.ui.table_state.selected(), Some(0));
        assert!(m.error.is_none());
        assert!(!m.loading.deals);
    }

    #[test]
    fn more_deals_loaded_extends_and_increments_offset() {
        let mut m = make_model();
        m.deals = vec![make_deal("a", "A", 61, 1.0, 0)];
        m.pagination.offset = 50;
        m.pagination.loading_more = true;
        update(
            &mut m,
            Message::MoreDealsLoaded {
                deals: vec![make_deal("b", "B", 61, 2.0, 0)],
                is_more: true,
                page_size: 50,
            },
        );
        assert_eq!(m.deals.len(), 2);
        assert_eq!(m.pagination.offset, 100);
        assert!(!m.pagination.loading_more);
    }

    #[test]
    fn deals_load_failed_records_error_and_clears_loading() {
        let mut m = make_model();
        m.loading.deals = true;
        m.pagination.loading_more = true;
        update(&mut m, Message::DealsLoadFailed("nope".to_string()));
        assert_eq!(m.error.as_deref(), Some("nope"));
        assert!(!m.loading.deals);
        assert!(!m.pagination.loading_more);
    }

    #[test]
    fn price_history_loaded_caches_and_clears_loading_marker() {
        let mut m = make_model();
        m.loading.price_history = Some("g1".to_string());
        update(
            &mut m,
            Message::PriceHistoryLoaded {
                game_id: "g1".to_string(),
                history: vec![],
            },
        );
        assert!(m.price_history_cache.contains_key("g1"));
        assert!(m.loading.price_history.is_none());
    }

    #[test]
    fn price_history_loaded_for_other_game_keeps_loading_marker() {
        let mut m = make_model();
        m.loading.price_history = Some("g1".to_string());
        update(
            &mut m,
            Message::PriceHistoryLoaded {
                game_id: "g2".to_string(),
                history: vec![],
            },
        );
        assert_eq!(m.loading.price_history.as_deref(), Some("g1"));
    }

    // --- System ---

    #[test]
    fn quit_sets_should_quit() {
        let mut m = make_model();
        update(&mut m, Message::Quit);
        assert!(m.should_quit);
    }

    #[test]
    fn tick_advances_spinner_only_when_loading() {
        let mut m = make_model();
        update(&mut m, Message::Tick);
        assert_eq!(m.ui.spinner_frame, 0);
        m.loading.deals = true;
        update(&mut m, Message::Tick);
        assert_eq!(m.ui.spinner_frame, 1);
    }

    // --- Pure model query: filtered_deals & price_filter ---

    #[test]
    fn filtered_deals_filters_by_platform_shop_id() {
        let mut m = make_model();
        m.deals = vec![
            make_deal("a", "A", 61, 1.0, 0), // Steam
            make_deal("b", "B", 35, 2.0, 0), // GOG
        ];
        m.platform_filter = Platform::Steam;
        let filtered = m.filtered_deals();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "a");
    }

    #[test]
    fn filtered_deals_applies_price_filter() {
        let mut m = make_model();
        m.deals = vec![
            make_deal("a", "A", 61, 5.0, 0),
            make_deal("b", "B", 61, 15.0, 0),
            make_deal("c", "C", 61, 25.0, 0),
        ];
        m.price_filter.active_min = Some(10.0);
        m.price_filter.active_max = Some(20.0);
        let filtered = m.filtered_deals();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "b");
    }

    #[test]
    fn price_filter_matches_bounds() {
        let mut pf = PriceFilterState::default();
        pf.active_min = Some(10.0);
        pf.active_max = Some(20.0);
        assert!(!pf.matches(9.99));
        assert!(pf.matches(10.0));
        assert!(pf.matches(15.0));
        assert!(pf.matches(20.0));
        assert!(!pf.matches(20.01));
    }

    #[test]
    fn filtered_deals_sorts_in_search_mode_by_price() {
        let mut m = make_model();
        m.active_search_query = Some("q".to_string());
        m.sort_state.criteria = SortCriteria::Price;
        m.sort_state.direction = SortDirection::Ascending;
        m.deals = vec![
            make_deal("a", "A", 61, 30.0, 0),
            make_deal("b", "B", 61, 10.0, 0),
            make_deal("c", "C", 61, 20.0, 0),
        ];
        let filtered = m.filtered_deals();
        let prices: Vec<f64> = filtered.iter().map(|d| d.price.amount).collect();
        assert_eq!(prices, vec![10.0, 20.0, 30.0]);
    }

    #[test]
    fn filtered_deals_sort_search_descending() {
        let mut m = make_model();
        m.active_search_query = Some("q".to_string());
        m.sort_state.criteria = SortCriteria::Price;
        m.sort_state.direction = SortDirection::Descending;
        m.deals = vec![
            make_deal("a", "A", 61, 10.0, 0),
            make_deal("b", "B", 61, 30.0, 0),
            make_deal("c", "C", 61, 20.0, 0),
        ];
        let filtered = m.filtered_deals();
        let prices: Vec<f64> = filtered.iter().map(|d| d.price.amount).collect();
        assert_eq!(prices, vec![30.0, 20.0, 10.0]);
    }

    // --- SortCriteria pure helpers ---

    #[test]
    fn sort_criteria_next_cycles_through_all() {
        let start = SortCriteria::Price;
        let mut current = start;
        for _ in 0..6 {
            current = current.next();
        }
        assert_eq!(current, start);
    }

    #[test]
    fn sort_criteria_api_param_respects_direction() {
        assert_eq!(SortCriteria::Price.api_param(true), "price");
        assert_eq!(SortCriteria::Price.api_param(false), "-price");
        assert_eq!(SortCriteria::ReleaseDate.api_param(true), "release-date");
    }
}
