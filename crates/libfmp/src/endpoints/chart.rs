//! Stock chart endpoint and query contracts.

use crate::{
    Client, Result,
    endpoints::{
        EndpointSpec, QueryEncoder, QueryParameters,
        metadata::{EndpointBounds, EndpointMetadata, GeographicAvailability},
    },
    responses::chart::{
        StockChartAdjustedBar, StockChartFullBar, StockChartIntradayBar, StockChartLightBar,
    },
    types::{Date, Ticker},
};

/// Shared query parameters for the four end-of-day stock chart endpoints.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StockChartEodQuery {
    symbol: Ticker,
    from: Option<Date>,
    to: Option<Date>,
}

impl StockChartEodQuery {
    /// Creates a query for one ticker without undocumented date defaults.
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

    /// Borrows the requested ticker.
    pub fn symbol(&self) -> &Ticker {
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

impl From<Ticker> for StockChartEodQuery {
    fn from(symbol: Ticker) -> Self {
        Self::new(symbol)
    }
}

impl From<&Ticker> for StockChartEodQuery {
    fn from(symbol: &Ticker) -> Self {
        Self::new(symbol.clone())
    }
}

impl QueryParameters for StockChartEodQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.required("symbol", &self.symbol);
        encoder.optional("from", self.from);
        encoder.optional("to", self.to);
    }
}

const STOCK_CHART_EOD_METADATA: EndpointMetadata = EndpointMetadata::new()
    .with_geography(GeographicAvailability::Worldwide)
    .with_bounds(EndpointBounds::new().with_response_rows(5_000));

/// Describes `GET historical-price-eod/light` without binding a transport.
pub fn stock_chart_light(
    query: StockChartEodQuery,
) -> EndpointSpec<StockChartEodQuery, Vec<StockChartLightBar>> {
    EndpointSpec::get(
        "historical-price-eod/light",
        "historical-price-eod/light",
        query,
    )
    .with_metadata(STOCK_CHART_EOD_METADATA)
}

/// Describes `GET historical-price-eod/full` without binding a transport.
pub fn stock_chart_full(
    query: StockChartEodQuery,
) -> EndpointSpec<StockChartEodQuery, Vec<StockChartFullBar>> {
    EndpointSpec::get(
        "historical-price-eod/full",
        "historical-price-eod/full",
        query,
    )
    .with_metadata(STOCK_CHART_EOD_METADATA)
}

/// Describes `GET historical-price-eod/non-split-adjusted` without binding a transport.
pub fn stock_chart_non_split_adjusted(
    query: StockChartEodQuery,
) -> EndpointSpec<StockChartEodQuery, Vec<StockChartAdjustedBar>> {
    EndpointSpec::get(
        "historical-price-eod/non-split-adjusted",
        "historical-price-eod/non-split-adjusted",
        query,
    )
    .with_metadata(STOCK_CHART_EOD_METADATA)
}

/// Describes `GET historical-price-eod/dividend-adjusted` without binding a transport.
pub fn stock_chart_dividend_adjusted(
    query: StockChartEodQuery,
) -> EndpointSpec<StockChartEodQuery, Vec<StockChartAdjustedBar>> {
    EndpointSpec::get(
        "historical-price-eod/dividend-adjusted",
        "historical-price-eod/dividend-adjusted",
        query,
    )
    .with_metadata(STOCK_CHART_EOD_METADATA)
}

impl Client {
    /// Retrieves compact worldwide end-of-day stock chart rows.
    ///
    /// The documented 5,000-row maximum is advisory metadata. Use
    /// [`EndpointBounds::accepts_response_rows`](super::metadata::EndpointBounds::accepts_response_rows)
    /// for optional preflight validation.
    pub async fn stock_chart_light(
        &self,
        query: impl Into<StockChartEodQuery>,
    ) -> Result<Vec<StockChartLightBar>> {
        self.execute(&stock_chart_light(query.into())).await
    }

    /// Retrieves detailed worldwide end-of-day stock chart rows.
    ///
    /// The documented 5,000-row maximum is advisory metadata. Use
    /// [`EndpointBounds::accepts_response_rows`](super::metadata::EndpointBounds::accepts_response_rows)
    /// for optional preflight validation.
    pub async fn stock_chart_full(
        &self,
        query: impl Into<StockChartEodQuery>,
    ) -> Result<Vec<StockChartFullBar>> {
        self.execute(&stock_chart_full(query.into())).await
    }

    /// Retrieves split-unadjusted worldwide end-of-day stock chart rows.
    ///
    /// The documented 5,000-row maximum is advisory metadata. Use
    /// [`EndpointBounds::accepts_response_rows`](super::metadata::EndpointBounds::accepts_response_rows)
    /// for optional preflight validation.
    pub async fn stock_chart_non_split_adjusted(
        &self,
        query: impl Into<StockChartEodQuery>,
    ) -> Result<Vec<StockChartAdjustedBar>> {
        self.execute(&stock_chart_non_split_adjusted(query.into()))
            .await
    }

