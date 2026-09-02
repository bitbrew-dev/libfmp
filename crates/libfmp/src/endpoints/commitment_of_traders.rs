//! Commitment of Traders endpoint contracts.

use crate::{
    Client, Result,
    endpoints::{
        EndpointSpec, QueryEncoder, QueryParameters,
        metadata::{EndpointBounds, EndpointMetadata},
    },
    responses::commitment_of_traders::{CotAnalysis, CotReport, CotReportListing},
    types::{Date, Ticker},
};

/// Independently optional filters shared by the COT report and analysis routes.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CotQuery {
    symbol: Option<Ticker>,
    from: Option<Date>,
    to: Option<Date>,
}

impl CotQuery {
    /// Creates an unfiltered query.
    pub const fn new() -> Self {
        Self {
            symbol: None,
            from: None,
            to: None,
        }
    }

    /// Filters by a provider COT symbol.
    pub fn with_symbol(mut self, symbol: Ticker) -> Self {
        self.symbol = Some(symbol);
        self
    }

    /// Sets the optional inclusive start date.
    pub const fn with_from(mut self, from: Date) -> Self {
        self.from = Some(from);
        self
    }

    /// Sets the optional inclusive end date.
    pub const fn with_to(mut self, to: Date) -> Self {
        self.to = Some(to);
        self
    }

    /// Borrows the optional provider COT symbol.
    pub const fn symbol(&self) -> Option<&Ticker> {
        self.symbol.as_ref()
    }

    /// Returns the optional inclusive start date.
    pub const fn from(&self) -> Option<Date> {
        self.from
    }

    /// Returns the optional inclusive end date.
    pub const fn to(&self) -> Option<Date> {
        self.to
    }
}

impl QueryParameters for CotQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.optional("symbol", self.symbol.as_ref());
        encoder.optional("from", self.from);
        encoder.optional("to", self.to);
    }
}

const ANALYSIS_METADATA: EndpointMetadata =
    EndpointMetadata::new().with_bounds(EndpointBounds::new().with_date_range_days(90));

/// Describes `GET commitment-of-traders-report` without binding a transport.
pub fn cot_report(query: CotQuery) -> EndpointSpec<CotQuery, Vec<CotReport>> {
    EndpointSpec::get(
        "commitment-of-traders-report",
        "commitment-of-traders-report",
        query,
    )
}

/// Describes `GET commitment-of-traders-analysis` without binding a transport.
pub fn cot_analysis(query: CotQuery) -> EndpointSpec<CotQuery, Vec<CotAnalysis>> {
    EndpointSpec::get(
        "commitment-of-traders-analysis",
        "commitment-of-traders-analysis",
        query,
    )
    .with_metadata(ANALYSIS_METADATA)
}

/// Describes `GET commitment-of-traders-list` without binding a transport.
pub fn cot_report_list() -> EndpointSpec<(), Vec<CotReportListing>> {
    EndpointSpec::get(
        "commitment-of-traders-list",
        "commitment-of-traders-list",
        (),
    )
}

impl Client {
    /// Retrieves detailed Commitment of Traders reports.
    pub async fn cot_report(&self, query: CotQuery) -> Result<Vec<CotReport>> {
        self.execute(&cot_report(query)).await
    }

    /// Retrieves derived Commitment of Traders market analysis.
    pub async fn cot_analysis(&self, query: CotQuery) -> Result<Vec<CotAnalysis>> {
        self.execute(&cot_analysis(query)).await
    }

    /// Lists the Commitment of Traders reports available from the provider.
    pub async fn cot_report_list(&self) -> Result<Vec<CotReportListing>> {
        self.execute(&cot_report_list()).await
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
    fn query_encodes_exact_order_and_independent_omissions() {
        let from = Date::parse("2024-01-01").unwrap();
        let to = Date::parse("2024-03-01").unwrap();
        assert!(encoded(&CotQuery::new()).is_empty());
        assert_eq!(
            encoded(
                &CotQuery::new()
                    .with_symbol(Ticker::new("VX / Index").unwrap())
                    .with_from(from)
                    .with_to(to)
            ),
            [
                ("symbol".into(), "VX / Index".into()),
                ("from".into(), "2024-01-01".into()),
                ("to".into(), "2024-03-01".into()),
            ]
        );
        assert_eq!(
            encoded(&CotQuery::new().with_to(to)),
            [("to".into(), "2024-03-01".into())]
        );
    }
}
