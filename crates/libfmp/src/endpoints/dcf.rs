//! Worldwide discounted-cash-flow valuation endpoint contracts.

use crate::{
    Client, Result,
    endpoints::{
        EndpointSpec, QueryEncoder, QueryParameters,
        metadata::{EndpointMetadata, GeographicAvailability},
    },
    responses::dcf::DcfValuation,
    types::Ticker,
};

const WORLDWIDE: EndpointMetadata =
    EndpointMetadata::new().with_geography(GeographicAvailability::Worldwide);

/// Required company symbol shared by the standard and levered DCF routes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DcfQuery {
    symbol: Ticker,
}

impl DcfQuery {
    /// Creates a DCF query for one company.
    pub const fn new(symbol: Ticker) -> Self {
        Self { symbol }
    }

    /// Borrows the requested company symbol.
    pub const fn symbol(&self) -> &Ticker {
        &self.symbol
    }
}

impl From<Ticker> for DcfQuery {
    fn from(symbol: Ticker) -> Self {
        Self::new(symbol)
    }
}

impl From<&Ticker> for DcfQuery {
    fn from(symbol: &Ticker) -> Self {
        Self::new(symbol.clone())
    }
}

impl QueryParameters for DcfQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.required("symbol", &self.symbol);
    }
}

/// Describes `GET discounted-cash-flow` without binding it to a transport.
pub fn discounted_cash_flow(query: DcfQuery) -> EndpointSpec<DcfQuery, Vec<DcfValuation>> {
    EndpointSpec::get("discounted-cash-flow", "discounted-cash-flow", query)
        .with_metadata(WORLDWIDE)
}

/// Describes `GET levered-discounted-cash-flow` without binding it to a transport.
pub fn levered_discounted_cash_flow(query: DcfQuery) -> EndpointSpec<DcfQuery, Vec<DcfValuation>> {
    EndpointSpec::get(
        "levered-discounted-cash-flow",
        "levered-discounted-cash-flow",
        query,
    )
    .with_metadata(WORLDWIDE)
}

impl Client {
    /// Retrieves the standard discounted-cash-flow valuation for one company.
    pub async fn discounted_cash_flow(
        &self,
        query: impl Into<DcfQuery>,
    ) -> Result<Vec<DcfValuation>> {
        self.execute(&discounted_cash_flow(query.into())).await
    }

    /// Retrieves the levered discounted-cash-flow valuation for one company.
    pub async fn levered_discounted_cash_flow(
        &self,
        query: impl Into<DcfQuery>,
    ) -> Result<Vec<DcfValuation>> {
        self.execute(&levered_discounted_cash_flow(query.into()))
            .await
    }
}
