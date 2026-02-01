use dealve_core::{models::Deal, DealveError, Result};
use crate::{client::ItadClient, types::DealsResponse};

impl ItadClient {
    pub async fn get_deals(
        &self,
        country: &str,
        limit: usize,
        shop_id: Option<u32>,
    ) -> Result<Vec<Deal>> {
        let api_key = self.api_key().ok_or_else(|| {
            DealveError::Config("API key is required".to_string())
        })?;

        let url = format!("{}/deals/v2", self.base_url());

        let mut query_params: Vec<(&str, String)> = vec![
            ("key", api_key.to_string()),
            ("country", country.to_string()),
            ("limit", limit.to_string()),
        ];

        if let Some(id) = shop_id {
            query_params.push(("shops", id.to_string()));
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
}