    /// Retrieves dividend-adjusted worldwide end-of-day stock chart rows.
    ///
    /// The documented 5,000-row maximum is advisory metadata. Use
    /// [`EndpointBounds::accepts_response_rows`](super::metadata::EndpointBounds::accepts_response_rows)
    /// for optional preflight validation.
    pub async fn stock_chart_dividend_adjusted(
        &self,
        query: impl Into<StockChartEodQuery>,
    ) -> Result<Vec<StockChartAdjustedBar>> {
        self.execute(&stock_chart_dividend_adjusted(query.into()))
            .await
    }
}

/// Shared query parameters for the six fixed-interval intraday stock charts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StockChartIntradayQuery {
    symbol: Ticker,
    from: Option<Date>,
    to: Option<Date>,
    nonadjusted: Option<bool>,
    extended: Option<bool>,
}

impl StockChartIntradayQuery {
    /// Creates a query for one ticker without undocumented date or flag defaults.
    pub fn new(symbol: Ticker) -> Self {
        Self {
            symbol,
            from: None,
            to: None,
            nonadjusted: None,
            extended: None,
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

    /// Sets the optional provider `nonadjusted` flag, preserving explicit false.
    pub const fn with_nonadjusted(mut self, nonadjusted: bool) -> Self {
        self.nonadjusted = Some(nonadjusted);
        self
    }

    /// Sets the optional provider `extended` flag, preserving explicit false.
    pub const fn with_extended(mut self, extended: bool) -> Self {
        self.extended = Some(extended);
        self
    }

    /// Borrows the requested ticker.
    pub fn symbol(&self) -> &Ticker {
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

    /// Returns the optional provider `nonadjusted` flag.
    pub const fn nonadjusted(&self) -> Option<bool> {
        self.nonadjusted
    }

    /// Returns the optional provider `extended` flag.
    pub const fn extended(&self) -> Option<bool> {
        self.extended
    }
}

impl From<Ticker> for StockChartIntradayQuery {
    fn from(symbol: Ticker) -> Self {
        Self::new(symbol)
    }
}

impl From<&Ticker> for StockChartIntradayQuery {
    fn from(symbol: &Ticker) -> Self {
        Self::new(symbol.clone())
    }
}

impl QueryParameters for StockChartIntradayQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.required("symbol", &self.symbol);
        encoder.optional("from", self.from);
        encoder.optional("to", self.to);
        encoder.optional("nonadjusted", self.nonadjusted);
        encoder.optional("extended", self.extended);
    }
}

const STOCK_CHART_INTRADAY_METADATA: EndpointMetadata =
    EndpointMetadata::new().with_geography(GeographicAvailability::Worldwide);

/// Describes `GET historical-chart/1min` without binding a transport.
pub fn stock_chart_one_minute(
    query: StockChartIntradayQuery,
) -> EndpointSpec<StockChartIntradayQuery, Vec<StockChartIntradayBar>> {
    EndpointSpec::get("historical-chart/1min", "historical-chart/1min", query)
        .with_metadata(STOCK_CHART_INTRADAY_METADATA)
}

/// Describes `GET historical-chart/5min` without binding a transport.
pub fn stock_chart_five_minutes(
    query: StockChartIntradayQuery,
) -> EndpointSpec<StockChartIntradayQuery, Vec<StockChartIntradayBar>> {
    EndpointSpec::get("historical-chart/5min", "historical-chart/5min", query)
        .with_metadata(STOCK_CHART_INTRADAY_METADATA)
}

/// Describes `GET historical-chart/15min` without binding a transport.
pub fn stock_chart_fifteen_minutes(
    query: StockChartIntradayQuery,
) -> EndpointSpec<StockChartIntradayQuery, Vec<StockChartIntradayBar>> {
    EndpointSpec::get("historical-chart/15min", "historical-chart/15min", query)
        .with_metadata(STOCK_CHART_INTRADAY_METADATA)
}

/// Describes `GET historical-chart/30min` without binding a transport.
pub fn stock_chart_thirty_minutes(
    query: StockChartIntradayQuery,
) -> EndpointSpec<StockChartIntradayQuery, Vec<StockChartIntradayBar>> {
    EndpointSpec::get("historical-chart/30min", "historical-chart/30min", query)
        .with_metadata(STOCK_CHART_INTRADAY_METADATA)
}

/// Describes `GET historical-chart/1hour` without binding a transport.
pub fn stock_chart_one_hour(
    query: StockChartIntradayQuery,
) -> EndpointSpec<StockChartIntradayQuery, Vec<StockChartIntradayBar>> {
    EndpointSpec::get("historical-chart/1hour", "historical-chart/1hour", query)
        .with_metadata(STOCK_CHART_INTRADAY_METADATA)
}

/// Describes `GET historical-chart/4hour` without binding a transport.
pub fn stock_chart_four_hours(
    query: StockChartIntradayQuery,
) -> EndpointSpec<StockChartIntradayQuery, Vec<StockChartIntradayBar>> {
    EndpointSpec::get("historical-chart/4hour", "historical-chart/4hour", query)
        .with_metadata(STOCK_CHART_INTRADAY_METADATA)
}

impl Client {
    /// Retrieves one-minute worldwide intraday stock chart rows.
    pub async fn stock_chart_one_minute(
        &self,
        query: impl Into<StockChartIntradayQuery>,
    ) -> Result<Vec<StockChartIntradayBar>> {
        self.execute(&stock_chart_one_minute(query.into())).await
    }

