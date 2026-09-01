//! Stock-market index directory, quote, history, and constituent endpoints.
//!
//! This initial slice provides the index directory and discoverable index
//! quote routes. Generic quote response and query contracts are deliberately
//! reused from [`super::quote`].

use crate::{
    Client, Result,
    endpoints::{
        EndpointSpec,
        metadata::{EndpointMetadata, GeographicAvailability},
    },
    responses::{
        indexes::IndexListing,
        quote::{Quote, QuoteShort},
    },
};

pub use super::quote::{QuoteQuery, QuoteShortQuery, ShortOnlyQuery};

const WORLDWIDE: EndpointMetadata =
    EndpointMetadata::new().with_geography(GeographicAvailability::Worldwide);

/// Describes `GET index-list` without binding it to a transport.
pub fn index_list() -> EndpointSpec<(), Vec<IndexListing>> {
    EndpointSpec::get("index-list", "index-list", ()).with_metadata(WORLDWIDE)
}

/// Describes the generic `GET quote` route for one stock-market index.
pub fn index_quote(query: QuoteQuery) -> EndpointSpec<QuoteQuery, Vec<Quote>> {
    super::quote::quote(query)
}

/// Describes the generic `GET quote-short` route for one stock-market index.
pub fn index_quote_short(query: QuoteShortQuery) -> EndpointSpec<QuoteShortQuery, Vec<QuoteShort>> {
    super::quote::quote_short(query)
}

/// Describes compact `GET batch-index-quotes` without binding it to a transport.
///
/// The returned query always emits `short=true`. The provider does not
/// document the batch endpoint's full response shape, so callers cannot select
/// `short=false` through this typed contract.
pub fn index_quotes() -> EndpointSpec<ShortOnlyQuery, Vec<QuoteShort>> {
    super::quote::index_quotes()
}

impl Client {
    /// Lists worldwide stock-market indexes.
    pub async fn index_list(&self) -> Result<Vec<IndexListing>> {
        self.execute(&index_list()).await
    }

    /// Retrieves a detailed quote for one stock-market index.
    pub async fn index_quote(&self, query: impl Into<QuoteQuery>) -> Result<Vec<Quote>> {
        self.execute(&index_quote(query.into())).await
    }

    /// Retrieves a compact quote for one stock-market index.
    pub async fn index_quote_short(
        &self,
        query: impl Into<QuoteShortQuery>,
    ) -> Result<Vec<QuoteShort>> {
        self.execute(&index_quote_short(query.into())).await
    }
}
