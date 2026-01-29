use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DealsResponse {
    pub list: Vec<DealItem>,
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
    pub cut: u8,
    pub url: String,
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

impl From<DealItem> for dealve_core::models::Deal {
    fn from(item: DealItem) -> Self {
        Self {
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
            url: item.deal.url,
        }
    }
}
