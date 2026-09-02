//! Commodity catalog and quote endpoints.
//!
//! Commodity-specific facades keep shared quote routes discoverable without
//! duplicating their query or response contracts.

use crate::{
    Client, Result,
    endpoints::{
        EndpointSpec,
        metadata::{EndpointMetadata, GeographicAvailability},
    },
    responses::commodities::{CommodityListing, Quote, QuoteShort},
};

pub use super::quote::{QuoteQuery, QuoteShortQuery, ShortOnlyQuery};

const US_ONLY: EndpointMetadata =
    EndpointMetadata::new().with_geography(GeographicAvailability::UsOnly);

/// Describes `GET commodities-list` without binding it to a transport.
pub fn commodities_list() -> EndpointSpec<(), Vec<CommodityListing>> {
    EndpointSpec::get("commodities-list", "commodities-list", ())
}

/// Describes the commodity-specific `GET quote` contract.
pub fn commodity_quote(query: QuoteQuery) -> EndpointSpec<QuoteQuery, Vec<Quote>> {
    EndpointSpec::get("quote", "quote", query).with_metadata(US_ONLY)
}

/// Describes the commodity-specific `GET quote-short` contract.
pub fn commodity_quote_short(
    query: QuoteShortQuery,
) -> EndpointSpec<QuoteShortQuery, Vec<QuoteShort>> {
    EndpointSpec::get("quote-short", "quote-short", query).with_metadata(US_ONLY)
}

/// Describes compact `GET batch-commodity-quotes`.
///
/// The returned query always emits `short=true`; the undocumented full batch
/// response shape is deliberately deferred.
pub fn commodity_quotes() -> EndpointSpec<ShortOnlyQuery, Vec<QuoteShort>> {
    super::quote::commodity_quotes()
}

impl Client {
    /// Lists the provider's documented commodity catalog.
    pub async fn commodities_list(&self) -> Result<Vec<CommodityListing>> {
        self.execute(&commodities_list()).await
    }

    /// Retrieves a detailed US-only quote for one commodity.
    pub async fn commodity_quote(&self, query: impl Into<QuoteQuery>) -> Result<Vec<Quote>> {
        self.execute(&commodity_quote(query.into())).await
    }

    /// Retrieves a compact US-only quote for one commodity.
    pub async fn commodity_quote_short(
        &self,
        query: impl Into<QuoteShortQuery>,
    ) -> Result<Vec<QuoteShort>> {
        self.execute(&commodity_quote_short(query.into())).await
    }
}
