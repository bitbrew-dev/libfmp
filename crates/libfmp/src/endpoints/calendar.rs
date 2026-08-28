//! Calendar endpoint query contracts.

use crate::{
    endpoints::{QueryEncoder, QueryParameters},
    types::{Date, Limit, Page, Ticker},
};

macro_rules! symbol_limit_query {
    ($name:ident, $description:literal) => {
        #[doc = $description]
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $name {
            symbol: Ticker,
            limit: Option<Limit>,
        }

        impl $name {
            /// Creates a query for one ticker without an undocumented limit.
            pub fn new(symbol: Ticker) -> Self {
                Self {
                    symbol,
                    limit: None,
                }
            }

            /// Sets the optional provider result limit.
            pub const fn with_limit(mut self, limit: Limit) -> Self {
                self.limit = Some(limit);
                self
            }

            /// Borrows the requested ticker.
            pub const fn symbol(&self) -> &Ticker {
                &self.symbol
            }

            /// Returns the optional provider result limit.
            pub const fn limit(&self) -> Option<Limit> {
                self.limit
            }
        }

        impl From<Ticker> for $name {
            fn from(symbol: Ticker) -> Self {
                Self::new(symbol)
            }
        }

        impl From<&Ticker> for $name {
            fn from(symbol: &Ticker) -> Self {
                Self::new(symbol.clone())
            }
        }

        impl QueryParameters for $name {
            fn encode(&self, encoder: &mut QueryEncoder<'_>) {
                encoder.required("symbol", &self.symbol);
                encoder.optional("limit", self.limit);
            }
        }
    };
}

macro_rules! date_query {
    ($name:ident, $description:literal) => {
        #[doc = $description]
        #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
        pub struct $name {
            from: Option<Date>,
            to: Option<Date>,
        }

        impl $name {
            /// Creates a query without undocumented date defaults.
            pub const fn new() -> Self {
                Self {
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

            /// Returns the optional independent start date.
            pub const fn from(&self) -> Option<Date> {
                self.from
            }

            /// Returns the optional independent end date.
            pub const fn to(&self) -> Option<Date> {
                self.to
            }
        }

        impl QueryParameters for $name {
            fn encode(&self, encoder: &mut QueryEncoder<'_>) {
                encoder.optional("from", self.from);
                encoder.optional("to", self.to);
            }
        }
    };
}

macro_rules! date_page_query {
    ($name:ident, $description:literal) => {
        #[doc = $description]
        #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
        pub struct $name {
            from: Option<Date>,
            to: Option<Date>,
            page: Option<Page>,
        }

        impl $name {
            /// Creates a query without undocumented date or page defaults.
            pub const fn new() -> Self {
                Self {
                    from: None,
                    to: None,
                    page: None,
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

            /// Sets the optional provider page index.
            pub const fn with_page(mut self, page: Page) -> Self {
                self.page = Some(page);
                self
            }

            /// Returns the optional independent start date.
            pub const fn from(&self) -> Option<Date> {
                self.from
            }

            /// Returns the optional independent end date.
            pub const fn to(&self) -> Option<Date> {
                self.to
            }

            /// Returns the optional provider page index.
            pub const fn page(&self) -> Option<Page> {
                self.page
            }
        }

        impl QueryParameters for $name {
            fn encode(&self, encoder: &mut QueryEncoder<'_>) {
                encoder.optional("from", self.from);
                encoder.optional("to", self.to);
                encoder.optional("page", self.page);
            }
        }
    };
}

symbol_limit_query!(
    DividendsQuery,
    "Required ticker and optional limit for company dividend events."
);
date_page_query!(
    DividendsCalendarQuery,
    "Optional independent dates and page for the dividend calendar."
);

/// Required ticker and optional limit/report-time flag for company earnings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EarningsQuery {
    symbol: Ticker,
    limit: Option<Limit>,
    include_report_times: Option<bool>,
}

impl EarningsQuery {
    /// Creates a query for one ticker without undocumented optional defaults.
    pub fn new(symbol: Ticker) -> Self {
        Self {
            symbol,
            limit: None,
            include_report_times: None,
        }
    }

    /// Sets the optional provider result limit.
    pub const fn with_limit(mut self, limit: Limit) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Sets whether the provider should include report-time detail.
    pub const fn with_include_report_times(mut self, include_report_times: bool) -> Self {
        self.include_report_times = Some(include_report_times);
        self
    }

    /// Borrows the requested ticker.
    pub const fn symbol(&self) -> &Ticker {
        &self.symbol
    }

    /// Returns the optional provider result limit.
    pub const fn limit(&self) -> Option<Limit> {
        self.limit
    }

    /// Returns the optional report-time flag.
    pub const fn include_report_times(&self) -> Option<bool> {
        self.include_report_times
    }
}

impl From<Ticker> for EarningsQuery {
    fn from(symbol: Ticker) -> Self {
        Self::new(symbol)
    }
}

impl From<&Ticker> for EarningsQuery {
    fn from(symbol: &Ticker) -> Self {
        Self::new(symbol.clone())
    }
}

impl QueryParameters for EarningsQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.required("symbol", &self.symbol);
        encoder.optional("limit", self.limit);
        encoder.optional("includeReportTimes", self.include_report_times);
    }
}

/// Optional independent dates, page, and report-time flag for the earnings calendar.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EarningsCalendarQuery {
    from: Option<Date>,
    to: Option<Date>,
    page: Option<Page>,
    include_report_times: Option<bool>,
}

