//! Cryptocurrency catalog and quote endpoints.
//!
//! Cryptocurrency-specific facades keep shared quote routes discoverable
//! without duplicating their query or response contracts.

use crate::{
    Client, Result,
    endpoints::EndpointSpec,
    responses::{
        chart::{StockChartFullBar, StockChartIntradayBar, StockChartLightBar},
        crypto::{CryptocurrencyListing, Quote, QuoteShort},
    },
};

pub use super::asset_chart::AssetChartQuery;
pub use super::quote::{QuoteQuery, QuoteShortQuery, ShortOnlyQuery};

/// Describes `GET cryptocurrency-list` without binding it to a transport.
pub fn cryptocurrency_list() -> EndpointSpec<(), Vec<CryptocurrencyListing>> {
    EndpointSpec::get("cryptocurrency-list", "cryptocurrency-list", ())
}

/// Describes the cryptocurrency-specific `GET quote` contract.
pub fn cryptocurrency_quote(query: QuoteQuery) -> EndpointSpec<QuoteQuery, Vec<Quote>> {
    EndpointSpec::get("quote", "quote", query)
}

/// Describes the cryptocurrency-specific `GET quote-short` contract.
pub fn cryptocurrency_quote_short(
    query: QuoteShortQuery,
) -> EndpointSpec<QuoteShortQuery, Vec<QuoteShort>> {
    EndpointSpec::get("quote-short", "quote-short", query)
}

/// Describes compact `GET batch-crypto-quotes`.
///
/// The returned query always emits `short=true`; the undocumented full batch
/// response shape is deliberately deferred.
pub fn cryptocurrency_quotes() -> EndpointSpec<ShortOnlyQuery, Vec<QuoteShort>> {
    super::quote::cryptocurrency_quotes()
}

/// Describes compact cryptocurrency `GET historical-price-eod/light` history.
pub fn cryptocurrency_chart_light(
    query: AssetChartQuery,
) -> EndpointSpec<AssetChartQuery, Vec<StockChartLightBar>> {
    super::asset_chart::chart_light(query)
}

/// Describes detailed cryptocurrency `GET historical-price-eod/full` history.
pub fn cryptocurrency_chart_full(
    query: AssetChartQuery,
) -> EndpointSpec<AssetChartQuery, Vec<StockChartFullBar>> {
    super::asset_chart::chart_full(query)
}

/// Describes cryptocurrency `GET historical-chart/1min` history.
pub fn cryptocurrency_chart_one_minute(
    query: AssetChartQuery,
) -> EndpointSpec<AssetChartQuery, Vec<StockChartIntradayBar>> {
    super::asset_chart::chart_one_minute(query)
}

/// Describes cryptocurrency `GET historical-chart/5min` history.
pub fn cryptocurrency_chart_five_minutes(
    query: AssetChartQuery,
) -> EndpointSpec<AssetChartQuery, Vec<StockChartIntradayBar>> {
    super::asset_chart::chart_five_minutes(query)
}

/// Describes cryptocurrency `GET historical-chart/1hour` history.
pub fn cryptocurrency_chart_one_hour(
    query: AssetChartQuery,
) -> EndpointSpec<AssetChartQuery, Vec<StockChartIntradayBar>> {
    super::asset_chart::chart_one_hour(query)
}

impl Client {
    /// Lists the provider's documented cryptocurrency catalog.
    pub async fn cryptocurrency_list(&self) -> Result<Vec<CryptocurrencyListing>> {
        self.execute(&cryptocurrency_list()).await
    }

    /// Retrieves a detailed quote for one cryptocurrency.
    pub async fn cryptocurrency_quote(&self, query: impl Into<QuoteQuery>) -> Result<Vec<Quote>> {
        self.execute(&cryptocurrency_quote(query.into())).await
    }

    /// Retrieves a compact quote for one cryptocurrency.
    pub async fn cryptocurrency_quote_short(
        &self,
        query: impl Into<QuoteShortQuery>,
    ) -> Result<Vec<QuoteShort>> {
        self.execute(&cryptocurrency_quote_short(query.into()))
            .await
    }

    /// Retrieves compact end-of-day cryptocurrency history.
    ///
    /// The documented 5,000-row maximum is advisory metadata. Use
    /// [`EndpointBounds::accepts_response_rows`](super::metadata::EndpointBounds::accepts_response_rows)
    /// for optional preflight validation.
    pub async fn cryptocurrency_chart_light(
        &self,
        query: impl Into<AssetChartQuery>,
    ) -> Result<Vec<StockChartLightBar>> {
        self.execute(&cryptocurrency_chart_light(query.into()))
            .await
    }

    /// Retrieves detailed end-of-day cryptocurrency history.
    ///
    /// The documented 5,000-row maximum is advisory metadata. Use
    /// [`EndpointBounds::accepts_response_rows`](super::metadata::EndpointBounds::accepts_response_rows)
    /// for optional preflight validation.
    pub async fn cryptocurrency_chart_full(
        &self,
        query: impl Into<AssetChartQuery>,
    ) -> Result<Vec<StockChartFullBar>> {
        self.execute(&cryptocurrency_chart_full(query.into())).await
    }

    /// Retrieves one-minute cryptocurrency history.
    pub async fn cryptocurrency_chart_one_minute(
        &self,
        query: impl Into<AssetChartQuery>,
    ) -> Result<Vec<StockChartIntradayBar>> {
        self.execute(&cryptocurrency_chart_one_minute(query.into()))
            .await
    }

    /// Retrieves five-minute cryptocurrency history.
    pub async fn cryptocurrency_chart_five_minutes(
        &self,
        query: impl Into<AssetChartQuery>,
    ) -> Result<Vec<StockChartIntradayBar>> {
        self.execute(&cryptocurrency_chart_five_minutes(query.into()))
            .await
    }

    /// Retrieves one-hour cryptocurrency history.
    pub async fn cryptocurrency_chart_one_hour(
        &self,
        query: impl Into<AssetChartQuery>,
    ) -> Result<Vec<StockChartIntradayBar>> {
        self.execute(&cryptocurrency_chart_one_hour(query.into()))
            .await
    }
}
