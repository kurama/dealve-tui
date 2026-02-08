use dealve_core::models::{Deal, PriceHistoryPoint};

pub enum Message {
    // Navigation
    SelectNext,
    SelectPrevious,
    OpenSelectedDeal,

    // Menu
    ToggleMenu,
    MenuNext,
    MenuPrevious,
    MenuSelect,

    // Filtering
    StartFilter,
    CancelFilter,
    ConfirmFilter,
    FilterPush(char),
    FilterPop,
    ClearFilters,

    // Price filter
    OpenPriceFilter,
    PriceFilterSwitchField,
    PriceFilterPush(char),
    PriceFilterPop,
    PriceFilterApply,
    PriceFilterClear,

    // Platform popup
    OpenPlatformPopup,
    PlatformPopupNext,
    PlatformPopupPrev,
    PlatformPopupSelect,

    // Sort
    ToggleSortDirection,
    NextSortCriteria,
    PrevSortCriteria,

    // Popups
    ClosePopup,

    // Options
    OptionsNextTab,
    OptionsPrevTab,
    OptionsNextItem,
    OptionsPrevItem,
    OptionsToggleItem,
    OptionsToggleSortDirection,

    // Data loading results
    RequestRefresh,
    DealsLoaded {
        deals: Vec<Deal>,
        is_more: bool,
        page_size: usize,
    },
    MoreDealsLoaded {
        deals: Vec<Deal>,
        is_more: bool,
        page_size: usize,
    },
    DealsLoadFailed(String),
    PriceHistoryLoaded {
        game_id: String,
        history: Vec<PriceHistoryPoint>,
    },

    // System
    Tick,
    Quit,
}
