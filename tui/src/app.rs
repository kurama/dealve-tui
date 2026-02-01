use dealve_api::ItadClient;
use dealve_core::models::{Deal, Platform};
use ratatui::widgets::ListState;

pub struct App {
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

        match self.client.get_deals("US", 50).await {
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

    pub fn dropdown_select(&mut self) {
        self.platform_filter = Platform::ALL[self.dropdown_selected];
        self.show_platform_dropdown = false;
        self.list_state.select(Some(0));
    }

    pub fn platforms() -> &'static [Platform] {
        &Platform::ALL
    }
}
