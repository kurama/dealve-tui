use dealve_api::ItadClient;
use dealve_core::models::{Deal, GameInfo, Platform, Region};
use ratatui::widgets::ListState;
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
}

impl OptionsTab {
    pub const ALL: &'static [OptionsTab] = &[
        OptionsTab::Region,
        OptionsTab::Platforms,
    ];

    pub fn name(&self) -> &str {
        match self {
            OptionsTab::Region => "Region",
            OptionsTab::Platforms => "Platforms",
        }
    }
}

/// State for the Options popup
pub struct OptionsState {
    pub current_tab: usize,
    pub platform_list_index: usize,
    pub region_list_index: usize,
    pub default_platform: Platform,
    pub enabled_platforms: HashSet<Platform>,
    pub region: Region,
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
            default_platform: Platform::All,
            enabled_platforms: enabled,
            region: Region::default(),
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
            default_platform,
            enabled_platforms,
            region,
        }
    }

    /// Save current state to config
    pub fn save_to_config(&self) {
        let mut config = Config::load();
        config.update_from_options(self.default_platform, &self.enabled_platforms, self.region);
        let _ = config.save(); // Ignore errors silently
    }
}

pub struct App {
    pub show_menu: bool,
    pub menu_selected: usize,
    pub popup: Popup,
    pub deals: Vec<Deal>,
    pub list_state: ListState,
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
    client: ItadClient,
}


impl App {
    pub fn new(api_key: Option<String>) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

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
            client: ItadClient::new(api_key),
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn next(&mut self) {
        let filtered_count = self.filtered_deals().len();
        if filtered_count > 0 {
            let i = match self.list_state.selected() {
                Some(i) => {
                    if i >= filtered_count - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            self.list_state.select(Some(i));
        }
    }

    pub fn previous(&mut self) {
        let filtered_count = self.filtered_deals().len();
        if filtered_count > 0 {
            let i = match self.list_state.selected() {
                Some(i) => {
                    if i == 0 {
                        filtered_count - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.list_state.select(Some(i));
        }
    }

    pub fn open_selected_deal(&self) {
        if let Some(i) = self.list_state.selected() {
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
        self.list_state.select(Some(0));
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
                self.list_state.select(Some(0));
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
    }

    // Options navigation
    pub fn options_next_tab(&mut self) {
        self.options.current_tab = (self.options.current_tab + 1) % OptionsTab::ALL.len();
        self.options.platform_list_index = 0;
        self.options.region_list_index = 0;
    }

    pub fn options_prev_tab(&mut self) {
        if self.options.current_tab == 0 {
            self.options.current_tab = OptionsTab::ALL.len() - 1;
        } else {
            self.options.current_tab -= 1;
        }
        self.options.platform_list_index = 0;
        self.options.region_list_index = 0;
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
        self.list_state
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
