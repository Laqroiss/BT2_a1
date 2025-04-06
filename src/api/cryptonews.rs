use reqwest::Client;
use serde::Deserialize;
use vader_sentiment::SentimentIntensityAnalyzer;

use crate::models::news::NewsItem;

#[derive(Debug, Deserialize)]
struct CryptoCompareResponse {
    Data: Vec<CryptoCompareArticle>,
}

#[derive(Debug, Deserialize)]
struct CryptoCompareArticle {
    title: String,
    source: String,
    published_on: u64,
    body: String,
    url: String,
}

pub async fn fetch_cryptonews(_symbol: &str) -> Result<Vec<NewsItem>, reqwest::Error> {
    let url = "https://min-api.cryptocompare.com/data/v2/news/?lang=EN";

    let client = Client::new();
    let response = client.get(url).send().await?;
    let json: CryptoCompareResponse = response.json().await?;

    let analyzer = SentimentIntensityAnalyzer::new();

    let articles = json.Data.into_iter().map(|article| {
        let date = chrono::NaiveDateTime::from_timestamp(article.published_on as i64, 0)
        .format("%Y-%m-%d %H:%M")
        .to_string();
    

        let scores = analyzer.polarity_scores(&article.body);
        let score = *scores.get("compound").unwrap_or(&0.0);            
        let sentiment = if score > 0.3 {
            "positive"
        } else if score < -0.3 {
            "negative"
        } else {
            "neutral"
        }.to_string();

        NewsItem {
            title: article.title,
            source_name: article.source,
            date,
            summary: article.body,
            url: article.url,
            sentiment,
        }
    }).collect();

    Ok(articles)
}
