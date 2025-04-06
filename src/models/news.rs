#[derive(Debug, Clone)]
pub struct NewsItem {
    pub title: String,
    pub source_name: String,
    pub date: String,
    pub summary: String,
    pub url: String,
    pub sentiment: String,
}
