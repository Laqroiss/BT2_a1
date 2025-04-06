use reqwest::Client;
use serde::Deserialize;
use vader_sentiment::SentimentIntensityAnalyzer;

use crate::models::news::NewsItem;

#[derive(Debug, Deserialize)]
struct CoinGeckoResponse {
    status_updates: Vec<CoinGeckoUpdate>,
}

#[derive(Debug, Deserialize)]
struct CoinGeckoUpdate {
    project: String,
    created_at: String,
    description: String,
    user: String,
}

pub async fn fetch_coingecko_news(symbol: &str) -> Result<Vec<NewsItem>, reqwest::Error> {
    let id = symbol.to_lowercase(); // e.g., BTC → btc
    let url = format!("https://api.coingecko.com/api/v3/coins/{}/status_updates", id);

    let client = Client::new();
    let response = client.get(&url).send().await?;
    let json: CoinGeckoResponse = response.json().await?;

    let analyzer = SentimentIntensityAnalyzer::new();

    let news = json.status_updates.into_iter().map(|item| {
        let scores = analyzer.polarity_scores(&item.description);
        let score = *scores.get("compound").unwrap_or(&0.0);                     
        let sentiment = if score > 0.3 {
            "positive"
        } else if score < -0.3 {
            "negative"
        } else {
            "neutral"
        }.to_string();

        NewsItem {
            title: format!("{} Update", item.project),
            source_name: item.user,
            date: item.created_at,
            summary: item.description,
            url: format!("https://www.coingecko.com/en/coins/{}", id),
            sentiment,
        }
    }).collect();

    Ok(news)
}
