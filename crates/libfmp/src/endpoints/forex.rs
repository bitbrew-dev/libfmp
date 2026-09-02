//! Forex catalog and quote endpoints.
//!
//! Forex-specific facades keep shared quote routes discoverable without
//! duplicating their query or response contracts.

use crate::{
    Client, Result,
    endpoints::{
        EndpointSpec,
        metadata::{EndpointMetadata, GeographicAvailability},
    },
    responses::forex::{ForexPair, Quote, QuoteShort},
};

pub use super::quote::{QuoteQuery, QuoteShortQuery, ShortOnlyQuery};

const US_ONLY: EndpointMetadata =
    EndpointMetadata::new().with_geography(GeographicAvailability::UsOnly);

/// Describes `GET forex-list` without binding it to a transport.
pub fn forex_list() -> EndpointSpec<(), Vec<ForexPair>> {
    EndpointSpec::get("forex-list", "forex-list", ())
}

/// Describes the forex-specific `GET quote` contract.
pub fn forex_quote(query: QuoteQuery) -> EndpointSpec<QuoteQuery, Vec<Quote>> {
    EndpointSpec::get("quote", "quote", query).with_metadata(US_ONLY)
}

/// Describes the forex-specific `GET quote-short` contract.
pub fn forex_quote_short(query: QuoteShortQuery) -> EndpointSpec<QuoteShortQuery, Vec<QuoteShort>> {
    EndpointSpec::get("quote-short", "quote-short", query).with_metadata(US_ONLY)
}

/// Describes compact `GET batch-forex-quotes`.
///
/// The returned query always emits `short=true`; the undocumented full batch
/// response shape is deliberately deferred.
pub fn forex_quotes() -> EndpointSpec<ShortOnlyQuery, Vec<QuoteShort>> {
    super::quote::forex_quotes()
}

impl Client {
    /// Lists the provider's documented forex pairs.
    pub async fn forex_list(&self) -> Result<Vec<ForexPair>> {
        self.execute(&forex_list()).await
    }

    /// Retrieves a detailed US-only quote for one forex pair.
    pub async fn forex_quote(&self, query: impl Into<QuoteQuery>) -> Result<Vec<Quote>> {
        self.execute(&forex_quote(query.into())).await
    }

    /// Retrieves a compact US-only quote for one forex pair.
    pub async fn forex_quote_short(
        &self,
        query: impl Into<QuoteShortQuery>,
    ) -> Result<Vec<QuoteShort>> {
        self.execute(&forex_quote_short(query.into())).await
    }
}
