use dealve_api::ItadClient;
use dealve_core::models::{Deal, Platform};

pub struct App {
    pub deals: Vec<Deal>,
    pub selected_index: usize,
    pub should_quit: bool,
    pub loading: bool,
    pub error: Option<String>,
    pub platform_filter: Platform,
    client: ItadClient,
}

impl App {
    pub fn new(api_key: Option<String>) -> Self {
        Self {
            deals: vec![],
            selected_index: 0,
            should_quit: false,
            loading: false,
            error: None,
            platform_filter: Platform::All,
            client: ItadClient::new(api_key),
        }
    }

    pub async fn load_deals(&mut self) {
        self.loading = true;
        self.error = None;

        match self.client.get_deals("US", 20).await {
            Ok(deals) => {
                self.deals = deals;
                self.selected_index = 0;
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
        if !self.deals.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.deals.len();
        }
    }

    pub fn previous(&mut self) {
        if !self.deals.is_empty() {
            if self.selected_index > 0 {
                self.selected_index -= 1;
            } else {
                self.selected_index = self.deals.len() - 1;
            }
        }
    }

    pub fn open_selected_deal(&self) {
        if let Some(deal) = self.deals.get(self.selected_index) {
            let _ = webbrowser::open(&deal.url);
        }
    }

    pub fn next_platform(&mut self) {
        self.platform_filter = self.platform_filter.next();
        self.selected_index = 0;
    }

    pub fn previous_platform(&mut self) {
        self.platform_filter = self.platform_filter.previous();
        self.selected_index = 0;
    }

    pub fn filtered_deals(&self) -> Vec<&Deal> {
        match self.platform_filter.shop_ids() {
            None => self.deals.iter().collect(),
            Some(shop_ids) => self
                .deals
                .iter()
                .filter(|deal| shop_ids.iter().any(|&id| deal.shop.id == id))
                .collect(),
        }
    }
}
