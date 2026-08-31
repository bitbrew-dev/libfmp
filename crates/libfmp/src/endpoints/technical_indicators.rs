//! Technical-indicator query contracts.

use crate::{
    endpoints::{QueryEncoder, QueryParameters},
    query::{ChartTimeframe, PeriodLength},
    types::{Date, Ticker},
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
