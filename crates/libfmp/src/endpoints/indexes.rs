//! Stock-market index directory, quote, history, and constituent endpoints.
//!
//! Index-specific descriptors make the provider's shared quote and chart
//! routes discoverable without duplicating their response contracts.

use crate::{
    Client, Result,
    endpoints::{
        EndpointSpec, QueryEncoder, QueryParameters,
        metadata::{EndpointBounds, EndpointMetadata, GeographicAvailability},
    },
    responses::{
        chart::{StockChartFullBar, StockChartIntradayBar, StockChartLightBar},
        indexes::IndexListing,
        quote::{Quote, QuoteShort},
    },
    types::{Date, Ticker},
};

pub use super::quote::{QuoteQuery, QuoteShortQuery, ShortOnlyQuery};

const WORLDWIDE: EndpointMetadata =
    EndpointMetadata::new().with_geography(GeographicAvailability::Worldwide);
const WORLDWIDE_EOD: EndpointMetadata =
    WORLDWIDE.with_bounds(EndpointBounds::new().with_response_rows(5_000));

/// Shared query parameters for the five documented index chart endpoints.
///
/// The index documentation exposes only independent `from` and `to` date
/// filters. Stock-only chart flags therefore do not appear in this contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexChartQuery {
    symbol: Ticker,
    from: Option<Date>,
    to: Option<Date>,
}

impl IndexChartQuery {
    /// Creates a query for one index without undocumented date defaults.
    pub fn new(symbol: Ticker) -> Self {
        Self {
            symbol,
            from: None,
            to: None,
        }
    }

    /// Sets the optional independent start date.
    pub const fn with_from(mut self, from: Date) -> Self {
        self.from = Some(from);
        self
    }

    /// Sets the optional independent end date.
    pub const fn with_to(mut self, to: Date) -> Self {
        self.to = Some(to);
        self
    }

    /// Borrows the requested index symbol.
    pub const fn symbol(&self) -> &Ticker {
        &self.symbol
    }

    /// Returns the optional independent start date.
    pub const fn from(&self) -> Option<Date> {
        self.from
    }

    /// Returns the optional independent end date.
    pub const fn to(&self) -> Option<Date> {
        self.to
    }
}

impl From<Ticker> for IndexChartQuery {
    fn from(symbol: Ticker) -> Self {
        Self::new(symbol)
    }
}

impl From<&Ticker> for IndexChartQuery {
    fn from(symbol: &Ticker) -> Self {
        Self::new(symbol.clone())
    }
}

impl QueryParameters for IndexChartQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.required("symbol", &self.symbol);
        encoder.optional("from", self.from);
        encoder.optional("to", self.to);
    }
}

/// Describes `GET index-list` without binding it to a transport.
pub fn index_list() -> EndpointSpec<(), Vec<IndexListing>> {
    EndpointSpec::get("index-list", "index-list", ()).with_metadata(WORLDWIDE)
}

/// Describes the generic `GET quote` route for one stock-market index.
pub fn index_quote(query: QuoteQuery) -> EndpointSpec<QuoteQuery, Vec<Quote>> {
    super::quote::quote(query)
}

/// Describes the generic `GET quote-short` route for one stock-market index.
pub fn index_quote_short(query: QuoteShortQuery) -> EndpointSpec<QuoteShortQuery, Vec<QuoteShort>> {
    super::quote::quote_short(query)
}

/// Describes compact `GET batch-index-quotes` without binding it to a transport.
///
/// The returned query always emits `short=true`. The provider does not
/// document the batch endpoint's full response shape, so callers cannot select
/// `short=false` through this typed contract.
pub fn index_quotes() -> EndpointSpec<ShortOnlyQuery, Vec<QuoteShort>> {
    super::quote::index_quotes()
}

/// Describes compact `GET historical-price-eod/light` index history.
pub fn index_chart_light(
    query: IndexChartQuery,
) -> EndpointSpec<IndexChartQuery, Vec<StockChartLightBar>> {
    EndpointSpec::get(
        "historical-price-eod/light",
        "historical-price-eod/light",
        query,
    )
    .with_metadata(WORLDWIDE_EOD)
}

/// Describes detailed `GET historical-price-eod/full` index history.
pub fn index_chart_full(
    query: IndexChartQuery,
) -> EndpointSpec<IndexChartQuery, Vec<StockChartFullBar>> {
    EndpointSpec::get(
        "historical-price-eod/full",
        "historical-price-eod/full",
        query,
    )
    .with_metadata(WORLDWIDE_EOD)
}

/// Describes `GET historical-chart/1min` index history.
pub fn index_chart_one_minute(
    query: IndexChartQuery,
) -> EndpointSpec<IndexChartQuery, Vec<StockChartIntradayBar>> {
    EndpointSpec::get("historical-chart/1min", "historical-chart/1min", query)
        .with_metadata(WORLDWIDE)
}

/// Describes `GET historical-chart/5min` index history.
pub fn index_chart_five_minutes(
    query: IndexChartQuery,
) -> EndpointSpec<IndexChartQuery, Vec<StockChartIntradayBar>> {
    EndpointSpec::get("historical-chart/5min", "historical-chart/5min", query)
        .with_metadata(WORLDWIDE)
}

/// Describes `GET historical-chart/1hour` index history.
pub fn index_chart_one_hour(
    query: IndexChartQuery,
) -> EndpointSpec<IndexChartQuery, Vec<StockChartIntradayBar>> {
    EndpointSpec::get("historical-chart/1hour", "historical-chart/1hour", query)
        .with_metadata(WORLDWIDE)
}

impl Client {
    /// Lists worldwide stock-market indexes.
    pub async fn index_list(&self) -> Result<Vec<IndexListing>> {
        self.execute(&index_list()).await
    }

    /// Retrieves a detailed quote for one stock-market index.
    pub async fn index_quote(&self, query: impl Into<QuoteQuery>) -> Result<Vec<Quote>> {
        self.execute(&index_quote(query.into())).await
    }

    /// Retrieves a compact quote for one stock-market index.
    pub async fn index_quote_short(
        &self,
        query: impl Into<QuoteShortQuery>,
    ) -> Result<Vec<QuoteShort>> {
        self.execute(&index_quote_short(query.into())).await
    }

    /// Retrieves compact end-of-day history for one stock-market index.
    pub async fn index_chart_light(
        &self,
        query: impl Into<IndexChartQuery>,
    ) -> Result<Vec<StockChartLightBar>> {
        self.execute(&index_chart_light(query.into())).await
    }

    /// Retrieves detailed end-of-day history for one stock-market index.
    pub async fn index_chart_full(
        &self,
        query: impl Into<IndexChartQuery>,
    ) -> Result<Vec<StockChartFullBar>> {
        self.execute(&index_chart_full(query.into())).await
    }

    /// Retrieves one-minute history for one stock-market index.
    pub async fn index_chart_one_minute(
        &self,
        query: impl Into<IndexChartQuery>,
    ) -> Result<Vec<StockChartIntradayBar>> {
        self.execute(&index_chart_one_minute(query.into())).await
    }

    /// Retrieves five-minute history for one stock-market index.
    pub async fn index_chart_five_minutes(
        &self,
        query: impl Into<IndexChartQuery>,
    ) -> Result<Vec<StockChartIntradayBar>> {
        self.execute(&index_chart_five_minutes(query.into())).await
    }

    /// Retrieves one-hour history for one stock-market index.
    pub async fn index_chart_one_hour(
        &self,
        query: impl Into<IndexChartQuery>,
    ) -> Result<Vec<StockChartIntradayBar>> {
        self.execute(&index_chart_one_hour(query.into())).await
    }
}
