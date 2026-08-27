//! Company profile, note, and peer endpoints.
//!
//! The reserved Python surface for a future binding is `fmp.company`, with
//! `FmpClient.profile`, `profile_by_cik`, `company_notes`, and `stock_peers`
//! methods and the corresponding public query and response models. This crate
//! does not implement those Python bindings.

use crate::{
    Client, Result,
    endpoints::{
        EndpointSpec, QueryEncoder, QueryParameters,
        metadata::{EndpointMetadata, GeographicAvailability},
    },
    responses::company::{CompanyNote, CompanyProfile, StockPeer},
    types::{Cik, Ticker},
};

macro_rules! required_query {
    ($docs:literal, $query:ident, $value:ident, $value_type:ty, $wire:literal) => {
        #[doc = $docs]
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $query {
            $value: $value_type,
        }

        impl $query {
            /// Creates a query from its required typed value.
            pub fn new($value: $value_type) -> Self {
                Self { $value }
            }

            /// Borrows the query's required typed value.
            pub fn $value(&self) -> &$value_type {
                &self.$value
            }
        }

        impl From<$value_type> for $query {
            fn from($value: $value_type) -> Self {
                Self::new($value)
            }
        }

        impl From<&$value_type> for $query {
            fn from($value: &$value_type) -> Self {
                Self::new($value.clone())
            }
        }

        impl QueryParameters for $query {
            fn encode(&self, encoder: &mut QueryEncoder<'_>) {
                encoder.required($wire, &self.$value);
            }
        }
    };
}

required_query!(
    "Required query parameters for a worldwide company profile.",
    ProfileQuery,
    symbol,
    Ticker,
    "symbol"
);
required_query!(
    "Required query parameters for a US company profile lookup by CIK.",
    ProfileByCikQuery,
    cik,
    Cik,
    "cik"
);
required_query!(
    "Required query parameters for US company-issued notes.",
    CompanyNotesQuery,
    symbol,
    Ticker,
    "symbol"
);
required_query!(
    "Required query parameters for worldwide stock peers.",
    StockPeersQuery,
    symbol,
    Ticker,
    "symbol"
);

const WORLDWIDE: EndpointMetadata =
    EndpointMetadata::new().with_geography(GeographicAvailability::Worldwide);
const US_ONLY: EndpointMetadata =
    EndpointMetadata::new().with_geography(GeographicAvailability::UsOnly);

/// Describes `GET profile` without binding it to a transport.
pub fn profile(query: ProfileQuery) -> EndpointSpec<ProfileQuery, Vec<CompanyProfile>> {
    EndpointSpec::get("profile", "profile", query).with_metadata(WORLDWIDE)
}

/// Describes `GET profile-cik` without binding it to a transport.
pub fn profile_by_cik(
    query: ProfileByCikQuery,
) -> EndpointSpec<ProfileByCikQuery, Vec<CompanyProfile>> {
    EndpointSpec::get("profile-cik", "profile-cik", query).with_metadata(US_ONLY)
}

/// Describes `GET company-notes` without binding it to a transport.
pub fn company_notes(
    query: CompanyNotesQuery,
) -> EndpointSpec<CompanyNotesQuery, Vec<CompanyNote>> {
    EndpointSpec::get("company-notes", "company-notes", query).with_metadata(US_ONLY)
}

/// Describes `GET stock-peers` without binding it to a transport.
pub fn stock_peers(query: StockPeersQuery) -> EndpointSpec<StockPeersQuery, Vec<StockPeer>> {
    EndpointSpec::get("stock-peers", "stock-peers", query).with_metadata(WORLDWIDE)
}

impl Client {
    /// Retrieves worldwide company profiles by provider ticker.
    pub async fn profile(&self, query: impl Into<ProfileQuery>) -> Result<Vec<CompanyProfile>> {
        self.execute(&profile(query.into())).await
    }

    /// Retrieves US company profiles by representation-preserving CIK.
    pub async fn profile_by_cik(
        &self,
        query: impl Into<ProfileByCikQuery>,
    ) -> Result<Vec<CompanyProfile>> {
        self.execute(&profile_by_cik(query.into())).await
    }

    /// Retrieves company-issued notes for a US company.
    pub async fn company_notes(
        &self,
        query: impl Into<CompanyNotesQuery>,
    ) -> Result<Vec<CompanyNote>> {
        self.execute(&company_notes(query.into())).await
    }

    /// Retrieves worldwide stock peers for a company.
    pub async fn stock_peers(&self, query: impl Into<StockPeersQuery>) -> Result<Vec<StockPeer>> {
        self.execute(&stock_peers(query.into())).await
    }
}
