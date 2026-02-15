use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DealsResponse {
    pub list: Vec<DealItem>,
}

#[derive(Debug, Deserialize)]
pub struct GameSearchItem {
    pub id: String,
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct DealItem {
    pub id: String,
    pub title: String,
    pub deal: DealInfo,
}

#[derive(Debug, Deserialize)]
pub struct DealInfo {
    pub shop: ShopInfo,
    pub price: PriceInfo,
    pub regular: PriceInfo,
    pub cut: u8,
    pub url: String,
    #[serde(rename = "historyLow")]
    pub history_low: Option<HistoryPrice>,
}

#[derive(Debug, Deserialize)]
pub struct ShopInfo {
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct PriceInfo {
    pub amount: f64,
    pub currency: String,
}

#[derive(Debug, Deserialize)]
pub struct HistoryPrice {
    pub amount: f64,
}

#[derive(Debug, Deserialize)]
pub struct GamePriceHistory {
    pub all: Option<PriceInfo>,
}

#[derive(Debug, Deserialize)]
pub struct GamePriceItem {
    pub id: String,
    #[serde(rename = "historyLow")]
    pub history_low: Option<GamePriceHistory>,
    pub deals: Vec<DealInfo>,
}

impl From<DealItem> for dealve_core::models::Deal {
    fn from(item: DealItem) -> Self {
        Self {
            id: item.id,
            title: item.title,
            shop: dealve_core::models::Shop {
                id: item.deal.shop.id.to_string(),
                name: item.deal.shop.name,
            },
            price: dealve_core::models::Price {
                amount: item.deal.price.amount,
                currency: item.deal.price.currency,
                discount: item.deal.cut,
            },
            regular_price: item.deal.regular.amount,
            url: item.deal.url,
            history_low: item.deal.history_low.map(|h| h.amount),
        }
    }
}

// Game Info API response types
#[derive(Debug, Deserialize)]
pub struct GameInfoResponse {
    pub id: String,
    pub title: String,
    #[serde(rename = "releaseDate")]
    pub release_date: Option<String>,
    pub developers: Option<Vec<CompanyInfo>>,
    pub publishers: Option<Vec<CompanyInfo>>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct CompanyInfo {
    pub name: String,
}

impl From<GameInfoResponse> for dealve_core::models::GameInfo {
    fn from(resp: GameInfoResponse) -> Self {
        Self {
            id: resp.id,
            title: resp.title,
            release_date: resp.release_date,
            developers: resp
                .developers
                .map(|d| d.into_iter().map(|c| c.name).collect())
                .unwrap_or_default(),
            publishers: resp
                .publishers
                .map(|p| p.into_iter().map(|c| c.name).collect())
                .unwrap_or_default(),
            tags: resp.tags.unwrap_or_default(),
        }
    }
}

// Price History API response types
#[derive(Debug, Deserialize)]
pub struct PriceHistoryResponse(pub Vec<PriceHistoryItem>);

#[derive(Debug, Deserialize)]
pub struct PriceHistoryItem {
    pub timestamp: String,
    pub shop: ShopInfo,
    pub deal: Option<HistoryDeal>,
}

#[derive(Debug, Deserialize)]
pub struct HistoryDeal {
    pub price: PriceInfo,
}

impl PriceHistoryItem {
    /// Convert to core model, parsing the ISO timestamp to unix timestamp
    pub fn to_price_history_point(&self) -> Option<dealve_core::models::PriceHistoryPoint> {
        let deal = self.deal.as_ref()?;

        // Parse ISO 8601 timestamp to unix timestamp
        // Format: "2021-12-17T00:20:46+01:00"
        let timestamp = chrono::DateTime::parse_from_rfc3339(&self.timestamp)
            .map(|dt| dt.timestamp())
            .unwrap_or(0);

        Some(dealve_core::models::PriceHistoryPoint {
            timestamp,
            price: deal.price.amount,
            shop_name: self.shop.name.clone(),
        })
    }
}
