//! Shared commodity, forex, and cryptocurrency chart contracts.

use serde::de::DeserializeOwned;

use crate::{
    endpoints::{
        EndpointSpec, QueryEncoder, QueryParameters,
        metadata::{EndpointBounds, EndpointMetadata},
    },
    responses::chart::{StockChartFullBar, StockChartIntradayBar, StockChartLightBar},
    types::{Date, Ticker},
};

/// Query parameters shared by commodity, forex, and cryptocurrency charts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetChartQuery {
    symbol: Ticker,
    from: Option<Date>,
    to: Option<Date>,
}

impl AssetChartQuery {
    /// Creates a chart query without undocumented date defaults.
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

    /// Borrows the requested asset symbol.
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

impl From<Ticker> for AssetChartQuery {
    fn from(symbol: Ticker) -> Self {
        Self::new(symbol)
    }
}

impl From<&Ticker> for AssetChartQuery {
    fn from(symbol: &Ticker) -> Self {
        Self::new(symbol.clone())
    }
}

impl QueryParameters for AssetChartQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.required("symbol", &self.symbol);
        encoder.optional("from", self.from);
        encoder.optional("to", self.to);
    }
}

const EOD_METADATA: EndpointMetadata =
    EndpointMetadata::new().with_bounds(EndpointBounds::new().with_response_rows(5_000));

fn endpoint<R>(
    path: &'static str,
    query: AssetChartQuery,
    metadata: EndpointMetadata,
) -> EndpointSpec<AssetChartQuery, Vec<R>>
where
    R: DeserializeOwned,
{
    EndpointSpec::get(path, path, query).with_metadata(metadata)
}

pub(crate) fn chart_light(
    query: AssetChartQuery,
) -> EndpointSpec<AssetChartQuery, Vec<StockChartLightBar>> {
    endpoint("historical-price-eod/light", query, EOD_METADATA)
}

pub(crate) fn chart_full(
    query: AssetChartQuery,
) -> EndpointSpec<AssetChartQuery, Vec<StockChartFullBar>> {
    endpoint("historical-price-eod/full", query, EOD_METADATA)
}

pub(crate) fn chart_one_minute(
    query: AssetChartQuery,
) -> EndpointSpec<AssetChartQuery, Vec<StockChartIntradayBar>> {
    endpoint("historical-chart/1min", query, EndpointMetadata::new())
}

pub(crate) fn chart_five_minutes(
    query: AssetChartQuery,
) -> EndpointSpec<AssetChartQuery, Vec<StockChartIntradayBar>> {
    endpoint("historical-chart/5min", query, EndpointMetadata::new())
}

pub(crate) fn chart_one_hour(
    query: AssetChartQuery,
) -> EndpointSpec<AssetChartQuery, Vec<StockChartIntradayBar>> {
    endpoint("historical-chart/1hour", query, EndpointMetadata::new())
}
