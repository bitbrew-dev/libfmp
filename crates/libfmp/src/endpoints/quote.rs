//! Stock, index, commodity, forex, and cryptocurrency quote endpoints.

use crate::{
    Client, Result,
    endpoints::{EndpointSpec, QueryEncoder, QueryParameters},
    responses::quote::QuoteShort,
    types::Ticker,
};

/// Required query parameters for the quote-short endpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuoteShortQuery {
    symbol: Ticker,
}

impl QuoteShortQuery {
    /// Creates a query for one validated provider ticker.
    pub fn new(symbol: Ticker) -> Self {
        Self { symbol }
    }

    /// Borrows the requested ticker.
    pub fn symbol(&self) -> &Ticker {
        &self.symbol
    }
}

impl From<Ticker> for QuoteShortQuery {
    fn from(symbol: Ticker) -> Self {
        Self::new(symbol)
    }
}

impl From<&Ticker> for QuoteShortQuery {
    fn from(symbol: &Ticker) -> Self {
        Self::new(symbol.clone())
    }
}

impl QueryParameters for QuoteShortQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.required("symbol", &self.symbol);
    }
}

/// Describes `GET quote-short` without binding it to a transport.
pub fn quote_short(query: QuoteShortQuery) -> EndpointSpec<QuoteShortQuery, Vec<QuoteShort>> {
    EndpointSpec::get("quote-short", "quote-short", query)
}

impl Client {
    /// Retrieves the provider's compact quote array for one ticker.
    ///
    /// The returned vector preserves the documented bare-array response: an
    /// empty provider response remains empty, and multiple rows remain present.
    /// Both an owned [`Ticker`] and `&Ticker` are accepted.
    ///
    /// ```no_run
    /// use libfmp::{Client, types::Ticker};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::builder()
    ///     .base_url("https://proxy.example/fmp")
    ///     .path_prefix("stable")
    ///     .build()?;
    /// let ticker = Ticker::new("AAPL")?;
    /// let quotes = client.quote_short(&ticker).await?;
    /// println!("received {} compact quote rows", quotes.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn quote_short(&self, symbol: impl Into<QuoteShortQuery>) -> Result<Vec<QuoteShort>> {
        self.execute(&quote_short(symbol.into())).await
    }
}
