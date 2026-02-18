use crate::{
    client::ItadClient,
    types::{
        DealInfo, DealsResponse, GameInfoResponse, GamePriceItem, GameSearchItem, PriceHistoryItem,
    },
};
use dealve_core::{
    models::{Deal, GameInfo, PriceHistoryPoint},
    DealveError, Result,
};
use std::{cmp::Ordering, collections::HashMap};

impl ItadClient {
    pub async fn get_deals(
        &self,
        country: &str,
        limit: usize,
        offset: usize,
        shop_id: Option<u32>,
        sort: Option<&str>,
    ) -> Result<Vec<Deal>> {
        let api_key = self
            .api_key()
            .ok_or_else(|| DealveError::Config("API key is required".to_string()))?;

        let url = format!("{}/deals/v2", self.base_url());

        let mut query_params: Vec<(&str, String)> = vec![
            ("key", api_key.to_string()),
            ("country", country.to_string()),
            ("limit", limit.to_string()),
            ("offset", offset.to_string()),
        ];

        if let Some(id) = shop_id {
            query_params.push(("shops", id.to_string()));
        }

        if let Some(s) = sort {
            query_params.push(("sort", s.to_string()));
        }

        let response = self
            .client()
            .get(&url)
            .query(&query_params)
            .send()
            .await
            .map_err(|e| DealveError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(DealveError::Api(format!(
                "API returned status {}: {}",
                status, body
            )));
        }

        let deals_response: DealsResponse = response
            .json()
            .await
            .map_err(|e| DealveError::Parse(e.to_string()))?;

        Ok(deals_response.list.into_iter().map(Deal::from).collect())
    }

    pub async fn get_game_info(&self, game_id: &str) -> Result<GameInfo> {
        let api_key = self
            .api_key()
            .ok_or_else(|| DealveError::Config("API key is required".to_string()))?;

        let url = format!("{}/games/info/v2", self.base_url());

        let response = self
            .client()
            .get(&url)
            .query(&[("key", api_key), ("id", game_id)])
            .send()
            .await
            .map_err(|e| DealveError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(DealveError::Api(format!(
                "API returned status {}: {}",
                status, body
            )));
        }

        let info_response: GameInfoResponse = response
            .json()
            .await
            .map_err(|e| DealveError::Parse(e.to_string()))?;

        Ok(GameInfo::from(info_response))
    }

    pub async fn search_games(&self, title: &str, results: usize) -> Result<Vec<GameSearchItem>> {
        let api_key = self
            .api_key()
            .ok_or_else(|| DealveError::Config("API key is required".to_string()))?;

        if title.trim().is_empty() || results == 0 {
            return Ok(vec![]);
        }

        let url = format!("{}/games/search/v1", self.base_url());
        let query_params = [
            ("key", api_key.to_string()),
            ("title", title.to_string()),
            ("results", results.to_string()),
        ];

        let response = self
            .client()
            .get(&url)
            .query(&query_params)
            .send()
            .await
            .map_err(|e| DealveError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(DealveError::Api(format!(
                "API returned status {}: {}",
                status, body
            )));
        }

        response
            .json()
            .await
            .map_err(|e| DealveError::Parse(e.to_string()))
    }

    pub async fn get_prices_for_games(
        &self,
        ids: &[String],
        country: &str,
        shop_id: Option<u32>,
    ) -> Result<Vec<GamePriceItem>> {
        let api_key = self
            .api_key()
            .ok_or_else(|| DealveError::Config("API key is required".to_string()))?;

        if ids.is_empty() {
            return Ok(vec![]);
        }

        let url = format!("{}/games/prices/v3", self.base_url());

        let mut query_params: Vec<(&str, String)> = vec![
            ("key", api_key.to_string()),
            ("country", country.to_string()),
            ("deals", "true".to_string()),
        ];

        if let Some(id) = shop_id {
            // Single shop: one deal per game is enough
            query_params.push(("capacity", "1".to_string()));
            query_params.push(("shops", id.to_string()));
        }

        let response = self
            .client()
            .post(&url)
            .query(&query_params)
            .json(ids)
            .send()
            .await
            .map_err(|e| DealveError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(DealveError::Api(format!(
                "API returned status {}: {}",
                status, body
            )));
        }

        response
            .json()
            .await
            .map_err(|e| DealveError::Parse(e.to_string()))
    }

