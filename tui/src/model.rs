use dealve_core::models::{Deal, GameInfo, Platform, PriceHistoryPoint, Region};
use ratatui::widgets::{ListState, TableState};
use std::collections::{HashMap, HashSet};

use crate::config::Config;

// ── Enums ───────────────────────────────────────────────────────────────────

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
    PriceFilter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortCriteria {
    #[default]
    Price,
    Cut,
    Hottest,
    ReleaseDate,
    Expiring,
    Popular,
}

impl SortCriteria {
    pub fn name(&self) -> &str {
        match self {
            SortCriteria::Price => "Price",
            SortCriteria::Cut => "Cut",
            SortCriteria::Hottest => "Hottest",
            SortCriteria::ReleaseDate => "Release",
            SortCriteria::Expiring => "Expiring",
            SortCriteria::Popular => "Popular",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            SortCriteria::Price => SortCriteria::Cut,
            SortCriteria::Cut => SortCriteria::Hottest,
            SortCriteria::Hottest => SortCriteria::ReleaseDate,
            SortCriteria::ReleaseDate => SortCriteria::Expiring,
            SortCriteria::Expiring => SortCriteria::Popular,
            SortCriteria::Popular => SortCriteria::Price,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            SortCriteria::Price => SortCriteria::Popular,
            SortCriteria::Cut => SortCriteria::Price,
            SortCriteria::Hottest => SortCriteria::Cut,
            SortCriteria::ReleaseDate => SortCriteria::Hottest,
            SortCriteria::Expiring => SortCriteria::ReleaseDate,
            SortCriteria::Popular => SortCriteria::Expiring,
        }
    }

    pub fn toggle_search(&self) -> Self {
        match self {
            SortCriteria::Price => SortCriteria::Cut,
            _ => SortCriteria::Price,
        }
    }

