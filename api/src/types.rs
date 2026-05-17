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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deal_item_converts_to_core_deal() {
        let item = DealItem {
            id: "game-1".to_string(),
            title: "A Game".to_string(),
            deal: DealInfo {
                shop: ShopInfo {
                    id: 61,
                    name: "Steam".to_string(),
                },
                price: PriceInfo {
                    amount: 9.99,
                    currency: "USD".to_string(),
                },
                regular: PriceInfo {
                    amount: 19.99,
                    currency: "USD".to_string(),
                },
                cut: 50,
                url: "https://example.com/deal".to_string(),
                history_low: Some(HistoryPrice { amount: 7.49 }),
            },
        };

        let deal = dealve_core::models::Deal::from(item);

        assert_eq!(deal.id, "game-1");
        assert_eq!(deal.title, "A Game");
        assert_eq!(deal.shop.id, "61");
        assert_eq!(deal.shop.name, "Steam");
        assert_eq!(deal.price.amount, 9.99);
        assert_eq!(deal.price.currency, "USD");
        assert_eq!(deal.price.discount, 50);
        assert_eq!(deal.regular_price, 19.99);
        assert_eq!(deal.url, "https://example.com/deal");
        assert_eq!(deal.history_low, Some(7.49));
    }

    #[test]
    fn game_info_response_defaults_missing_collections() {
        let response = GameInfoResponse {
            id: "game-2".to_string(),
            title: "Another Game".to_string(),
            release_date: None,
            developers: None,
            publishers: None,
            tags: None,
        };

        let info = dealve_core::models::GameInfo::from(response);

        assert_eq!(info.id, "game-2");
        assert_eq!(info.title, "Another Game");
        assert_eq!(info.release_date, None);
        assert!(info.developers.is_empty());
        assert!(info.publishers.is_empty());
        assert!(info.tags.is_empty());
    }

    #[test]
    fn game_info_response_extracts_company_names() {
        let response = GameInfoResponse {
            id: "game-3".to_string(),
            title: "Named Game".to_string(),
            release_date: Some("2026-05-17".to_string()),
            developers: Some(vec![CompanyInfo {
                name: "Dev Studio".to_string(),
            }]),
            publishers: Some(vec![CompanyInfo {
                name: "Publisher".to_string(),
            }]),
            tags: Some(vec!["RPG".to_string(), "Indie".to_string()]),
        };

        let info = dealve_core::models::GameInfo::from(response);

        assert_eq!(info.release_date, Some("2026-05-17".to_string()));
        assert_eq!(info.developers, vec!["Dev Studio"]);
        assert_eq!(info.publishers, vec!["Publisher"]);
        assert_eq!(info.tags, vec!["RPG", "Indie"]);
    }

    #[test]
    fn price_history_item_requires_deal() {
        let item = PriceHistoryItem {
            timestamp: "2026-05-17T00:00:00Z".to_string(),
            shop: ShopInfo {
                id: 61,
                name: "Steam".to_string(),
            },
            deal: None,
        };

        assert!(item.to_price_history_point().is_none());
    }

    #[test]
    fn price_history_item_converts_valid_timestamp() {
        let item = PriceHistoryItem {
            timestamp: "2026-05-17T00:00:00Z".to_string(),
            shop: ShopInfo {
                id: 61,
                name: "Steam".to_string(),
            },
            deal: Some(HistoryDeal {
                price: PriceInfo {
                    amount: 12.5,
                    currency: "USD".to_string(),
                },
            }),
        };

        let point = item.to_price_history_point().unwrap();

        assert_eq!(point.timestamp, 1_778_976_000);
        assert_eq!(point.price, 12.5);
        assert_eq!(point.shop_name, "Steam");
    }

    #[test]
    fn price_history_item_uses_zero_for_malformed_timestamp() {
        let item = PriceHistoryItem {
            timestamp: "not-a-date".to_string(),
            shop: ShopInfo {
                id: 61,
                name: "Steam".to_string(),
            },
            deal: Some(HistoryDeal {
                price: PriceInfo {
                    amount: 12.5,
                    currency: "USD".to_string(),
                },
            }),
        };

        let point = item.to_price_history_point().unwrap();

        assert_eq!(point.timestamp, 0);
        assert_eq!(point.price, 12.5);
        assert_eq!(point.shop_name, "Steam");
    }
}
