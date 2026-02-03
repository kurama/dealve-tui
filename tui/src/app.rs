use dealve_api::ItadClient;
use dealve_core::models::{Deal, GameInfo, Platform, Region};
use ratatui::widgets::{ListState, TableState};
use std::collections::{HashMap, HashSet};

use crate::config::Config;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuItem {
    Browse,
    Options,
    Keybinds,
    Quit,
}

impl MenuItem {
    pub const ALL: &'static [MenuItem] = &[
        MenuItem::Browse,
        MenuItem::Options,
        MenuItem::Keybinds,
        MenuItem::Quit,
    ];

    pub fn name(&self) -> &str {
        match self {
            MenuItem::Browse => "BROWSE DEALS",
            MenuItem::Options => "OPTIONS",
            MenuItem::Keybinds => "KEYBINDS",
            MenuItem::Quit => "QUIT",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Popup {
    None,
    Options,
    Keybinds,
    Platform,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortOrder {
    #[default]
    None,       // Default API order
    PriceAsc,   // Cheapest first
    PriceDesc,  // Most expensive first
}

impl SortOrder {
    pub fn next(&self) -> Self {
        match self {
            SortOrder::None => SortOrder::PriceAsc,
            SortOrder::PriceAsc => SortOrder::PriceDesc,
            SortOrder::PriceDesc => SortOrder::None,
        }
    }

    pub fn label(&self) -> Option<&str> {
        match self {
            SortOrder::None => None,
            SortOrder::PriceAsc => Some("Price ↑"),
            SortOrder::PriceDesc => Some("Price ↓"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionsTab {
    Region,
    Platforms,
    Advanced,
}

impl OptionsTab {
    pub const ALL: &'static [OptionsTab] = &[
        OptionsTab::Region,
        OptionsTab::Platforms,
        OptionsTab::Advanced,
    ];

    pub fn name(&self) -> &str {
        match self {
            OptionsTab::Region => "Region",
            OptionsTab::Platforms => "Platforms",
            OptionsTab::Advanced => "Advanced",
        }
    }
}

/// State for the Options popup
pub struct OptionsState {
    pub current_tab: usize,
    pub platform_list_index: usize,
    pub region_list_index: usize,
    pub advanced_list_index: usize,
    pub default_platform: Platform,
    pub enabled_platforms: HashSet<Platform>,
    pub region: Region,
    // Advanced settings
    pub deals_page_size: usize,
    pub game_info_delay_ms: u64,
}

impl Default for OptionsState {
    fn default() -> Self {
        // All platforms enabled by default
        let mut enabled = HashSet::new();
        for platform in Platform::ALL.iter() {
            enabled.insert(*platform);
        }
        Self {
            current_tab: 0,
            platform_list_index: 0,
            region_list_index: 0,
            advanced_list_index: 0,
            default_platform: Platform::All,
            enabled_platforms: enabled,
            region: Region::default(),
            deals_page_size: 50,
            game_info_delay_ms: 200,
        }
    }
}

impl OptionsState {
    /// Create OptionsState from saved config
    pub fn from_config(config: &Config) -> Self {
        let enabled_platforms = config.get_enabled_platforms();
        let default_platform = config.get_default_platform();
        let region = config.get_region();

        // Ensure default platform is enabled
        let default_platform = if enabled_platforms.contains(&default_platform) {
            default_platform
        } else {
            Platform::All
        };

        Self {
            current_tab: 0,
            platform_list_index: 0,
            region_list_index: 0,
            advanced_list_index: 0,
            default_platform,
            enabled_platforms,
            region,
            deals_page_size: config.deals_page_size,
            game_info_delay_ms: config.game_info_delay_ms,
        }
    }

    /// Save current state to config
    pub fn save_to_config(&self) {
        let mut config = Config::load();
        config.update_from_options(self.default_platform, &self.enabled_platforms, self.region);
        config.deals_page_size = self.deals_page_size;
        config.game_info_delay_ms = self.game_info_delay_ms;
        let _ = config.save(); // Ignore errors silently
    }
}

pub struct App {
    pub show_menu: bool,
    pub menu_selected: usize,
    pub popup: Popup,
    pub deals: Vec<Deal>,
    pub list_state: ListState,
    pub table_state: TableState,
    pub should_quit: bool,
    pub loading: bool,
    pub error: Option<String>,
    pub platform_filter: Platform,
    pub region: Region,
    // Game info cache and loading state
    pub game_info_cache: HashMap<String, GameInfo>,
    pub loading_game_info: Option<String>,
    // Options state
    pub options: OptionsState,
    // Animation frame counter for spinner
    pub spinner_frame: usize,
    // Sort order for deals
    pub sort_order: SortOrder,
    // Platform popup selection
    pub platform_popup_index: usize,
    // Pagination state
    pub deals_offset: usize,
    pub has_more_deals: bool,
    pub loading_more: bool,
    // Configurable parameters
    pub deals_page_size: usize,
    pub game_info_delay_ms: u64,
    // Filter state
    pub filter_active: bool,
    pub filter_text: String,
    // API client
    pub api_key: Option<String>,
    client: ItadClient,
}


impl App {
    pub fn new(api_key: Option<String>) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        let mut table_state = TableState::default();
        table_state.select(Some(0));

        // Load config from disk
        let config = Config::load();
        let options = OptionsState::from_config(&config);
        let platform_filter = options.default_platform;
        let region = options.region;

        Self {
            show_menu: false,
            menu_selected: 0,
            popup: Popup::None,
            deals: vec![],
            list_state,
            table_state,
            should_quit: false,
            loading: false,
            error: None,
            platform_filter,
            region,
            game_info_cache: HashMap::new(),
            loading_game_info: None,
            options,
            spinner_frame: 0,
            sort_order: SortOrder::default(),
            platform_popup_index: 0,
            deals_offset: 0,
            has_more_deals: true,
            loading_more: false,
            deals_page_size: config.deals_page_size,
            game_info_delay_ms: config.game_info_delay_ms,
            filter_active: false,
            filter_text: String::new(),
            api_key: api_key.clone(),
            client: ItadClient::new(api_key),
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Select an item by index (syncs both list_state and table_state)
    fn select(&mut self, index: Option<usize>) {
        self.list_state.select(index);
        self.table_state.select(index);
    }

    pub fn next(&mut self) {
        let filtered_count = self.filtered_deals().len();
        if filtered_count > 0 {
            let i = match self.table_state.selected() {
                Some(i) => {
                    if i >= filtered_count - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            self.select(Some(i));
        }
    }

    pub fn previous(&mut self) {
        let filtered_count = self.filtered_deals().len();
        if filtered_count > 0 {
            let i = match self.table_state.selected() {
                Some(i) => {
                    if i == 0 {
                        filtered_count - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.select(Some(i));
        }
    }

    pub fn open_selected_deal(&self) {
        if let Some(i) = self.table_state.selected() {
            let filtered = self.filtered_deals();
            if let Some(deal) = filtered.get(i) {
                let _ = webbrowser::open(&deal.url);
            }
        }
    }

    pub fn filtered_deals(&self) -> Vec<&Deal> {
        let mut deals: Vec<&Deal> = match self.platform_filter.shop_id() {
            None => self.deals.iter().collect(),
            Some(shop_id) => self
                .deals
                .iter()
                .filter(|deal| deal.shop.id == shop_id.to_string())
                .collect(),
        };

        // Apply name filter only when confirmed (not during active typing)
        if !self.filter_active && !self.filter_text.is_empty() {
            let filter_lower = self.filter_text.to_lowercase();
            deals.retain(|deal| deal.title.to_lowercase().contains(&filter_lower));
        }

        // Sort based on current sort order
        match self.sort_order {
            SortOrder::None => {} // Keep API order
            SortOrder::PriceAsc => {
                deals.sort_by(|a, b| a.price.amount.partial_cmp(&b.price.amount).unwrap_or(std::cmp::Ordering::Equal));
            }
            SortOrder::PriceDesc => {
                deals.sort_by(|a, b| b.price.amount.partial_cmp(&a.price.amount).unwrap_or(std::cmp::Ordering::Equal));
            }
        }

        deals
    }

    pub fn cycle_sort_order(&mut self) {
        self.sort_order = self.sort_order.next();
        // Reset selection to first item when changing sort
        self.select(Some(0));
    }

    /// Activate filter input mode
    pub fn start_filter(&mut self) {
        self.filter_active = true;
        self.filter_text.clear();
    }

    /// Deactivate filter and clear text
    pub fn cancel_filter(&mut self) {
        self.filter_active = false;
        self.filter_text.clear();
        self.select(Some(0));
    }

    /// Confirm filter (just deactivate input mode, keep filter text)
    pub fn confirm_filter(&mut self) {
        self.filter_active = false;
        self.select(Some(0));
    }

    /// Add character to filter text
    pub fn filter_push(&mut self, c: char) {
        self.filter_text.push(c);
        self.select(Some(0));
    }

    /// Remove last character from filter text
    pub fn filter_pop(&mut self) {
        self.filter_text.pop();
        self.select(Some(0));
    }

    /// Clear the filter completely
    pub fn clear_filter(&mut self) {
        self.filter_text.clear();
        self.filter_active = false;
        self.select(Some(0));
    }

    /// Open platform selection popup
    pub fn open_platform_popup(&mut self) {
        let enabled = self.enabled_platforms();
        // Find current platform index in enabled list
        self.platform_popup_index = enabled
            .iter()
            .position(|&p| p == self.platform_filter)
            .unwrap_or(0);
        self.popup = Popup::Platform;
    }

    /// Navigate to next platform in popup
    pub fn platform_popup_next(&mut self) {
        let enabled = self.enabled_platforms();
        if !enabled.is_empty() {
            self.platform_popup_index = (self.platform_popup_index + 1) % enabled.len();
        }
    }

    /// Navigate to previous platform in popup
    pub fn platform_popup_prev(&mut self) {
        let enabled = self.enabled_platforms();
        if !enabled.is_empty() {
            if self.platform_popup_index == 0 {
                self.platform_popup_index = enabled.len() - 1;
            } else {
                self.platform_popup_index -= 1;
            }
        }
    }

    /// Select platform from popup and close it
    /// Returns true if a different platform was selected (needs reload)
    pub fn platform_popup_select(&mut self) -> bool {
        let enabled = self.enabled_platforms();
        if let Some(&platform) = enabled.get(self.platform_popup_index) {
            let changed = self.platform_filter != platform;
            self.platform_filter = platform;
            self.popup = Popup::None;
            if changed {
                self.select(Some(0));
            }
            changed
        } else {
            self.popup = Popup::None;
            false
        }
    }

    pub fn set_loading(&mut self, loading: bool) {
        self.loading = loading;
        if loading {
            self.spinner_frame = 0;
        }
    }

    /// Reset pagination state (called when changing filters)
    pub fn reset_pagination(&mut self) {
        self.deals.clear();
        self.deals_offset = 0;
        self.has_more_deals = true;
        self.select(Some(0));
    }

    /// Check if we're near the end of the list and should load more
    pub fn should_load_more(&self) -> bool {
        // Always load more in background if available (transparent loading)
        !self.loading && !self.loading_more && self.has_more_deals
    }

    pub fn tick_spinner(&mut self) {
        self.spinner_frame = (self.spinner_frame + 1) % 10;
    }

    pub fn spinner_char(&self) -> char {
        const SPINNER_FRAMES: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
        SPINNER_FRAMES[self.spinner_frame]
    }

    pub fn toggle_menu(&mut self) {
        self.show_menu = !self.show_menu;
        if self.show_menu {
            self.menu_selected = 0;
        }
    }

    pub fn menu_next(&mut self) {
        self.menu_selected = (self.menu_selected + 1) % MenuItem::ALL.len();
    }

    pub fn menu_previous(&mut self) {
        if self.menu_selected == 0 {
            self.menu_selected = MenuItem::ALL.len() - 1;
        } else {
            self.menu_selected -= 1;
        }
    }

    pub async fn menu_select(&mut self) {
        match MenuItem::ALL[self.menu_selected] {
            MenuItem::Browse => {
                self.show_menu = false;
            }
            MenuItem::Options => {
                self.popup = Popup::Options;
            }
            MenuItem::Keybinds => {
                self.popup = Popup::Keybinds;
            }
            MenuItem::Quit => self.quit(),
        }
    }

    pub fn close_popup(&mut self) {
        self.popup = Popup::None;
        // Reset options navigation when closing
        self.options.platform_list_index = 0;
        self.options.region_list_index = 0;
        self.options.advanced_list_index = 0;
    }

    // Options navigation
    pub fn options_next_tab(&mut self) {
        self.options.current_tab = (self.options.current_tab + 1) % OptionsTab::ALL.len();
        self.options.platform_list_index = 0;
        self.options.region_list_index = 0;
        self.options.advanced_list_index = 0;
    }

    pub fn options_prev_tab(&mut self) {
        if self.options.current_tab == 0 {
            self.options.current_tab = OptionsTab::ALL.len() - 1;
        } else {
            self.options.current_tab -= 1;
        }
        self.options.platform_list_index = 0;
        self.options.region_list_index = 0;
        self.options.advanced_list_index = 0;
    }

    /// Get platforms without "All" (for the checkbox list)
    fn platforms_without_all() -> Vec<Platform> {
        Platform::ALL
            .iter()
            .copied()
            .filter(|p| *p != Platform::All)
            .collect()
    }

    pub fn options_next_item(&mut self) {
        match OptionsTab::ALL[self.options.current_tab] {
            OptionsTab::Region => {
                self.options.region_list_index = (self.options.region_list_index + 1) % Region::ALL.len();
            }
            OptionsTab::Platforms => {
                // +1 for the "Default Platform" option at the top, rest are platforms without All
                let total_items = 1 + Self::platforms_without_all().len();
                self.options.platform_list_index = (self.options.platform_list_index + 1) % total_items;
            }
            OptionsTab::Advanced => {
                // 3 settings: page size, load threshold, game info delay
                self.options.advanced_list_index = (self.options.advanced_list_index + 1) % 3;
            }
        }
    }

    pub fn options_prev_item(&mut self) {
        match OptionsTab::ALL[self.options.current_tab] {
            OptionsTab::Region => {
                if self.options.region_list_index == 0 {
                    self.options.region_list_index = Region::ALL.len() - 1;
                } else {
                    self.options.region_list_index -= 1;
                }
            }
            OptionsTab::Platforms => {
                let total_items = 1 + Self::platforms_without_all().len();
                if self.options.platform_list_index == 0 {
                    self.options.platform_list_index = total_items - 1;
                } else {
                    self.options.platform_list_index -= 1;
                }
            }
            OptionsTab::Advanced => {
                if self.options.advanced_list_index == 0 {
                    self.options.advanced_list_index = 2; // 3 items (0, 1, 2)
                } else {
                    self.options.advanced_list_index -= 1;
                }
            }
        }
    }

    pub fn options_toggle_item(&mut self) -> bool {
        let mut needs_reload = false;
        match OptionsTab::ALL[self.options.current_tab] {
            OptionsTab::Region => {
                // Select the region
                if let Some(&region) = Region::ALL.get(self.options.region_list_index) {
                    if self.options.region != region {
                        self.options.region = region;
                        self.region = region;
                        needs_reload = true;
                    }
                }
                self.options.save_to_config();
            }
            OptionsTab::Platforms => {
                if self.options.platform_list_index == 0 {
                    // Cycle through enabled platforms for default
                    self.cycle_default_platform();
                } else {
                    // Toggle platform enabled/disabled (list without All)
                    let platforms = Self::platforms_without_all();
                    let platform_idx = self.options.platform_list_index - 1;
                    if let Some(&platform) = platforms.get(platform_idx) {
                        if self.options.enabled_platforms.contains(&platform) {
                            self.options.enabled_platforms.remove(&platform);
                        } else {
                            self.options.enabled_platforms.insert(platform);
                        }
                    }
                }
                // Save after any change
                self.options.save_to_config();
            }
            OptionsTab::Advanced => {
                // Cycle through preset values for each setting
                match self.options.advanced_list_index {
                    0 => {
                        // Page size: cycle through 25, 50, 100, 200
                        self.options.deals_page_size = match self.options.deals_page_size {
                            25 => 50,
                            50 => 100,
                            100 => 200,
                            _ => 25,
                        };
                        self.deals_page_size = self.options.deals_page_size;
                    }
                    1 => {
                        // Game info delay: cycle through 100, 200, 300, 500
                        self.options.game_info_delay_ms = match self.options.game_info_delay_ms {
                            100 => 200,
                            200 => 300,
                            300 => 500,
                            _ => 100,
                        };
                        self.game_info_delay_ms = self.options.game_info_delay_ms;
                    }
                    _ => {}
                }
                self.options.save_to_config();
            }
        }
        needs_reload
    }

    fn cycle_default_platform(&mut self) {
        // Find current index
        let current_idx = Platform::ALL
            .iter()
            .position(|&p| p == self.options.default_platform)
            .unwrap_or(0);

        // Find next enabled platform
        let len = Platform::ALL.len();
        for i in 1..=len {
            let next_idx = (current_idx + i) % len;
            let next_platform = Platform::ALL[next_idx];
            if self.options.enabled_platforms.contains(&next_platform) {
                self.options.default_platform = next_platform;
                self.platform_filter = next_platform;
                // Note: save_to_config is called by options_toggle_item after this
                return;
            }
        }
    }

    pub fn enabled_platforms(&self) -> Vec<Platform> {
        Platform::ALL
            .iter()
            .copied()
            .filter(|p| self.options.enabled_platforms.contains(p))
            .collect()
    }

    pub fn selected_deal(&self) -> Option<&Deal> {
        self.table_state
            .selected()
            .and_then(|i| self.filtered_deals().get(i).copied())
    }

    pub fn selected_game_info(&self) -> Option<&GameInfo> {
        self.selected_deal()
            .and_then(|deal| self.game_info_cache.get(&deal.id))
    }

    pub fn needs_game_info_load(&self) -> Option<String> {
        // Check if we need to load game info for current selection
        if let Some(deal) = self.selected_deal() {
            if !self.game_info_cache.contains_key(&deal.id)
                && self.loading_game_info.as_ref() != Some(&deal.id)
            {
                return Some(deal.id.clone());
            }
        }
        None
    }

    pub fn start_loading_game_info(&mut self, game_id: String) {
        self.loading_game_info = Some(game_id);
    }

    pub fn finish_loading_game_info(&mut self, game_id: String, info: Option<GameInfo>) {
        if let Some(info) = info {
            self.game_info_cache.insert(game_id.clone(), info);
        }
        if self.loading_game_info.as_ref() == Some(&game_id) {
            self.loading_game_info = None;
        }
    }

    pub async fn load_game_info_for_selected(&mut self) {
        if let Some(game_id) = self.needs_game_info_load() {
            self.start_loading_game_info(game_id.clone());
            match self.client.get_game_info(&game_id).await {
                Ok(info) => {
                    self.finish_loading_game_info(game_id, Some(info));
                }
                Err(_) => {
                    self.finish_loading_game_info(game_id, None);
                }
            }
        }
    }
}
