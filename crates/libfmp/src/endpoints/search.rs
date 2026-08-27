//! Symbol, name, identifier, and exchange-variant search endpoints.
//!
//! The reserved Python surface for a future binding is `fmp.search`, with
//! `FmpClient.search_symbol`, `search_name`, `search_cik`, `search_cusip`,
//! `search_isin`, and `search_exchange_variants` methods and matching response
//! model names. This crate does not implement those Python bindings.

use crate::{
    Client, Result,
    endpoints::{
        EndpointSpec, QueryEncoder, QueryParameters,
        metadata::{EndpointMetadata, GeographicAvailability},
    },
    responses::search::{
        CikSearchResult, CusipSearchResult, ExchangeVariant, IsinSearchResult, NameSearchResult,
        SymbolSearchResult,
    },
    types::{Cik, Cusip, ExchangeCode, Isin, Limit, SearchTerm, Ticker},
};

/// Required and optional query parameters for symbol search.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolSearchQuery {
    query: SearchTerm,
    limit: Option<Limit>,
    exchange: Option<ExchangeCode>,
}

impl SymbolSearchQuery {
    /// Creates a symbol search without undocumented defaults.
    pub fn new(query: SearchTerm) -> Self {
        Self {
            query,
            limit: None,
            exchange: None,
        }
    }

    /// Sets the optional provider result limit.
    pub fn with_limit(mut self, limit: Limit) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Restricts results to the supplied open exchange code.
    pub fn with_exchange(mut self, exchange: ExchangeCode) -> Self {
        self.exchange = Some(exchange);
        self
    }

    /// Borrows the representation-preserving search term.
    pub fn query(&self) -> &SearchTerm {
        &self.query
    }

    /// Returns the optional provider result limit.
    pub fn limit(&self) -> Option<Limit> {
        self.limit
    }

    /// Borrows the optional exchange restriction.
    pub fn exchange(&self) -> Option<&ExchangeCode> {
        self.exchange.as_ref()
    }
}

impl From<SearchTerm> for SymbolSearchQuery {
    fn from(query: SearchTerm) -> Self {
        Self::new(query)
    }
}

impl From<&SearchTerm> for SymbolSearchQuery {
    fn from(query: &SearchTerm) -> Self {
        Self::new(query.clone())
    }
}

impl QueryParameters for SymbolSearchQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.required("query", &self.query);
        encoder.optional("limit", self.limit);
        encoder.optional("exchange", self.exchange.as_ref());
    }
}

/// Required and optional query parameters for company-name search.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameSearchQuery {
    query: SearchTerm,
    limit: Option<Limit>,
    exchange: Option<ExchangeCode>,
}

impl NameSearchQuery {
    /// Creates a company-name search without undocumented defaults.
    pub fn new(query: SearchTerm) -> Self {
        Self {
            query,
            limit: None,
            exchange: None,
        }
    }

    /// Sets the optional provider result limit.
    pub fn with_limit(mut self, limit: Limit) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Restricts results to the supplied open exchange code.
    pub fn with_exchange(mut self, exchange: ExchangeCode) -> Self {
        self.exchange = Some(exchange);
        self
    }

    /// Borrows the representation-preserving search term.
    pub fn query(&self) -> &SearchTerm {
        &self.query
    }

    /// Returns the optional provider result limit.
    pub fn limit(&self) -> Option<Limit> {
        self.limit
    }

    /// Borrows the optional exchange restriction.
    pub fn exchange(&self) -> Option<&ExchangeCode> {
        self.exchange.as_ref()
    }
}

impl From<SearchTerm> for NameSearchQuery {
    fn from(query: SearchTerm) -> Self {
        Self::new(query)
    }
}

impl From<&SearchTerm> for NameSearchQuery {
    fn from(query: &SearchTerm) -> Self {
        Self::new(query.clone())
    }
}

impl QueryParameters for NameSearchQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.required("query", &self.query);
        encoder.optional("limit", self.limit);
        encoder.optional("exchange", self.exchange.as_ref());
    }
}

/// Required and optional query parameters for CIK search.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CikSearchQuery {
    cik: Cik,
    limit: Option<Limit>,
}

impl CikSearchQuery {
    /// Creates a CIK search without an undocumented result limit.
    pub fn new(cik: Cik) -> Self {
        Self { cik, limit: None }
    }

    /// Sets the optional provider result limit.
    pub fn with_limit(mut self, limit: Limit) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Borrows the representation-preserving CIK.
    pub fn cik(&self) -> &Cik {
        &self.cik
    }

    /// Returns the optional provider result limit.
    pub fn limit(&self) -> Option<Limit> {
        self.limit
    }
}

