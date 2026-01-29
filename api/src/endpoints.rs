use dealve_core::{models::Deal, Result};
use crate::client::ItadClient;

impl ItadClient {
    pub async fn get_deals(&self) -> Result<Vec<Deal>> {
        // TODO: Implement API call
        Ok(vec![])
    }
}
