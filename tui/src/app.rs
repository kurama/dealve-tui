use dealve_api::ItadClient;
use dealve_core::models::{Deal, Platform};
use ratatui::widgets::ListState;

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
    pub show_platform_dropdown: bool,
    pub dropdown_selected: usize,
    client: ItadClient,
}


impl App {
    pub fn new(api_key: Option<String>) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            show_menu: false,
            menu_selected: 0,
            popup: Popup::None,
            deals: vec![],
            list_state,
            should_quit: false,
            loading: false,
            error: None,
            platform_filter: Platform::All,
            show_platform_dropdown: false,
            dropdown_selected: 0,
            client: ItadClient::new(api_key),
        }
    }

    pub async fn load_deals(&mut self) {
        self.loading = true;
        self.error = None;

        let shop_id = self.platform_filter.shop_id();

        match self.client.get_deals("US", 50, shop_id).await {
            Ok(deals) => {
                self.deals = deals;
                self.list_state.select(Some(0));
            }
            Err(e) => {
                self.error = Some(e.to_string());
            }
        }

        self.loading = false;
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
        match self.platform_filter.shop_id() {
            None => self.deals.iter().collect(),
            Some(shop_id) => self
                .deals
                .iter()
                .filter(|deal| deal.shop.id == shop_id.to_string())
                .collect(),
        }
    }

    pub fn toggle_dropdown(&mut self) {
        self.show_platform_dropdown = !self.show_platform_dropdown;
        if self.show_platform_dropdown {
            self.dropdown_selected = Platform::ALL
                .iter()
                .position(|&p| p == self.platform_filter)
                .unwrap_or(0);
        }
    }

    pub fn dropdown_next(&mut self) {
        self.dropdown_selected = (self.dropdown_selected + 1) % Platform::ALL.len();
    }

    pub fn dropdown_previous(&mut self) {
        if self.dropdown_selected == 0 {
            self.dropdown_selected = Platform::ALL.len() - 1;
        } else {
            self.dropdown_selected -= 1;
        }
    }

    pub async fn dropdown_select(&mut self) {
        self.platform_filter = Platform::ALL[self.dropdown_selected];
        self.show_platform_dropdown = false;
        self.load_deals().await;
    }

    pub fn platforms() -> &'static [Platform] {
        &Platform::ALL
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
    }
}
