//! ETF and mutual-fund endpoint query contracts.

use crate::{
    endpoints::{QueryEncoder, QueryParameters},
    query::{Quarter, Year},
    types::{Cik, SearchTerm, Ticker},
};

macro_rules! required_symbol_query {
    ($name:ident, $description:literal) => {
        #[doc = $description]
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $name {
            symbol: Ticker,
        }

        impl $name {
            /// Creates a query for one ticker.
            pub const fn new(symbol: Ticker) -> Self {
                Self { symbol }
            }

            /// Borrows the requested ticker.
            pub const fn symbol(&self) -> &Ticker {
                &self.symbol
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
            }
        }
    };
}

required_symbol_query!(
    EtfHoldingsQuery,
    "Required fund ticker for retrieving ETF or mutual-fund holdings."
);
required_symbol_query!(
    EtfInfoQuery,
    "Required fund ticker for retrieving ETF or mutual-fund information."
);
required_symbol_query!(
    EtfCountryWeightingsQuery,
    "Required fund ticker for retrieving country allocation."
);
required_symbol_query!(
    EtfAssetExposureQuery,
    "Required asset ticker for discovering ETF exposure."
);
required_symbol_query!(
    EtfSectorWeightingsQuery,
    "Required fund ticker for retrieving sector allocation."
);
required_symbol_query!(
    LatestFundDisclosureHoldersQuery,
    "Required asset ticker for retrieving the latest disclosed fund holders."
);

/// Required fund and reporting period plus an optional filer CIK.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FundDisclosureQuery {
    symbol: Ticker,
    year: Year,
    quarter: Quarter,
    cik: Option<Cik>,
}

impl FundDisclosureQuery {
    /// Creates a disclosure query without an undocumented CIK default.
    pub const fn new(symbol: Ticker, year: Year, quarter: Quarter) -> Self {
        Self {
            symbol,
            year,
            quarter,
            cik: None,
        }
    }

    /// Restricts the query to one provider filer CIK.
    pub fn with_cik(mut self, cik: Cik) -> Self {
        self.cik = Some(cik);
        self
    }

    /// Borrows the requested fund ticker.
    pub const fn symbol(&self) -> &Ticker {
        &self.symbol
    }

    /// Returns the required provider query year.
    pub const fn year(&self) -> Year {
        self.year
    }

    /// Returns the required provider query quarter.
    pub const fn quarter(&self) -> Quarter {
        self.quarter
    }

    /// Borrows the optional filer CIK.
    pub const fn cik(&self) -> Option<&Cik> {
        self.cik.as_ref()
    }
}

impl QueryParameters for FundDisclosureQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.required("symbol", &self.symbol);
        encoder.required("year", self.year);
        encoder.required("quarter", self.quarter);
        encoder.optional("cik", self.cik.as_ref());
    }
}

/// Required representation-preserving fund or ETF name search term.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FundDisclosureHolderSearchQuery {
    name: SearchTerm,
}

impl FundDisclosureHolderSearchQuery {
    /// Creates a disclosure-holder name search.
    pub const fn new(name: SearchTerm) -> Self {
        Self { name }
    }

    /// Borrows the exact requested name.
    pub const fn name(&self) -> &SearchTerm {
        &self.name
    }
}

impl From<SearchTerm> for FundDisclosureHolderSearchQuery {
    fn from(name: SearchTerm) -> Self {
        Self::new(name)
    }
}

impl From<&SearchTerm> for FundDisclosureHolderSearchQuery {
    fn from(name: &SearchTerm) -> Self {
        Self::new(name.clone())
    }
}

impl QueryParameters for FundDisclosureHolderSearchQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.required("name", &self.name);
    }
}

/// Required fund ticker and optional filer CIK for available disclosure dates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FundDisclosureDatesQuery {
    symbol: Ticker,
    cik: Option<Cik>,
}

