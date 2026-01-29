use serde::{Deserialize, Serialize};

/// Represents a game deal from IsThereAnyDeal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deal {
    pub title: String,
    pub shop: Shop,
    pub price: Price,
    pub url: String,
}

/// Store/shop information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shop {
    pub id: String,
    pub name: String,
}

/// Price information with discount
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Price {
    pub amount: f64,
    pub currency: String,
    pub discount: u8,
}

/// Filter options for deal queries
#[derive(Debug, Clone, Default)]
pub struct DealFilter {
    pub shop_ids: Option<Vec<String>>,
    pub country: String,
    pub limit: usize,
}

impl Default for Shop {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
        }
    }
}

impl Default for Price {
    fn default() -> Self {
        Self {
            amount: 0.0,
            currency: "USD".to_string(),
            discount: 0,
        }
    }
}