    pub fn api_param(&self, ascending: bool) -> String {
        let base = match self {
            SortCriteria::Price => "price",
            SortCriteria::Cut => "cut",
            SortCriteria::Hottest => "hot",
            SortCriteria::ReleaseDate => "release-date",
            SortCriteria::Expiring => "expiry",
            SortCriteria::Popular => "rank",
        };
        if ascending {
            base.to_string()
        } else {
            format!("-{}", base)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortDirection {
    #[default]
    Ascending,
    Descending,
}

impl SortDirection {
    pub fn toggle(&self) -> Self {
        match self {
            SortDirection::Ascending => SortDirection::Descending,
            SortDirection::Descending => SortDirection::Ascending,
        }
    }

    pub fn arrow(&self) -> &str {
        match self {
            SortDirection::Ascending => "↑",
            SortDirection::Descending => "↓",
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SortState {
    pub criteria: SortCriteria,
    pub direction: SortDirection,
}

impl SortState {
    pub fn api_param(&self) -> String {
        self.criteria
            .api_param(self.direction == SortDirection::Ascending)
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

// ── Sub-states ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct PriceFilterState {
    pub min_input: String,
    pub max_input: String,
    pub selected_field: usize, // 0 = min, 1 = max
    pub active_min: Option<f64>,
    pub active_max: Option<f64>,
}

impl PriceFilterState {
    pub fn clear(&mut self) {
        self.min_input.clear();
        self.max_input.clear();
        self.active_min = None;
        self.active_max = None;
    }

    pub fn apply(&mut self) {
        self.active_min = self.min_input.parse().ok();
        self.active_max = self.max_input.parse().ok();
    }

    pub fn is_active(&self) -> bool {
        self.active_min.is_some() || self.active_max.is_some()
    }

    pub fn label(&self) -> String {
        match (self.active_min, self.active_max) {
            (Some(min), Some(max)) => format!("{:.0}-{:.0}", min, max),
            (Some(min), None) => format!(">{:.0}", min),
            (None, Some(max)) => format!("<{:.0}", max),
            (None, None) => "—".to_string(),
        }
    }

    pub fn matches(&self, price: f64) -> bool {
        if let Some(min) = self.active_min {
            if price < min {
                return false;
            }
        }
        if let Some(max) = self.active_max {
            if price > max {
                return false;
            }
        }
        true
    }
}

pub struct OptionsState {
    pub current_tab: usize,
    pub platform_list_index: usize,
    pub region_list_index: usize,
    pub advanced_list_index: usize,
    pub default_platform: Platform,
    pub enabled_platforms: HashSet<Platform>,
    pub region: Region,
    pub deals_page_size: usize,
    pub game_info_delay_ms: u64,
    pub default_sort: SortState,
}

impl Default for OptionsState {
    fn default() -> Self {
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
            default_sort: SortState::default(),
        }
    }
}

impl OptionsState {
    pub fn from_config(config: &Config) -> Self {
        let enabled_platforms = config.get_enabled_platforms();
        let default_platform = config.get_default_platform();
        let region = config.get_region();
        let default_sort = config.get_default_sort();

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
            default_sort,
        }
    }

    pub fn save_to_config(&self) {
        let mut config = Config::load();
        config.update_from_options(
            self.default_platform,
            &self.enabled_platforms,
            self.region,
            self.default_sort,
        );
        config.deals_page_size = self.deals_page_size;
        config.game_info_delay_ms = self.game_info_delay_ms;
        let _ = config.save();
    }
}

#[derive(Default)]
pub struct FilterState {
    pub active: bool,
    pub text: String,
}

pub struct PaginationState {
    pub offset: usize,
    pub has_more: bool,
    pub loading_more: bool,
}

impl Default for PaginationState {
    fn default() -> Self {
        Self {
            offset: 0,
            has_more: true,
            loading_more: false,
        }
    }
}

#[derive(Default)]
pub struct LoadingState {
    pub deals: bool,
    pub game_info: Option<String>,
    pub price_history: Option<String>,
}

pub struct UiState {
    pub show_menu: bool,
    pub menu_selected: usize,
    pub popup: Popup,
    pub table_state: TableState,
    pub list_state: ListState,
    pub spinner_frame: usize,
    pub platform_popup_index: usize,
}

impl Default for UiState {
    fn default() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        let mut table_state = TableState::default();
        table_state.select(Some(0));
        Self {
            show_menu: false,
            menu_selected: 0,
            popup: Popup::None,
            table_state,
            list_state,
            spinner_frame: 0,
            platform_popup_index: 0,
        }
    }
}

// ── Model ───────────────────────────────────────────────────────────────────

pub struct Model {
    // Data
    pub deals: Vec<Deal>,
    pub game_info_cache: HashMap<String, GameInfo>,
    pub price_history_cache: HashMap<String, Vec<PriceHistoryPoint>>,

    // UI
    pub ui: UiState,

    // Filters
    pub filter: FilterState,
    pub active_search_query: Option<String>,
    pub price_filter: PriceFilterState,

    // Sort
    pub sort_state: SortState,

    // Platform & Region
    pub platform_filter: Platform,
    pub region: Region,

    // Pagination
    pub pagination: PaginationState,

    // Loading
    pub loading: LoadingState,

    // Options
    pub options: OptionsState,

    // Config
    pub api_key: Option<String>,
    pub deals_page_size: usize,
    pub game_info_delay_ms: u64,

    // Error
    pub error: Option<String>,

    // Control
    pub should_quit: bool,
}

impl Model {
    pub fn new(api_key: Option<String>) -> Self {
        let config = Config::load();
        let options = OptionsState::from_config(&config);
        let platform_filter = options.default_platform;
        let region = options.region;
        let sort_state = options.default_sort;

        Self {
            deals: vec![],
            game_info_cache: HashMap::new(),
            price_history_cache: HashMap::new(),
            ui: UiState::default(),
            filter: FilterState::default(),
            active_search_query: None,
            price_filter: PriceFilterState::default(),
            sort_state,
            platform_filter,
            region,
            pagination: PaginationState::default(),
            loading: LoadingState::default(),
            options,
            api_key,
            deals_page_size: config.deals_page_size,
            game_info_delay_ms: config.game_info_delay_ms,
            error: None,
            should_quit: false,
        }
    }

    pub fn error_clear(&mut self) {
        self.error = None;
    }

    pub fn set_loading(&mut self, loading: bool) {
        self.loading.deals = loading;
        if loading {
            self.ui.spinner_frame = 0;
        }
    }

    pub fn reset_pagination(&mut self) {
        self.deals.clear();
        self.pagination.offset = 0;
        self.pagination.has_more = true;
        self.pagination.loading_more = false;
        self.select(Some(0));
    }

    // ── Query methods ───────────────────────────────────────────────────

    pub fn filtered_deals(&self) -> Vec<&Deal> {
        let mut deals: Vec<&Deal> = match self.platform_filter.shop_id() {
            None => self.deals.iter().collect(),
            Some(shop_id) => self
                .deals
                .iter()
                .filter(|deal| deal.shop.id == shop_id.to_string())
                .collect(),
        };

        // Apply price filter
        if self.price_filter.is_active() {
            deals.retain(|deal| self.price_filter.matches(deal.price.amount));
        }

        if self.is_search_mode() {
            self.sort_search_results(&mut deals);
        }

        deals
    }

    pub fn is_search_mode(&self) -> bool {
        self.active_search_query.is_some()
    }

    pub fn is_search_sort_supported(&self) -> bool {
        matches!(
            self.sort_state.criteria,
            SortCriteria::Price | SortCriteria::Cut
        )
    }

    pub fn selected_deal(&self) -> Option<&Deal> {
        self.ui
            .table_state
            .selected()
            .and_then(|i| self.filtered_deals().get(i).copied())
    }

    pub fn selected_game_info(&self) -> Option<&GameInfo> {
        self.selected_deal()
            .and_then(|deal| self.game_info_cache.get(&deal.id))
    }

    pub fn selected_price_history(&self) -> Option<&Vec<PriceHistoryPoint>> {
        let deal = self.selected_deal()?;
        self.price_history_cache.get(&deal.id)
    }

    pub fn enabled_platforms(&self) -> Vec<Platform> {
        Platform::ALL
            .iter()
            .copied()
            .filter(|p| self.options.enabled_platforms.contains(p))
            .collect()
    }

    pub fn should_load_more(&self) -> bool {
        !self.loading.deals && !self.pagination.loading_more && self.pagination.has_more
    }

    pub fn needs_game_info_load(&self) -> Option<String> {
        if let Some(deal) = self.selected_deal() {
            if !self.game_info_cache.contains_key(&deal.id)
                && self.loading.game_info.as_ref() != Some(&deal.id)
            {
                return Some(deal.id.clone());
            }
        }
        None
    }

    pub fn needs_price_history_load(&self) -> Option<String> {
        let deal = self.selected_deal()?;
        if self.price_history_cache.contains_key(&deal.id) {
            return None;
        }
        if self.loading.price_history.as_ref() == Some(&deal.id) {
            return None;
        }
        Some(deal.id.clone())
    }

    pub fn spinner_char(&self) -> char {
        const SPINNER_FRAMES: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
        SPINNER_FRAMES[self.ui.spinner_frame]
    }

    /// Sync both list_state and table_state selection
    pub fn select(&mut self, index: Option<usize>) {
        self.ui.list_state.select(index);
        self.ui.table_state.select(index);
    }

    /// Get platforms without "All" (for the checkbox list in options)
    pub fn platforms_without_all() -> Vec<Platform> {
        Platform::ALL
            .iter()
            .copied()
            .filter(|p| *p != Platform::All)
            .collect()
    }

    fn sort_search_results(&self, deals: &mut Vec<&Deal>) {
        match self.sort_state.criteria {
            SortCriteria::Price => deals.sort_by(|a, b| a.price.amount.total_cmp(&b.price.amount)),
            SortCriteria::Cut => deals.sort_by_key(|deal| deal.price.discount),
            _ => return,
        }

        if self.sort_state.direction == SortDirection::Descending {
            deals.reverse();
        }
    }
}
