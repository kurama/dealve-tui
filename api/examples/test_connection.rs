use dealve_api::ItadClient;
use std::env;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let api_key = env::var("ITAD_API_KEY").ok();

    if api_key.is_none() {
        eprintln!("Error: ITAD_API_KEY not set.");
        eprintln!("Create a .env file with:");
        eprintln!("ITAD_API_KEY=your_key_here\n");
        return;
    }

    let client = ItadClient::new(api_key);

    println!("Fetching top 5 deals from IsThereAnyDeal...\n");

    match client.get_deals("US", 20, None).await {
        Ok(deals) => {
            println!("Found {} deals:", deals.len());
            for (i, deal) in deals.iter().enumerate() {
                println!(
                    "{}. {} - ${:.2} (-{}%) @ {} (ID: {})",
                    i + 1,
                    deal.title,
                    deal.price.amount,
                    deal.price.discount,
                    deal.shop.name,
                    deal.shop.id
                );
            }
        }
        Err(e) => {
            eprintln!("Error fetching deals: {}", e);
        }
    }
}