    pub async fn search_deals(
        &self,
        query: &str,
        country: &str,
        shop_id: Option<u32>,
        limit: usize,
    ) -> Result<Vec<Deal>> {
        let query = query.trim();
        if query.is_empty() || limit == 0 {
            return Ok(vec![]);
        }

        let search_results = self.search_games(query, limit).await?;
        if search_results.is_empty() {
            return Ok(vec![]);
        }

        let mut ids = Vec::with_capacity(search_results.len());
        let mut titles_by_id = HashMap::with_capacity(search_results.len());

        for result in search_results {
            if titles_by_id.contains_key(&result.id) {
                continue;
            }
            ids.push(result.id.clone());
            titles_by_id.insert(result.id, result.title);
        }

        let prices = self.get_prices_for_games(&ids, country, shop_id).await?;
        let mut deals_by_id: HashMap<String, (DealInfo, Option<f64>)> = HashMap::new();

        for price_item in prices {
            let history_low = price_item
                .history_low
                .and_then(|h| h.all.map(|price| price.amount));

            if let Some(best_deal) = select_best_deal(price_item.deals) {
                deals_by_id.insert(price_item.id, (best_deal, history_low));
            }
        }

        let mut deals = Vec::new();
        for id in ids {
            let Some((deal_info, history_low)) = deals_by_id.remove(&id) else {
                continue;
            };

            let title = titles_by_id.remove(&id).unwrap_or_else(|| id.clone());
            deals.push(Deal {
                id,
                title,
                shop: dealve_core::models::Shop {
                    id: deal_info.shop.id.to_string(),
                    name: deal_info.shop.name,
                },
                price: dealve_core::models::Price {
                    amount: deal_info.price.amount,
                    currency: deal_info.price.currency,
                    discount: deal_info.cut,
                },
                regular_price: deal_info.regular.amount,
                url: deal_info.url,
                history_low: history_low.or_else(|| deal_info.history_low.map(|h| h.amount)),
            });
        }

        Ok(deals)
    }

    /// Get price history for a game (max 1 year of data)
    pub async fn get_price_history(
        &self,
        game_id: &str,
        country: &str,
    ) -> Result<Vec<PriceHistoryPoint>> {
        let api_key = self
            .api_key()
            .ok_or_else(|| DealveError::Config("API key is required".to_string()))?;

        let url = format!("{}/games/history/v2", self.base_url());

        // Request data from 1 year ago (ISO 8601 format)
        let one_year_ago = chrono::Utc::now() - chrono::Duration::days(365);
        let since = one_year_ago.format("%Y-%m-%dT%H:%M:%SZ").to_string();

        let response = self
            .client()
            .get(&url)
            .query(&[
                ("key", api_key),
                ("id", game_id),
                ("country", country),
                ("since", since.as_str()),
            ])
            .send()
            .await
            .map_err(|e| DealveError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(DealveError::Api(format!(
                "API returned status {}: {}",
                status, body
            )));
        }

        let history_items: Vec<PriceHistoryItem> = response
            .json()
            .await
            .map_err(|e| DealveError::Parse(e.to_string()))?;

        // Convert to our model, filtering out items without deals
        // and sorting by timestamp (oldest first for charting)
        let mut points: Vec<PriceHistoryPoint> = history_items
            .into_iter()
            .filter_map(|item| {
                let deal = item.deal?;
                let timestamp = chrono::DateTime::parse_from_rfc3339(&item.timestamp)
                    .ok()?
                    .timestamp();
                Some(PriceHistoryPoint {
                    timestamp,
                    price: deal.price.amount,
                    shop_name: item.shop.name,
                })
            })
            .collect();

        // Sort by timestamp ascending (oldest first)
        points.sort_by_key(|p| p.timestamp);

        Ok(points)
    }

    /// Validate an API key by making a lightweight request
    /// Returns Ok(()) if valid, Err with specific error otherwise
    pub async fn validate_api_key(api_key: &str) -> Result<()> {
        let client = reqwest::Client::new();
        let url = "https://api.isthereanydeal.com/deals/v2";

        let response = client
            .get(url)
            .query(&[("key", api_key), ("limit", "1"), ("country", "US")])
            .send()
            .await
            .map_err(|e| DealveError::Network(e.to_string()))?;

        match response.status().as_u16() {
            200..=299 => Ok(()),
            401 | 403 => Err(DealveError::Api("Invalid API key".to_string())),
            429 => Err(DealveError::Api(
                "Rate limited - please wait and try again".to_string(),
            )),
            _ => {
                let body = response.text().await.unwrap_or_default();
                Err(DealveError::Api(format!("API error: {}", body)))
            }
        }
    }
}

fn select_best_deal(deals: Vec<DealInfo>) -> Option<DealInfo> {
    deals.into_iter().min_by(|a, b| {
        let price_order = a.price.amount.total_cmp(&b.price.amount);

        if price_order == Ordering::Equal {
            b.cut.cmp(&a.cut)
        } else {
            price_order
        }
    })
}
