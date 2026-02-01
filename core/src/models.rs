use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    All,
    Steam,
    EpicGames,
    Gog,
    HumbleStore,
}

impl Platform {
    pub fn name(&self) -> &str {
        match self {
            Platform::All => "All Platforms",
            Platform::Steam => "Steam",
            Platform::EpicGames => "Epic Games",
            Platform::Gog => "GOG",
            Platform::HumbleStore => "Humble Store",
        }
    }

    pub fn shop_ids(&self) -> Option<Vec<&str>> {
        match self {
            Platform::All => None,
            Platform::Steam => Some(vec!["steam"]),
            Platform::EpicGames => Some(vec!["epic"]),
            Platform::Gog => Some(vec!["gog"]),
            Platform::HumbleStore => Some(vec!["humblestore"]),
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Platform::All => Platform::Steam,
            Platform::Steam => Platform::EpicGames,
            Platform::EpicGames => Platform::Gog,
            Platform::Gog => Platform::HumbleStore,
            Platform::HumbleStore => Platform::All,
        }
    }

    pub fn previous(&self) -> Self {
        match self {
            Platform::All => Platform::HumbleStore,
            Platform::Steam => Platform::All,
            Platform::EpicGames => Platform::Steam,
            Platform::Gog => Platform::EpicGames,
            Platform::HumbleStore => Platform::Gog,
        }
    }
}

/// Represents a game deal from IsThereAnyDeal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deal {
    pub title: String,
    pub shop: Shop,
    pub price: Price,
    pub url: String,
    pub history_low: Option<f64>,
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
