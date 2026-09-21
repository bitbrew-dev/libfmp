//! Commodity catalog and quote endpoints.
//!
//! Commodity-specific facades keep shared quote routes discoverable without
//! duplicating their query or response contracts.

use crate::{
    Client, Result,
    endpoints::{
        EndpointSpec,
        metadata::{EndpointMetadata, GeographicAvailability},
    },
    responses::{
        chart::{StockChartFullBar, StockChartIntradayBar, StockChartLightBar},
        commodities::{CommodityListing, Quote, QuoteShort},
    },
};

pub use super::asset_chart::AssetChartQuery;
pub use super::quote::{QuoteQuery, QuoteShortQuery, ShortOnlyQuery};

const US_ONLY: EndpointMetadata =
    EndpointMetadata::new().with_geography(GeographicAvailability::UsOnly);

/// Describes `GET commodities-list` without binding it to a transport.
pub fn commodities_list() -> EndpointSpec<(), Vec<CommodityListing>> {
    EndpointSpec::get("commodities-list", "commodities-list", ())
}

/// Describes the commodity-specific `GET quote` contract.
pub fn commodity_quote(query: QuoteQuery) -> EndpointSpec<QuoteQuery, Vec<Quote>> {
    EndpointSpec::get("quote", "quote", query).with_metadata(US_ONLY)
}

/// Describes the commodity-specific `GET quote-short` contract.
pub fn commodity_quote_short(
    query: QuoteShortQuery,
) -> EndpointSpec<QuoteShortQuery, Vec<QuoteShort>> {
    EndpointSpec::get("quote-short", "quote-short", query).with_metadata(US_ONLY)
}

/// Describes compact `GET batch-commodity-quotes`.
///
/// The returned query always emits `short=true`; the undocumented full batch
/// response shape is deliberately deferred.
pub fn commodity_quotes() -> EndpointSpec<ShortOnlyQuery, Vec<QuoteShort>> {
    super::quote::commodity_quotes()
}

/// Describes compact commodity `GET historical-price-eod/light` history.
pub fn commodity_chart_light(
    query: AssetChartQuery,
) -> EndpointSpec<AssetChartQuery, Vec<StockChartLightBar>> {
    super::asset_chart::chart_light(query)
}

/// Describes detailed commodity `GET historical-price-eod/full` history.
pub fn commodity_chart_full(
    query: AssetChartQuery,
) -> EndpointSpec<AssetChartQuery, Vec<StockChartFullBar>> {
    super::asset_chart::chart_full(query)
}

/// Describes commodity `GET historical-chart/1min` history.
pub fn commodity_chart_one_minute(
    query: AssetChartQuery,
) -> EndpointSpec<AssetChartQuery, Vec<StockChartIntradayBar>> {
    super::asset_chart::chart_one_minute(query)
}

/// Describes commodity `GET historical-chart/5min` history.
pub fn commodity_chart_five_minutes(
    query: AssetChartQuery,
) -> EndpointSpec<AssetChartQuery, Vec<StockChartIntradayBar>> {
    super::asset_chart::chart_five_minutes(query)
}

/// Describes commodity `GET historical-chart/1hour` history.
pub fn commodity_chart_one_hour(
    query: AssetChartQuery,
) -> EndpointSpec<AssetChartQuery, Vec<StockChartIntradayBar>> {
    super::asset_chart::chart_one_hour(query)
}

impl Client {
    /// Lists the provider's documented commodity catalog.
    pub async fn commodities_list(&self) -> Result<Vec<CommodityListing>> {
        self.execute(&commodities_list()).await
    }

    /// Retrieves a detailed US-only quote for one commodity.
    pub async fn commodity_quote(&self, query: impl Into<QuoteQuery>) -> Result<Vec<Quote>> {
        self.execute(&commodity_quote(query.into())).await
    }

    /// Retrieves a compact US-only quote for one commodity.
    pub async fn commodity_quote_short(
        &self,
        query: impl Into<QuoteShortQuery>,
    ) -> Result<Vec<QuoteShort>> {
        self.execute(&commodity_quote_short(query.into())).await
    }

    /// Retrieves compact end-of-day commodity history.
    ///
    /// The documented 5,000-row maximum is advisory metadata. Use
    /// [`EndpointBounds::accepts_response_rows`](super::metadata::EndpointBounds::accepts_response_rows)
    /// for optional preflight validation.
    pub async fn commodity_chart_light(
        &self,
        query: impl Into<AssetChartQuery>,
    ) -> Result<Vec<StockChartLightBar>> {
        self.execute(&commodity_chart_light(query.into())).await
    }

    /// Retrieves detailed end-of-day commodity history.
    ///
    /// The documented 5,000-row maximum is advisory metadata. Use
    /// [`EndpointBounds::accepts_response_rows`](super::metadata::EndpointBounds::accepts_response_rows)
    /// for optional preflight validation.
    pub async fn commodity_chart_full(
        &self,
        query: impl Into<AssetChartQuery>,
    ) -> Result<Vec<StockChartFullBar>> {
        self.execute(&commodity_chart_full(query.into())).await
    }

    /// Retrieves one-minute commodity history.
    pub async fn commodity_chart_one_minute(
        &self,
        query: impl Into<AssetChartQuery>,
    ) -> Result<Vec<StockChartIntradayBar>> {
        self.execute(&commodity_chart_one_minute(query.into()))
            .await
    }

    /// Retrieves five-minute commodity history.
    pub async fn commodity_chart_five_minutes(
        &self,
        query: impl Into<AssetChartQuery>,
    ) -> Result<Vec<StockChartIntradayBar>> {
        self.execute(&commodity_chart_five_minutes(query.into()))
            .await
    }

    /// Retrieves one-hour commodity history.
    pub async fn commodity_chart_one_hour(
        &self,
        query: impl Into<AssetChartQuery>,
    ) -> Result<Vec<StockChartIntradayBar>> {
        self.execute(&commodity_chart_one_hour(query.into())).await
    }
}