impl EarningsCalendarQuery {
    /// Creates a query without undocumented optional defaults.
    pub const fn new() -> Self {
        Self {
            from: None,
            to: None,
            page: None,
            include_report_times: None,
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

    /// Sets the optional provider page index.
    pub const fn with_page(mut self, page: Page) -> Self {
        self.page = Some(page);
        self
    }

    /// Sets whether the provider should include report-time detail.
    pub const fn with_include_report_times(mut self, include_report_times: bool) -> Self {
        self.include_report_times = Some(include_report_times);
        self
    }

    /// Returns the optional independent start date.
    pub const fn from(&self) -> Option<Date> {
        self.from
    }

    /// Returns the optional independent end date.
    pub const fn to(&self) -> Option<Date> {
        self.to
    }

    /// Returns the optional provider page index.
    pub const fn page(&self) -> Option<Page> {
        self.page
    }

    /// Returns the optional report-time flag.
    pub const fn include_report_times(&self) -> Option<bool> {
        self.include_report_times
    }
}

impl QueryParameters for EarningsCalendarQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.optional("from", self.from);
        encoder.optional("to", self.to);
        encoder.optional("page", self.page);
        encoder.optional("includeReportTimes", self.include_report_times);
    }
}

date_query!(
    IposCalendarQuery,
    "Optional independent dates for the worldwide IPO calendar."
);
date_query!(
    IposDisclosureQuery,
    "Optional independent dates for US IPO disclosures."
);
date_query!(
    IposProspectusQuery,
    "Optional independent dates for US IPO prospectuses."
);

symbol_limit_query!(
    StockSplitsQuery,
    "Required ticker and optional limit for company stock-split events."
);
date_page_query!(
    StockSplitsCalendarQuery,
    "Optional independent dates and page for the stock-splits calendar."
);

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
    fn symbol_queries_encode_required_symbol_then_optional_values() {
        let symbol = Ticker::new("AAPL").unwrap();
        let dividends = DividendsQuery::new(symbol.clone()).with_limit(Limit(100));
        assert_eq!(
            pairs(&dividends),
            [
                ("symbol".into(), "AAPL".into()),
                ("limit".into(), "100".into())
            ]
        );

        let earnings = EarningsQuery::new(symbol.clone())
            .with_limit(Limit(100))
            .with_include_report_times(false);
        assert_eq!(
            pairs(&earnings),
            [
                ("symbol".into(), "AAPL".into()),
                ("limit".into(), "100".into()),
                ("includeReportTimes".into(), "false".into()),
            ]
        );

        let splits = StockSplitsQuery::from(&symbol).with_limit(Limit(100));
        assert_eq!(
            pairs(&splits),
            [
                ("symbol".into(), "AAPL".into()),
                ("limit".into(), "100".into())
            ]
        );
    }

    #[test]
    fn date_queries_preserve_independent_omission_and_exact_order() {
        let from = Date::from_str("2026-03-06").unwrap();
        let to = Date::from_str("2026-06-06").unwrap();

        let dividends = DividendsCalendarQuery::new()
            .with_from(from)
            .with_to(to)
            .with_page(Page(0));
        assert_eq!(
            pairs(&dividends),
            [
                ("from".into(), "2026-03-06".into()),
                ("to".into(), "2026-06-06".into()),
                ("page".into(), "0".into()),
            ]
        );

        let earnings = EarningsCalendarQuery::new()
            .with_from(from)
            .with_to(to)
            .with_page(Page(0))
            .with_include_report_times(true);
        assert_eq!(
            pairs(&earnings),
            [
                ("from".into(), "2026-03-06".into()),
                ("to".into(), "2026-06-06".into()),
                ("page".into(), "0".into()),
                ("includeReportTimes".into(), "true".into()),
            ]
        );

        let ipo = IposCalendarQuery::new().with_from(from);
        assert_eq!(ipo.to(), None);
        assert_eq!(pairs(&ipo), [("from".into(), "2026-03-06".into())]);

        let disclosure = IposDisclosureQuery::new().with_to(to);
        assert_eq!(disclosure.from(), None);
        assert_eq!(pairs(&disclosure), [("to".into(), "2026-06-06".into())]);

        let prospectus = IposProspectusQuery::new().with_from(from).with_to(to);
        assert_eq!(
            pairs(&prospectus),
            [
                ("from".into(), "2026-03-06".into()),
                ("to".into(), "2026-06-06".into()),
            ]
        );

        let splits = StockSplitsCalendarQuery::new()
            .with_to(to)
            .with_page(Page(0));
        assert_eq!(splits.from(), None);
        assert_eq!(
            pairs(&splits),
            [
                ("to".into(), "2026-06-06".into()),
                ("page".into(), "0".into())
            ]
        );
    }

    #[test]
    fn report_time_flags_distinguish_omitted_false_and_true() {
        let symbol = Ticker::new("AAPL").unwrap();
        assert_eq!(
            pairs(&EarningsQuery::new(symbol.clone())),
            [("symbol".into(), "AAPL".into())]
        );
        assert_eq!(
            pairs(&EarningsQuery::new(symbol).with_include_report_times(false)),
            [
                ("symbol".into(), "AAPL".into()),
                ("includeReportTimes".into(), "false".into())
            ]
        );
        assert!(pairs(&EarningsCalendarQuery::new()).is_empty());
        assert_eq!(
            pairs(&EarningsCalendarQuery::new().with_include_report_times(true)),
            [("includeReportTimes".into(), "true".into())]
        );
    }
}