impl From<Cik> for CikSearchQuery {
    fn from(cik: Cik) -> Self {
        Self::new(cik)
    }
}

impl From<&Cik> for CikSearchQuery {
    fn from(cik: &Cik) -> Self {
        Self::new(cik.clone())
    }
}

impl QueryParameters for CikSearchQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.required("cik", &self.cik);
        encoder.optional("limit", self.limit);
    }
}

macro_rules! required_identifier_query {
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

required_identifier_query!(
    "Required query parameters for CUSIP search.",
    CusipSearchQuery,
    cusip,
    Cusip,
    "cusip"
);
required_identifier_query!(
    "Required query parameters for ISIN search.",
    IsinSearchQuery,
    isin,
    Isin,
    "isin"
);
required_identifier_query!(
    "Required query parameters for exchange-variants search.",
    ExchangeVariantsQuery,
    symbol,
    Ticker,
    "symbol"
);

const WORLDWIDE: EndpointMetadata =
    EndpointMetadata::new().with_geography(GeographicAvailability::Worldwide);
const US_ONLY: EndpointMetadata =
    EndpointMetadata::new().with_geography(GeographicAvailability::UsOnly);

/// Describes `GET search-symbol` without binding it to a transport.
pub fn search_symbol(
    query: SymbolSearchQuery,
) -> EndpointSpec<SymbolSearchQuery, Vec<SymbolSearchResult>> {
    EndpointSpec::get("search-symbol", "search-symbol", query).with_metadata(WORLDWIDE)
}

/// Describes `GET search-name` without binding it to a transport.
pub fn search_name(query: NameSearchQuery) -> EndpointSpec<NameSearchQuery, Vec<NameSearchResult>> {
    EndpointSpec::get("search-name", "search-name", query).with_metadata(WORLDWIDE)
}

/// Describes `GET search-cik` without binding it to a transport.
pub fn search_cik(query: CikSearchQuery) -> EndpointSpec<CikSearchQuery, Vec<CikSearchResult>> {
    EndpointSpec::get("search-cik", "search-cik", query).with_metadata(US_ONLY)
}

/// Describes `GET search-cusip` without binding it to a transport.
pub fn search_cusip(
    query: CusipSearchQuery,
) -> EndpointSpec<CusipSearchQuery, Vec<CusipSearchResult>> {
    EndpointSpec::get("search-cusip", "search-cusip", query).with_metadata(WORLDWIDE)
}

/// Describes `GET search-isin` without binding it to a transport.
pub fn search_isin(query: IsinSearchQuery) -> EndpointSpec<IsinSearchQuery, Vec<IsinSearchResult>> {
    EndpointSpec::get("search-isin", "search-isin", query).with_metadata(WORLDWIDE)
}

/// Describes `GET search-exchange-variants` without binding it to a transport.
pub fn search_exchange_variants(
    query: ExchangeVariantsQuery,
) -> EndpointSpec<ExchangeVariantsQuery, Vec<ExchangeVariant>> {
    EndpointSpec::get(
        "search-exchange-variants",
        "search-exchange-variants",
        query,
    )
    .with_metadata(WORLDWIDE)
}

impl Client {
    /// Searches worldwide companies and instruments by symbol.
    pub async fn search_symbol(
        &self,
        query: impl Into<SymbolSearchQuery>,
    ) -> Result<Vec<SymbolSearchResult>> {
        self.execute(&search_symbol(query.into())).await
    }

    /// Searches worldwide companies and instruments by name.
    pub async fn search_name(
        &self,
        query: impl Into<NameSearchQuery>,
    ) -> Result<Vec<NameSearchResult>> {
        self.execute(&search_name(query.into())).await
    }

    /// Searches US companies by Central Index Key.
    pub async fn search_cik(
        &self,
        query: impl Into<CikSearchQuery>,
    ) -> Result<Vec<CikSearchResult>> {
        self.execute(&search_cik(query.into())).await
    }

    /// Searches worldwide securities by CUSIP without constraining its length.
    pub async fn search_cusip(
        &self,
        query: impl Into<CusipSearchQuery>,
    ) -> Result<Vec<CusipSearchResult>> {
        self.execute(&search_cusip(query.into())).await
    }

    /// Searches worldwide securities by ISIN without constraining its length.
    pub async fn search_isin(
        &self,
        query: impl Into<IsinSearchQuery>,
    ) -> Result<Vec<IsinSearchResult>> {
        self.execute(&search_isin(query.into())).await
    }

    /// Finds worldwide exchange listings for a provider ticker.
    pub async fn search_exchange_variants(
        &self,
        query: impl Into<ExchangeVariantsQuery>,
    ) -> Result<Vec<ExchangeVariant>> {
        self.execute(&search_exchange_variants(query.into())).await
    }
}
