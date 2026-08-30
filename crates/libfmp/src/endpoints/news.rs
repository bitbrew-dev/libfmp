//! News endpoint query contracts.

use crate::{
    endpoints::{QueryEncoder, QueryParameters},
    types::{Date, Limit, Page, TickerList},
};

/// Optional page and limit for Financial Modeling Prep articles.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FmpArticlesQuery {
    page: Option<Page>,
    limit: Option<Limit>,
}

impl FmpArticlesQuery {
    /// Creates a query without undocumented page or limit defaults.
    pub const fn new() -> Self {
        Self {
            page: None,
            limit: None,
        }
    }

    /// Sets the optional provider page index.
    pub const fn with_page(mut self, page: Page) -> Self {
        self.page = Some(page);
        self
    }

    /// Sets the optional provider result limit.
    pub const fn with_limit(mut self, limit: Limit) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Returns the optional provider page index.
    pub const fn page(&self) -> Option<Page> {
        self.page
    }

    /// Returns the optional provider result limit.
    pub const fn limit(&self) -> Option<Limit> {
        self.limit
    }
}

impl QueryParameters for FmpArticlesQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.optional("page", self.page);
        encoder.optional("limit", self.limit);
    }
}

macro_rules! latest_news_query {
    ($name:ident, $description:literal) => {
        #[doc = $description]
        #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
        pub struct $name {
            from: Option<Date>,
            to: Option<Date>,
            page: Option<Page>,
            limit: Option<Limit>,
        }

        impl $name {
            /// Creates a query without undocumented date, page, or limit defaults.
            pub const fn new() -> Self {
                Self {
                    from: None,
                    to: None,
                    page: None,
                    limit: None,
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

            /// Sets the optional provider result limit.
            pub const fn with_limit(mut self, limit: Limit) -> Self {
                self.limit = Some(limit);
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

            /// Returns the optional provider result limit.
            pub const fn limit(&self) -> Option<Limit> {
                self.limit
            }
        }

        impl QueryParameters for $name {
            fn encode(&self, encoder: &mut QueryEncoder<'_>) {
                encoder.optional("from", self.from);
                encoder.optional("to", self.to);
                encoder.optional("page", self.page);
                encoder.optional("limit", self.limit);
            }
        }
    };
}

latest_news_query!(
    LatestGeneralNewsQuery,
    "Optional filters and pagination for the latest general news."
);
latest_news_query!(
    LatestPressReleasesQuery,
    "Optional filters and pagination for the latest press releases."
);
latest_news_query!(
    LatestStockNewsQuery,
    "Optional filters and pagination for the latest stock news."
);
latest_news_query!(
    LatestCryptoNewsQuery,
    "Optional filters and pagination for the latest cryptocurrency news."
);
latest_news_query!(
    LatestForexNewsQuery,
    "Optional filters and pagination for the latest foreign-exchange news."
);

macro_rules! search_news_query {
    ($name:ident, $description:literal) => {
        #[doc = $description]
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $name {
            symbols: TickerList,
            from: Option<Date>,
            to: Option<Date>,
            page: Option<Page>,
            limit: Option<Limit>,
        }

        impl $name {
            /// Creates a query for a non-empty ticker list without optional defaults.
            pub fn new(symbols: TickerList) -> Self {
                Self {
                    symbols,
                    from: None,
                    to: None,
                    page: None,
                    limit: None,
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

            /// Sets the optional provider result limit.
            pub const fn with_limit(mut self, limit: Limit) -> Self {
                self.limit = Some(limit);
                self
            }

            /// Borrows the required tickers in provider request order.
            pub const fn symbols(&self) -> &TickerList {
                &self.symbols
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

            /// Returns the optional provider result limit.
            pub const fn limit(&self) -> Option<Limit> {
                self.limit
            }
        }

        impl From<TickerList> for $name {
            fn from(symbols: TickerList) -> Self {
                Self::new(symbols)
            }
        }

        impl From<&TickerList> for $name {
            fn from(symbols: &TickerList) -> Self {
                Self::new(symbols.clone())
            }
        }

        impl QueryParameters for $name {
            fn encode(&self, encoder: &mut QueryEncoder<'_>) {
                encoder.required("symbols", &self.symbols);
                encoder.optional("from", self.from);
                encoder.optional("to", self.to);
                encoder.optional("page", self.page);
                encoder.optional("limit", self.limit);
            }
        }
    };
}

search_news_query!(
    SearchPressReleasesQuery,
    "Required tickers, optional filters, and pagination for press-release search."
);
search_news_query!(
    SearchStockNewsQuery,
    "Required tickers, optional filters, and pagination for stock-news search."
);
search_news_query!(
    SearchCryptoNewsQuery,
    "Required tickers, optional filters, and pagination for cryptocurrency-news search."
);
search_news_query!(
    SearchForexNewsQuery,
    "Required tickers, optional filters, and pagination for foreign-exchange-news search."
);

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use crate::types::Ticker;

    fn pairs(query: &impl QueryParameters) -> Vec<(String, String)> {
        let mut pairs = Vec::new();
        query.encode(&mut QueryEncoder::new(&mut |name, value| {
            pairs.push((name.to_owned(), value.to_owned()));
        }));
        pairs
    }

    #[test]
    fn fmp_articles_encodes_page_then_limit_and_preserves_omission() {
        assert!(pairs(&FmpArticlesQuery::new()).is_empty());
        let query = FmpArticlesQuery::new()
            .with_page(Page(0))
            .with_limit(Limit(0));
        assert_eq!(
            pairs(&query),
            [
                ("page".to_owned(), "0".to_owned()),
                ("limit".to_owned(), "0".to_owned()),
            ]
        );
    }

    #[test]
    fn all_latest_queries_encode_from_to_page_limit_in_order() {
        let from = Date::from_str("2026-01-27").unwrap();
        let to = Date::from_str("2026-04-28").unwrap();
        let expected = [
            ("from".to_owned(), "2026-01-27".to_owned()),
            ("to".to_owned(), "2026-04-28".to_owned()),
            ("page".to_owned(), "0".to_owned()),
            ("limit".to_owned(), "20".to_owned()),
        ];

        macro_rules! assert_latest {
            ($query:ty) => {
                let query = <$query>::new()
                    .with_from(from)
                    .with_to(to)
                    .with_page(Page(0))
                    .with_limit(Limit(20));
                assert_eq!(pairs(&query), expected);
                assert!(pairs(&<$query>::new()).is_empty());
            };
        }

        assert_latest!(LatestGeneralNewsQuery);
        assert_latest!(LatestPressReleasesQuery);
        assert_latest!(LatestStockNewsQuery);
        assert_latest!(LatestCryptoNewsQuery);
        assert_latest!(LatestForexNewsQuery);

        assert_eq!(
            pairs(&LatestGeneralNewsQuery::new().with_from(from)),
            [("from".to_owned(), "2026-01-27".to_owned())]
        );
        assert_eq!(
            pairs(&LatestGeneralNewsQuery::new().with_to(to)),
            [("to".to_owned(), "2026-04-28".to_owned())]
        );
    }

    #[test]
    fn all_search_queries_encode_symbols_then_from_to_page_limit() {
        let symbols = TickerList::new(vec![
            Ticker::new("AAPL").unwrap(),
            Ticker::new("MSFT").unwrap(),
        ])
        .unwrap();
        let from = Date::from_str("2026-01-27").unwrap();
        let to = Date::from_str("2026-04-28").unwrap();
        let expected = [
            ("symbols".to_owned(), "AAPL,MSFT".to_owned()),
            ("from".to_owned(), "2026-01-27".to_owned()),
            ("to".to_owned(), "2026-04-28".to_owned()),
            ("page".to_owned(), "0".to_owned()),
            ("limit".to_owned(), "20".to_owned()),
        ];

        macro_rules! assert_search {
            ($query:ty) => {
                let query = <$query>::new(symbols.clone())
                    .with_from(from)
                    .with_to(to)
                    .with_page(Page(0))
                    .with_limit(Limit(20));
                assert_eq!(pairs(&query), expected);
                assert_eq!(
                    pairs(&<$query>::new(symbols.clone())),
                    [("symbols".to_owned(), "AAPL,MSFT".to_owned())]
                );
            };
        }

        assert_search!(SearchPressReleasesQuery);
        assert_search!(SearchStockNewsQuery);
        assert_search!(SearchCryptoNewsQuery);
        assert_search!(SearchForexNewsQuery);

        assert_eq!(
            pairs(&SearchStockNewsQuery::new(symbols.clone()).with_from(from)),
            [
                ("symbols".to_owned(), "AAPL,MSFT".to_owned()),
                ("from".to_owned(), "2026-01-27".to_owned()),
            ]
        );
        assert_eq!(
            pairs(&SearchStockNewsQuery::new(symbols).with_to(to)),
            [
                ("symbols".to_owned(), "AAPL,MSFT".to_owned()),
                ("to".to_owned(), "2026-04-28".to_owned()),
            ]
        );
    }
}
