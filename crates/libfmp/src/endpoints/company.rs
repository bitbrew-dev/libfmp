//! Company profile, note, peer, delisting, and workforce endpoints.
//!
//! The reserved Python surface for a future binding is `fmp.company`, with
//! `FmpClient.profile`, `profile_by_cik`, `company_notes`, `stock_peers`,
//! `delisted_companies`, `employee_count`, and `historical_employee_count`
//! methods. The matching `ProfileQuery`, `ProfileByCikQuery`,
//! `CompanyNotesQuery`, `StockPeersQuery`, `DelistedCompaniesQuery`,
//! `EmployeeCountQuery`, and `HistoricalEmployeeCountQuery` names are also
//! reserved there. This crate does not implement those Python bindings.

use crate::{
    Client, Result,
    endpoints::{
        EndpointSpec, QueryEncoder, QueryParameters,
        metadata::{EndpointBounds, EndpointMetadata, GeographicAvailability},
    },
    responses::company::{CompanyNote, CompanyProfile, DelistedCompany, EmployeeCount, StockPeer},
    types::{Cik, Limit, Page, Ticker},
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

/// Optional pagination parameters for delisted US companies.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DelistedCompaniesQuery {
    page: Option<Page>,
    limit: Option<Limit>,
}

impl DelistedCompaniesQuery {
    /// Creates a query without undocumented pagination defaults.
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

impl QueryParameters for DelistedCompaniesQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.optional("page", self.page);
        encoder.optional("limit", self.limit);
    }
}

macro_rules! employee_count_query {
    ($docs:literal, $query:ident) => {
        #[doc = $docs]
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $query {
            symbol: Ticker,
            limit: Option<Limit>,
        }

        impl $query {
            /// Creates a query without an undocumented result limit.
            pub fn new(symbol: Ticker) -> Self {
                Self {
                    symbol,
                    limit: None,
                }
            }

            /// Sets the optional provider result limit.
            pub fn with_limit(mut self, limit: Limit) -> Self {
                self.limit = Some(limit);
                self
            }

            /// Borrows the required provider ticker.
            pub fn symbol(&self) -> &Ticker {
                &self.symbol
            }

            /// Returns the optional provider result limit.
            pub const fn limit(&self) -> Option<Limit> {
                self.limit
            }
        }

        impl From<Ticker> for $query {
            fn from(symbol: Ticker) -> Self {
                Self::new(symbol)
            }
        }

        impl From<&Ticker> for $query {
            fn from(symbol: &Ticker) -> Self {
                Self::new(symbol.clone())
            }
        }

        impl QueryParameters for $query {
            fn encode(&self, encoder: &mut QueryEncoder<'_>) {
                encoder.required("symbol", &self.symbol);
                encoder.optional("limit", self.limit);
            }
        }
    };
}

employee_count_query!(
    "Required symbol and optional limit for the current US employee count.",
    EmployeeCountQuery
);
employee_count_query!(
    "Required symbol and optional limit for historical US employee counts.",
    HistoricalEmployeeCountQuery
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
const DELISTED_COMPANIES: EndpointMetadata =
    US_ONLY.with_bounds(EndpointBounds::new().with_response_rows(100));
const EMPLOYEE_COUNTS: EndpointMetadata =
    US_ONLY.with_bounds(EndpointBounds::new().with_response_rows(10_000));

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

/// Describes `GET delisted-companies` without binding it to a transport.
pub fn delisted_companies(
    query: DelistedCompaniesQuery,
) -> EndpointSpec<DelistedCompaniesQuery, Vec<DelistedCompany>> {
    EndpointSpec::get("delisted-companies", "delisted-companies", query)
        .with_metadata(DELISTED_COMPANIES)
}

/// Describes `GET employee-count` without binding it to a transport.
pub fn employee_count(
    query: EmployeeCountQuery,
) -> EndpointSpec<EmployeeCountQuery, Vec<EmployeeCount>> {
    EndpointSpec::get("employee-count", "employee-count", query).with_metadata(EMPLOYEE_COUNTS)
}

/// Describes `GET historical-employee-count` without binding it to a transport.
pub fn historical_employee_count(
    query: HistoricalEmployeeCountQuery,
) -> EndpointSpec<HistoricalEmployeeCountQuery, Vec<EmployeeCount>> {
    EndpointSpec::get(
        "historical-employee-count",
        "historical-employee-count",
        query,
    )
    .with_metadata(EMPLOYEE_COUNTS)
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

    /// Retrieves delisted US companies with optional provider pagination.
    pub async fn delisted_companies(
        &self,
        query: DelistedCompaniesQuery,
    ) -> Result<Vec<DelistedCompany>> {
        self.execute(&delisted_companies(query)).await
    }

    /// Retrieves current employee-count filings for a US company.
    pub async fn employee_count(
        &self,
        query: impl Into<EmployeeCountQuery>,
    ) -> Result<Vec<EmployeeCount>> {
        self.execute(&employee_count(query.into())).await
    }

    /// Retrieves historical employee-count filings for a US company.
    pub async fn historical_employee_count(
        &self,
        query: impl Into<HistoricalEmployeeCountQuery>,
    ) -> Result<Vec<EmployeeCount>> {
        self.execute(&historical_employee_count(query.into())).await
    }
}
