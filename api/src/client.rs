use reqwest::Client;

const API_BASE_URL: &str = "https://api.isthereanydeal.com";

pub struct ItadClient {
    client: Client,
    api_key: Option<String>,
}

impl ItadClient {
    pub fn new(api_key: Option<String>) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    pub fn base_url(&self) -> &str {
        API_BASE_URL
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn api_key(&self) -> Option<&str> {
        self.api_key.as_deref()
    }
}
