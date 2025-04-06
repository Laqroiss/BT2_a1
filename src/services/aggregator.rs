use crate::api::{coingecko, cryptonews};
use crate::models::news::NewsItem;

pub async fn fetch_combined_news(symbol: &str) -> Result<Vec<NewsItem>, String> {
    let mut combined_news = Vec::new();
    let mut errors = Vec::new();

    // Try CryptoCompare (cryptonews)
    match cryptonews::fetch_cryptonews(symbol).await {
        Ok(mut news) => combined_news.append(&mut news),
        Err(e) => errors.push(format!("CryptoCompare error: {}", e)),
    }

    // Try CoinGecko
    match coingecko::fetch_coingecko_news(symbol).await {
        Ok(mut news) => combined_news.append(&mut news),
        Err(e) => errors.push(format!("CoinGecko error: {}", e)),
    }

    if !combined_news.is_empty() {
        Ok(combined_news)
    } else {
        Err(errors.join(" | "))
    }
}
