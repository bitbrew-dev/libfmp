//! Response rows returned by news endpoints.

use serde::{Deserialize, Deserializer, Serialize};

use crate::types::{ApiDateTime, Ticker};

fn required_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

/// One Financial Modeling Prep editorial article.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct Article {
    pub title: String,
    pub date: ApiDateTime,
    pub content: String,
    pub tickers: String,
    pub image: String,
    pub link: String,
    pub author: String,
    pub site: String,
}

/// One provider-news article shared across general and market-specific feeds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct NewsArticle {
    #[serde(deserialize_with = "required_option")]
    pub symbol: Option<Ticker>,
    pub published_date: ApiDateTime,
    pub publisher: String,
    pub title: String,
    #[serde(deserialize_with = "required_option")]
    pub image: Option<String>,
    #[serde(deserialize_with = "required_option")]
    pub site: Option<String>,
    pub text: String,
    pub url: String,
}
