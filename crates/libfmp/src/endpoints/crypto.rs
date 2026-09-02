//! Cryptocurrency catalog and quote endpoints.
//!
//! Cryptocurrency-specific facades keep shared quote routes discoverable
//! without duplicating their query or response contracts.

use crate::{
    Client, Result,
    endpoints::EndpointSpec,
    responses::crypto::{CryptocurrencyListing, Quote, QuoteShort},
};

pub use super::quote::{QuoteQuery, QuoteShortQuery, ShortOnlyQuery};

/// Describes `GET cryptocurrency-list` without binding it to a transport.
pub fn cryptocurrency_list() -> EndpointSpec<(), Vec<CryptocurrencyListing>> {
    EndpointSpec::get("cryptocurrency-list", "cryptocurrency-list", ())
}

/// Describes the cryptocurrency-specific `GET quote` contract.
pub fn cryptocurrency_quote(query: QuoteQuery) -> EndpointSpec<QuoteQuery, Vec<Quote>> {
    EndpointSpec::get("quote", "quote", query)
}

/// Describes the cryptocurrency-specific `GET quote-short` contract.
pub fn cryptocurrency_quote_short(
    query: QuoteShortQuery,
) -> EndpointSpec<QuoteShortQuery, Vec<QuoteShort>> {
    EndpointSpec::get("quote-short", "quote-short", query)
}

/// Describes compact `GET batch-crypto-quotes`.
///
/// The returned query always emits `short=true`; the undocumented full batch
/// response shape is deliberately deferred.
pub fn cryptocurrency_quotes() -> EndpointSpec<ShortOnlyQuery, Vec<QuoteShort>> {
    super::quote::cryptocurrency_quotes()
}

impl Client {
    /// Lists the provider's documented cryptocurrency catalog.
    pub async fn cryptocurrency_list(&self) -> Result<Vec<CryptocurrencyListing>> {
        self.execute(&cryptocurrency_list()).await
    }

    /// Retrieves a detailed quote for one cryptocurrency.
    pub async fn cryptocurrency_quote(&self, query: impl Into<QuoteQuery>) -> Result<Vec<Quote>> {
        self.execute(&cryptocurrency_quote(query.into())).await
    }

    /// Retrieves a compact quote for one cryptocurrency.
    pub async fn cryptocurrency_quote_short(
        &self,
        query: impl Into<QuoteShortQuery>,
    ) -> Result<Vec<QuoteShort>> {
        self.execute(&cryptocurrency_quote_short(query.into()))
            .await
    }
}