    /// Retrieves five-minute worldwide intraday stock chart rows.
    pub async fn stock_chart_five_minutes(
        &self,
        query: impl Into<StockChartIntradayQuery>,
    ) -> Result<Vec<StockChartIntradayBar>> {
        self.execute(&stock_chart_five_minutes(query.into())).await
    }

    /// Retrieves fifteen-minute worldwide intraday stock chart rows.
    pub async fn stock_chart_fifteen_minutes(
        &self,
        query: impl Into<StockChartIntradayQuery>,
    ) -> Result<Vec<StockChartIntradayBar>> {
        self.execute(&stock_chart_fifteen_minutes(query.into()))
            .await
    }

    /// Retrieves thirty-minute worldwide intraday stock chart rows.
    pub async fn stock_chart_thirty_minutes(
        &self,
        query: impl Into<StockChartIntradayQuery>,
    ) -> Result<Vec<StockChartIntradayBar>> {
        self.execute(&stock_chart_thirty_minutes(query.into()))
            .await
    }

    /// Retrieves one-hour worldwide intraday stock chart rows.
    pub async fn stock_chart_one_hour(
        &self,
        query: impl Into<StockChartIntradayQuery>,
    ) -> Result<Vec<StockChartIntradayBar>> {
        self.execute(&stock_chart_one_hour(query.into())).await
    }

    /// Retrieves four-hour worldwide intraday stock chart rows.
    pub async fn stock_chart_four_hours(
        &self,
        query: impl Into<StockChartIntradayQuery>,
    ) -> Result<Vec<StockChartIntradayBar>> {
        self.execute(&stock_chart_four_hours(query.into())).await
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    fn pairs(query: &impl QueryParameters) -> Vec<(String, String)> {
        let mut pairs = Vec::new();
        query.encode(&mut QueryEncoder::new(&mut |name, value| {
            pairs.push((name.to_owned(), value.to_owned()));
        }));
        pairs
    }

    #[test]
    fn eod_dates_are_independent_and_encode_in_documented_order() {
        let symbol = Ticker::new("BRK.B / Class A").unwrap();
        let from = Date::from_str("2026-04-30").unwrap();
        let to = Date::from_str("2026-07-30").unwrap();

        let omitted: StockChartEodQuery = (&symbol).into();
        assert_eq!(omitted.symbol(), &symbol);
        assert_eq!(omitted.from(), None);
        assert_eq!(omitted.to(), None);
        assert_eq!(
            pairs(&omitted),
            [("symbol".to_owned(), "BRK.B / Class A".to_owned())]
        );

        let from_only = StockChartEodQuery::new(symbol.clone()).with_from(from);
        let to_only = StockChartEodQuery::new(symbol.clone()).with_to(to);
        assert_eq!(from_only.from(), Some(from));
        assert_eq!(from_only.to(), None);
        assert_eq!(to_only.from(), None);
        assert_eq!(to_only.to(), Some(to));

        assert_eq!(
            pairs(&StockChartEodQuery::new(symbol).with_from(from).with_to(to)),
            [
                ("symbol".to_owned(), "BRK.B / Class A".to_owned()),
                ("from".to_owned(), "2026-04-30".to_owned()),
                ("to".to_owned(), "2026-07-30".to_owned()),
            ]
        );
    }

    #[test]
    fn intraday_flags_preserve_omission_false_true_and_exact_order() {
        let symbol = Ticker::new("AAPL").unwrap();
        let from = Date::from_str("2024-01-01").unwrap();
        let to = Date::from_str("2024-03-01").unwrap();
        let omitted: StockChartIntradayQuery = symbol.clone().into();

        assert_eq!(omitted.symbol(), &symbol);
        assert_eq!(omitted.from(), None);
        assert_eq!(omitted.to(), None);
        assert_eq!(omitted.nonadjusted(), None);
        assert_eq!(omitted.extended(), None);
        assert_eq!(pairs(&omitted), [("symbol".to_owned(), "AAPL".to_owned())]);

        let query = StockChartIntradayQuery::new(symbol)
            .with_from(from)
            .with_to(to)
            .with_nonadjusted(false)
            .with_extended(true);
        assert_eq!(query.from(), Some(from));
        assert_eq!(query.to(), Some(to));
        assert_eq!(query.nonadjusted(), Some(false));
        assert_eq!(query.extended(), Some(true));
        assert_eq!(
            pairs(&query),
            [
                ("symbol".to_owned(), "AAPL".to_owned()),
                ("from".to_owned(), "2024-01-01".to_owned()),
                ("to".to_owned(), "2024-03-01".to_owned()),
                ("nonadjusted".to_owned(), "false".to_owned()),
                ("extended".to_owned(), "true".to_owned()),
            ]
        );
    }
}