impl FundDisclosureDatesQuery {
    /// Creates a query without an undocumented CIK default.
    pub const fn new(symbol: Ticker) -> Self {
        Self { symbol, cik: None }
    }

    /// Restricts the query to one provider filer CIK.
    pub fn with_cik(mut self, cik: Cik) -> Self {
        self.cik = Some(cik);
        self
    }

    /// Borrows the requested fund ticker.
    pub const fn symbol(&self) -> &Ticker {
        &self.symbol
    }

    /// Borrows the optional filer CIK.
    pub const fn cik(&self) -> Option<&Cik> {
        self.cik.as_ref()
    }
}

impl From<Ticker> for FundDisclosureDatesQuery {
    fn from(symbol: Ticker) -> Self {
        Self::new(symbol)
    }
}

impl From<&Ticker> for FundDisclosureDatesQuery {
    fn from(symbol: &Ticker) -> Self {
        Self::new(symbol.clone())
    }
}

impl QueryParameters for FundDisclosureDatesQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.required("symbol", &self.symbol);
        encoder.optional("cik", self.cik.as_ref());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pairs(query: &impl QueryParameters) -> Vec<(String, String)> {
        let mut pairs = Vec::new();
        let mut visitor = |name: &str, value: &str| {
            pairs.push((name.to_owned(), value.to_owned()));
        };
        query.encode(&mut QueryEncoder::new(&mut visitor));
        pairs
    }

    #[test]
    fn symbol_queries_emit_the_exact_only_pair() {
        let symbol = Ticker::new("BRK.B / Class A").unwrap();

        macro_rules! assert_symbol_query {
            ($query:ty) => {
                assert_eq!(
                    pairs(&<$query>::from(&symbol)),
                    [("symbol".to_owned(), "BRK.B / Class A".to_owned())]
                );
            };
        }

        assert_symbol_query!(EtfHoldingsQuery);
        assert_symbol_query!(EtfInfoQuery);
        assert_symbol_query!(EtfCountryWeightingsQuery);
        assert_symbol_query!(EtfAssetExposureQuery);
        assert_symbol_query!(EtfSectorWeightingsQuery);
        assert_symbol_query!(LatestFundDisclosureHoldersQuery);
    }

    #[test]
    fn disclosures_emit_required_values_then_optional_cik() {
        let base =
            FundDisclosureQuery::new(Ticker::new("000089.SZ").unwrap(), Year(0), Quarter::Q4);
        assert_eq!(
            pairs(&base),
            [
                ("symbol".to_owned(), "000089.SZ".to_owned()),
                ("year".to_owned(), "0".to_owned()),
                ("quarter".to_owned(), "4".to_owned()),
            ]
        );

        let with_cik = base.with_cik(Cik::new("0000857489").unwrap());
        assert_eq!(
            pairs(&with_cik),
            [
                ("symbol".to_owned(), "000089.SZ".to_owned()),
                ("year".to_owned(), "0".to_owned()),
                ("quarter".to_owned(), "4".to_owned()),
                ("cik".to_owned(), "0000857489".to_owned()),
            ]
        );
    }

    #[test]
    fn search_and_dates_preserve_exact_values_order_and_omission() {
        let name = SearchTerm::new("Federated Hermes Government Income Securities, Inc.").unwrap();
        assert_eq!(
            pairs(&FundDisclosureHolderSearchQuery::from(&name)),
            [("name".to_owned(), name.to_string())]
        );

        let dates = FundDisclosureDatesQuery::new(Ticker::new("VWO").unwrap());
        assert_eq!(pairs(&dates), [("symbol".to_owned(), "VWO".to_owned())]);
        assert_eq!(
            pairs(&dates.with_cik(Cik::new("0000036405").unwrap())),
            [
                ("symbol".to_owned(), "VWO".to_owned()),
                ("cik".to_owned(), "0000036405".to_owned()),
            ]
        );
    }
}
