use dealve_api::ItadClient;
use dealve_core::models::Deal;

pub struct App {
    pub deals: Vec<Deal>,
    pub selected_index: usize,
    pub should_quit: bool,
    pub loading: bool,
    pub error: Option<String>,
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
}
