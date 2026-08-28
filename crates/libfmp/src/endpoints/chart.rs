//! Shared query contracts for stock chart endpoints.
//!
//! Endpoint descriptors and client methods are intentionally added by later
//! chart implementation branches.

use crate::{
    endpoints::{QueryEncoder, QueryParameters},
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
