//! Environmental, social, and governance endpoint contracts.
//!
//! A future Python binding reserves the matching `FmpClient` method names
//! `esg_disclosures`, `esg_ratings`, and `esg_benchmark`, with response models
//! under `fmp.esg`. This crate does not implement those Python bindings.

use crate::{
    Client, Result,
    endpoints::{
        EndpointSpec, QueryEncoder, QueryParameters,
        metadata::{EndpointMetadata, GeographicAvailability},
    },
    responses::esg::{EsgBenchmark, EsgDisclosure, EsgRating},
    types::{BenchmarkYear, Ticker},
};

/// Required company symbol shared by ESG disclosure and rating routes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EsgSymbolQuery {
    symbol: Ticker,
}

impl EsgSymbolQuery {
    /// Creates an ESG query for one company.
    pub const fn new(symbol: Ticker) -> Self {
        Self { symbol }
    }

    /// Borrows the requested company symbol.
    pub const fn symbol(&self) -> &Ticker {
        &self.symbol
    }
}

impl From<Ticker> for EsgSymbolQuery {
    fn from(symbol: Ticker) -> Self {
        Self::new(symbol)
    }
}

impl From<&Ticker> for EsgSymbolQuery {
    fn from(symbol: &Ticker) -> Self {
        Self::new(symbol.clone())
    }
}

impl QueryParameters for EsgSymbolQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.required("symbol", &self.symbol);
    }
}

/// Independently optional benchmark year, preserving the provider string.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EsgBenchmarkQuery {
    year: Option<BenchmarkYear>,
}

impl EsgBenchmarkQuery {
    /// Creates a benchmark query without an undocumented year default.
    pub const fn new() -> Self {
        Self { year: None }
    }

    /// Sets the optional provider year string.
    pub fn with_year(mut self, year: BenchmarkYear) -> Self {
        self.year = Some(year);
        self
    }

    /// Borrows the optional provider year string.
    pub const fn year(&self) -> Option<&BenchmarkYear> {
        self.year.as_ref()
    }
}

impl QueryParameters for EsgBenchmarkQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.optional("year", self.year.as_ref());
    }
}

const US_ONLY: EndpointMetadata =
    EndpointMetadata::new().with_geography(GeographicAvailability::UsOnly);

/// Describes `GET esg-disclosures` without binding a transport.
pub fn esg_disclosures(query: EsgSymbolQuery) -> EndpointSpec<EsgSymbolQuery, Vec<EsgDisclosure>> {
    EndpointSpec::get("esg-disclosures", "esg-disclosures", query).with_metadata(US_ONLY)
}

/// Describes `GET esg-ratings` without binding a transport.
pub fn esg_ratings(query: EsgSymbolQuery) -> EndpointSpec<EsgSymbolQuery, Vec<EsgRating>> {
    EndpointSpec::get("esg-ratings", "esg-ratings", query).with_metadata(US_ONLY)
}

/// Describes `GET esg-benchmark` without binding a transport.
pub fn esg_benchmark(
    query: EsgBenchmarkQuery,
) -> EndpointSpec<EsgBenchmarkQuery, Vec<EsgBenchmark>> {
    EndpointSpec::get("esg-benchmark", "esg-benchmark", query).with_metadata(US_ONLY)
}

impl Client {
    /// Retrieves ESG disclosure filings for one company.
    pub async fn esg_disclosures(
        &self,
        query: impl Into<EsgSymbolQuery>,
    ) -> Result<Vec<EsgDisclosure>> {
        self.execute(&esg_disclosures(query.into())).await
    }

    /// Retrieves ESG ratings for one company.
    pub async fn esg_ratings(&self, query: impl Into<EsgSymbolQuery>) -> Result<Vec<EsgRating>> {
        self.execute(&esg_ratings(query.into())).await
    }

    /// Retrieves sector ESG benchmarks, optionally for one provider year string.
    pub async fn esg_benchmark(&self, query: EsgBenchmarkQuery) -> Result<Vec<EsgBenchmark>> {
        self.execute(&esg_benchmark(query)).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encoded(query: &impl QueryParameters) -> Vec<(String, String)> {
        let mut pairs = Vec::new();
        let mut visitor = |name: &str, value: &str| {
            pairs.push((name.to_owned(), value.to_owned()));
        };
        query.encode(&mut QueryEncoder::new(&mut visitor));
        pairs
    }

    #[test]
    fn queries_encode_exact_documented_keys_order_and_omissions() {
        let symbol = Ticker::new("BRK.B / Class A").unwrap();
        assert_eq!(
            encoded(&EsgSymbolQuery::from(&symbol)),
            [("symbol".into(), "BRK.B / Class A".into())]
        );
        assert!(encoded(&EsgBenchmarkQuery::new()).is_empty());
        assert_eq!(
            encoded(&EsgBenchmarkQuery::new().with_year(BenchmarkYear::new("FY 2024/25").unwrap())),
            [("year".into(), "FY 2024/25".into())]
        );
    }
}
