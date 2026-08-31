//! Technical-indicator query contracts.

use crate::{
    Client, Result,
    endpoints::{QueryEncoder, QueryParameters},
    query::{ChartTimeframe, PeriodLength},
    responses::technical_indicators::{
        AverageDirectionalIndexBar, DoubleExponentialMovingAverageBar, ExponentialMovingAverageBar,
        RelativeStrengthIndexBar, SimpleMovingAverageBar, StandardDeviationBar,
        TripleExponentialMovingAverageBar, WeightedMovingAverageBar, WilliamsBar,
    },
    types::{Date, Ticker},
};

use super::{
    EndpointSpec,
    metadata::{EndpointMetadata, GeographicAvailability},
};

/// Shared query parameters for the nine technical-indicator endpoints.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TechnicalIndicatorQuery {
    symbol: Ticker,
    period_length: PeriodLength,
    timeframe: ChartTimeframe,
    from: Option<Date>,
    to: Option<Date>,
}

impl TechnicalIndicatorQuery {
    /// Creates a technical-indicator query without undocumented date defaults.
    pub const fn new(
        symbol: Ticker,
        period_length: PeriodLength,
        timeframe: ChartTimeframe,
    ) -> Self {
        Self {
            symbol,
            period_length,
            timeframe,
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
    pub const fn symbol(&self) -> &Ticker {
        &self.symbol
    }

    /// Returns the strictly positive look-back period length.
    pub const fn period_length(&self) -> PeriodLength {
        self.period_length
    }

    /// Returns the requested chart timeframe.
    pub const fn timeframe(&self) -> ChartTimeframe {
        self.timeframe
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

impl QueryParameters for TechnicalIndicatorQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.required("symbol", &self.symbol);
        encoder.required("periodLength", self.period_length);
        encoder.required("timeframe", self.timeframe);
        encoder.optional("from", self.from);
        encoder.optional("to", self.to);
    }
}

const WORLDWIDE: EndpointMetadata =
    EndpointMetadata::new().with_geography(GeographicAvailability::Worldwide);

/// Describes `GET technical-indicators/sma` without binding a transport.
pub fn simple_moving_average(
    query: TechnicalIndicatorQuery,
) -> EndpointSpec<TechnicalIndicatorQuery, Vec<SimpleMovingAverageBar>> {
    EndpointSpec::get(
        "technical-indicators/sma",
        "technical-indicators/sma",
        query,
    )
    .with_metadata(WORLDWIDE)
}

/// Describes `GET technical-indicators/ema` without binding a transport.
pub fn exponential_moving_average(
    query: TechnicalIndicatorQuery,
) -> EndpointSpec<TechnicalIndicatorQuery, Vec<ExponentialMovingAverageBar>> {
    EndpointSpec::get(
        "technical-indicators/ema",
        "technical-indicators/ema",
        query,
    )
    .with_metadata(WORLDWIDE)
}

/// Describes `GET technical-indicators/wma` without binding a transport.
pub fn weighted_moving_average(
    query: TechnicalIndicatorQuery,
) -> EndpointSpec<TechnicalIndicatorQuery, Vec<WeightedMovingAverageBar>> {
    EndpointSpec::get(
        "technical-indicators/wma",
        "technical-indicators/wma",
        query,
    )
    .with_metadata(WORLDWIDE)
}

/// Describes `GET technical-indicators/dema` without binding a transport.
pub fn double_exponential_moving_average(
    query: TechnicalIndicatorQuery,
) -> EndpointSpec<TechnicalIndicatorQuery, Vec<DoubleExponentialMovingAverageBar>> {
    EndpointSpec::get(
        "technical-indicators/dema",
        "technical-indicators/dema",
        query,
    )
    .with_metadata(WORLDWIDE)
}

/// Describes `GET technical-indicators/tema` without binding a transport.
pub fn triple_exponential_moving_average(
    query: TechnicalIndicatorQuery,
) -> EndpointSpec<TechnicalIndicatorQuery, Vec<TripleExponentialMovingAverageBar>> {
    EndpointSpec::get(
        "technical-indicators/tema",
        "technical-indicators/tema",
        query,
    )
    .with_metadata(WORLDWIDE)
}

/// Describes `GET technical-indicators/rsi` without binding a transport.
pub fn relative_strength_index(
    query: TechnicalIndicatorQuery,
) -> EndpointSpec<TechnicalIndicatorQuery, Vec<RelativeStrengthIndexBar>> {
    EndpointSpec::get(
        "technical-indicators/rsi",
        "technical-indicators/rsi",
        query,
    )
    .with_metadata(WORLDWIDE)
}

/// Describes `GET technical-indicators/standarddeviation` without binding a transport.
pub fn standard_deviation(
    query: TechnicalIndicatorQuery,
) -> EndpointSpec<TechnicalIndicatorQuery, Vec<StandardDeviationBar>> {
    EndpointSpec::get(
        "technical-indicators/standarddeviation",
        "technical-indicators/standarddeviation",
        query,
    )
    .with_metadata(WORLDWIDE)
}

/// Describes `GET technical-indicators/williams` without binding a transport.
pub fn williams(
    query: TechnicalIndicatorQuery,
) -> EndpointSpec<TechnicalIndicatorQuery, Vec<WilliamsBar>> {
    EndpointSpec::get(
        "technical-indicators/williams",
        "technical-indicators/williams",
        query,
    )
    .with_metadata(WORLDWIDE)
}

/// Describes `GET technical-indicators/adx` without binding a transport.
pub fn average_directional_index(
    query: TechnicalIndicatorQuery,
) -> EndpointSpec<TechnicalIndicatorQuery, Vec<AverageDirectionalIndexBar>> {
    EndpointSpec::get(
        "technical-indicators/adx",
        "technical-indicators/adx",
        query,
    )
    .with_metadata(WORLDWIDE)
}

impl Client {
    /// Retrieves worldwide simple-moving-average bars.
    pub async fn simple_moving_average(
        &self,
        query: TechnicalIndicatorQuery,
    ) -> Result<Vec<SimpleMovingAverageBar>> {
        self.execute(&simple_moving_average(query)).await
    }

    /// Retrieves worldwide exponential-moving-average bars.
    pub async fn exponential_moving_average(
        &self,
        query: TechnicalIndicatorQuery,
    ) -> Result<Vec<ExponentialMovingAverageBar>> {
        self.execute(&exponential_moving_average(query)).await
    }

    /// Retrieves worldwide weighted-moving-average bars.
    pub async fn weighted_moving_average(
        &self,
        query: TechnicalIndicatorQuery,
    ) -> Result<Vec<WeightedMovingAverageBar>> {
        self.execute(&weighted_moving_average(query)).await
    }

    /// Retrieves worldwide double-exponential-moving-average bars.
    pub async fn double_exponential_moving_average(
        &self,
        query: TechnicalIndicatorQuery,
    ) -> Result<Vec<DoubleExponentialMovingAverageBar>> {
        self.execute(&double_exponential_moving_average(query))
            .await
    }

    /// Retrieves worldwide triple-exponential-moving-average bars.
    pub async fn triple_exponential_moving_average(
        &self,
        query: TechnicalIndicatorQuery,
    ) -> Result<Vec<TripleExponentialMovingAverageBar>> {
        self.execute(&triple_exponential_moving_average(query))
            .await
    }

    /// Retrieves worldwide relative-strength-index bars.
    pub async fn relative_strength_index(
        &self,
        query: TechnicalIndicatorQuery,
    ) -> Result<Vec<RelativeStrengthIndexBar>> {
        self.execute(&relative_strength_index(query)).await
    }

    /// Retrieves worldwide standard-deviation bars.
    pub async fn standard_deviation(
        &self,
        query: TechnicalIndicatorQuery,
    ) -> Result<Vec<StandardDeviationBar>> {
        self.execute(&standard_deviation(query)).await
    }

    /// Retrieves worldwide Williams bars.
    pub async fn williams(&self, query: TechnicalIndicatorQuery) -> Result<Vec<WilliamsBar>> {
        self.execute(&williams(query)).await
    }

    /// Retrieves worldwide average-directional-index bars.
    pub async fn average_directional_index(
        &self,
        query: TechnicalIndicatorQuery,
    ) -> Result<Vec<AverageDirectionalIndexBar>> {
        self.execute(&average_directional_index(query)).await
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
    fn required_values_and_independent_dates_encode_in_exact_order() {
        let from = Date::from_str("2026-06-01").unwrap();
        let to = Date::from_str("2026-03-01").unwrap();
        let query = TechnicalIndicatorQuery::new(
            Ticker::new("BRK.B / Class A").unwrap(),
            PeriodLength::new(u32::MAX).unwrap(),
            ChartTimeframe::OneMinute,
        )
        .with_from(from)
        .with_to(to);

        assert_eq!(
            pairs(&query),
            [
                ("symbol".to_owned(), "BRK.B / Class A".to_owned()),
                ("periodLength".to_owned(), u32::MAX.to_string()),
                ("timeframe".to_owned(), "1min".to_owned()),
                ("from".to_owned(), "2026-06-01".to_owned()),
                ("to".to_owned(), "2026-03-01".to_owned()),
            ]
        );
    }

    #[test]
    fn optional_dates_are_omitted_independently() {
        let query = TechnicalIndicatorQuery::new(
            Ticker::new("AAPL").unwrap(),
            PeriodLength::new(10).unwrap(),
            ChartTimeframe::OneDay,
        );
        assert_eq!(
            pairs(&query),
            [
                ("symbol".to_owned(), "AAPL".to_owned()),
                ("periodLength".to_owned(), "10".to_owned()),
                ("timeframe".to_owned(), "1day".to_owned()),
            ]
        );
    }
}
