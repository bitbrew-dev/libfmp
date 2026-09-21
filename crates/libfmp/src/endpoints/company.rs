//! Company profile, note, peer, delisting, workforce, market-cap, share-float,
//! merger-and-acquisition, and company-governance endpoints.
//!
//! Geographic availability varies per route and is carried by each
//! descriptor's [`GeographicAvailability`]. The Python binding exposes the
//! same seventeen routes under `client.company`; the only renamed method is
//! `company_notes`, reached from Python as `client.company.notes`. The typed
//! query structs below are Rust-only contracts and have no Python classes.

use crate::{
    Client, Result,
    endpoints::{
        EndpointSpec, QueryEncoder, QueryParameters,
        metadata::{EndpointBounds, EndpointMetadata, GeographicAvailability},
    },
    responses::company::{
        AllSharesFloatRecord, CompanyExecutive, CompanyNote, CompanyProfile, CompanyShareFloat,
        DelistedCompany, EmployeeCount, ExecutiveCompensation, ExecutiveCompensationBenchmark,
        MarketCapitalizationRecord, MergerAcquisition, StockPeer,
    },
    types::{BenchmarkYear, Cik, Date, Limit, Page, SearchTerm, Ticker, TickerList},
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
    "Required company-name search term for US mergers and acquisitions.",
    MergersAcquisitionsSearchQuery,
    name,
    SearchTerm,
    "name"
);
required_query!(
    "Required query parameters for worldwide company executives.",
    KeyExecutivesQuery,
    symbol,
    Ticker,
    "symbol"
);
required_query!(
    "Required query parameters for US executive compensation.",
    ExecutiveCompensationQuery,
    symbol,
    Ticker,
    "symbol"
);

/// Optional year for US executive-compensation industry benchmarks.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExecutiveCompensationBenchmarkQuery {
    year: Option<BenchmarkYear>,
}

impl ExecutiveCompensationBenchmarkQuery {
    /// Creates a query without an undocumented year default.
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

impl QueryParameters for ExecutiveCompensationBenchmarkQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.optional("year", self.year.as_ref());
    }
}

/// Optional pagination parameters for the latest US mergers and acquisitions.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MergersAcquisitionsLatestQuery {
    page: Option<Page>,
    limit: Option<Limit>,
}

impl MergersAcquisitionsLatestQuery {
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

impl QueryParameters for MergersAcquisitionsLatestQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.optional("page", self.page);
        encoder.optional("limit", self.limit);
    }
}
required_query!(
    "Required query parameters for a worldwide company market-capitalization lookup.",
    MarketCapitalizationQuery,
    symbol,
    Ticker,
    "symbol"
);
required_query!(
    "Required query parameters for a worldwide batch market-capitalization lookup.",
    MarketCapitalizationBatchQuery,
    symbols,
    TickerList,
    "symbols"
);
required_query!(
    "Required query parameters for a worldwide company share-float lookup.",
    SharesFloatQuery,
    symbol,
    Ticker,
    "symbol"
);

/// Required symbol and independently optional filters for historical market capitalization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoricalMarketCapitalizationQuery {
    symbol: Ticker,
    limit: Option<Limit>,
    from: Option<Date>,
    to: Option<Date>,
}

impl HistoricalMarketCapitalizationQuery {
    /// Creates a query without undocumented filter defaults.
    pub fn new(symbol: Ticker) -> Self {
        Self {
            symbol,
            limit: None,
            from: None,
            to: None,
        }
    }

    /// Sets the optional provider result limit.
    pub fn with_limit(mut self, limit: Limit) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Sets the optional inclusive start date independently of the end date.
    pub fn with_from(mut self, from: Date) -> Self {
        self.from = Some(from);
        self
    }

    /// Sets the optional inclusive end date independently of the start date.
    pub fn with_to(mut self, to: Date) -> Self {
        self.to = Some(to);
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

    /// Returns the optional inclusive start date.
    pub const fn from(&self) -> Option<Date> {
        self.from
    }

    /// Returns the optional inclusive end date.
    pub const fn to(&self) -> Option<Date> {
        self.to
    }
}

impl From<Ticker> for HistoricalMarketCapitalizationQuery {
    fn from(symbol: Ticker) -> Self {
        Self::new(symbol)
    }
}

impl From<&Ticker> for HistoricalMarketCapitalizationQuery {
    fn from(symbol: &Ticker) -> Self {
        Self::new(symbol.clone())
    }
}

impl QueryParameters for HistoricalMarketCapitalizationQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.required("symbol", &self.symbol);
        encoder.optional("limit", self.limit);
        encoder.optional("from", self.from);
        encoder.optional("to", self.to);
    }
}

/// Optional pagination parameters for worldwide all-company share-float data.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SharesFloatAllQuery {
    page: Option<Page>,
    limit: Option<Limit>,
}

impl SharesFloatAllQuery {
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

impl QueryParameters for SharesFloatAllQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.optional("page", self.page);
        encoder.optional("limit", self.limit);
    }
}

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
const MARKET_DATA_5K: EndpointMetadata =
    WORLDWIDE.with_bounds(EndpointBounds::new().with_response_rows(5_000));
const MERGERS_ACQUISITIONS_LATEST: EndpointMetadata =
    US_ONLY.with_bounds(EndpointBounds::new().with_response_rows(1_000));

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

/// Describes `GET market-capitalization` without binding it to a transport.
pub fn market_capitalization(
    query: MarketCapitalizationQuery,
) -> EndpointSpec<MarketCapitalizationQuery, Vec<MarketCapitalizationRecord>> {
    EndpointSpec::get("market-capitalization", "market-capitalization", query)
        .with_metadata(WORLDWIDE)
}

/// Describes `GET market-capitalization-batch` without binding it to a transport.
pub fn market_capitalization_batch(
    query: MarketCapitalizationBatchQuery,
) -> EndpointSpec<MarketCapitalizationBatchQuery, Vec<MarketCapitalizationRecord>> {
    EndpointSpec::get(
        "market-capitalization-batch",
        "market-capitalization-batch",
        query,
    )
    .with_metadata(WORLDWIDE)
}

/// Describes `GET historical-market-capitalization` without binding it to a transport.
pub fn historical_market_capitalization(
    query: HistoricalMarketCapitalizationQuery,
) -> EndpointSpec<HistoricalMarketCapitalizationQuery, Vec<MarketCapitalizationRecord>> {
    EndpointSpec::get(
        "historical-market-capitalization",
        "historical-market-capitalization",
        query,
    )
    .with_metadata(MARKET_DATA_5K)
}

/// Describes `GET shares-float` without binding it to a transport.
pub fn shares_float(
    query: SharesFloatQuery,
) -> EndpointSpec<SharesFloatQuery, Vec<CompanyShareFloat>> {
    EndpointSpec::get("shares-float", "shares-float", query).with_metadata(WORLDWIDE)
}

/// Describes `GET shares-float-all` without binding it to a transport.
pub fn shares_float_all(
    query: SharesFloatAllQuery,
) -> EndpointSpec<SharesFloatAllQuery, Vec<AllSharesFloatRecord>> {
    EndpointSpec::get("shares-float-all", "shares-float-all", query).with_metadata(MARKET_DATA_5K)
}

/// Describes `GET mergers-acquisitions-latest` without binding it to a transport.
pub fn mergers_acquisitions_latest(
    query: MergersAcquisitionsLatestQuery,
) -> EndpointSpec<MergersAcquisitionsLatestQuery, Vec<MergerAcquisition>> {
    EndpointSpec::get(
        "mergers-acquisitions-latest",
        "mergers-acquisitions-latest",
        query,
    )
    .with_metadata(MERGERS_ACQUISITIONS_LATEST)
}

/// Describes `GET mergers-acquisitions-search` without binding it to a transport.
pub fn mergers_acquisitions_search(
    query: MergersAcquisitionsSearchQuery,
) -> EndpointSpec<MergersAcquisitionsSearchQuery, Vec<MergerAcquisition>> {
    EndpointSpec::get(
        "mergers-acquisitions-search",
        "mergers-acquisitions-search",
        query,
    )
    .with_metadata(US_ONLY)
}

/// Describes `GET key-executives` without binding it to a transport.
pub fn key_executives(
    query: KeyExecutivesQuery,
) -> EndpointSpec<KeyExecutivesQuery, Vec<CompanyExecutive>> {
    EndpointSpec::get("key-executives", "key-executives", query).with_metadata(WORLDWIDE)
}

/// Describes `GET governance-executive-compensation` without binding it to a transport.
pub fn executive_compensation(
    query: ExecutiveCompensationQuery,
) -> EndpointSpec<ExecutiveCompensationQuery, Vec<ExecutiveCompensation>> {
    EndpointSpec::get(
        "governance-executive-compensation",
        "governance-executive-compensation",
        query,
    )
    .with_metadata(US_ONLY)
}

/// Describes `GET executive-compensation-benchmark` without binding it to a transport.
pub fn executive_compensation_benchmark(
    query: ExecutiveCompensationBenchmarkQuery,
) -> EndpointSpec<ExecutiveCompensationBenchmarkQuery, Vec<ExecutiveCompensationBenchmark>> {
    EndpointSpec::get(
        "executive-compensation-benchmark",
        "executive-compensation-benchmark",
        query,
    )
    .with_metadata(US_ONLY)
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

    /// Retrieves current worldwide market capitalization for one company.
    pub async fn market_capitalization(
        &self,
        query: impl Into<MarketCapitalizationQuery>,
    ) -> Result<Vec<MarketCapitalizationRecord>> {
        self.execute(&market_capitalization(query.into())).await
    }

    /// Retrieves current worldwide market capitalization for multiple companies.
    pub async fn market_capitalization_batch(
        &self,
        query: impl Into<MarketCapitalizationBatchQuery>,
    ) -> Result<Vec<MarketCapitalizationRecord>> {
        self.execute(&market_capitalization_batch(query.into()))
            .await
    }

    /// Retrieves historical worldwide market capitalization for one company.
    pub async fn historical_market_capitalization(
        &self,
        query: impl Into<HistoricalMarketCapitalizationQuery>,
    ) -> Result<Vec<MarketCapitalizationRecord>> {
        self.execute(&historical_market_capitalization(query.into()))
            .await
    }

    /// Retrieves current worldwide share-float data for one company.
    pub async fn shares_float(
        &self,
        query: impl Into<SharesFloatQuery>,
    ) -> Result<Vec<CompanyShareFloat>> {
        self.execute(&shares_float(query.into())).await
    }

    /// Retrieves paginated worldwide share-float data for all companies.
    pub async fn shares_float_all(
        &self,
        query: SharesFloatAllQuery,
    ) -> Result<Vec<AllSharesFloatRecord>> {
        self.execute(&shares_float_all(query)).await
    }

    /// Retrieves the latest US mergers and acquisitions with optional pagination.
    pub async fn mergers_acquisitions_latest(
        &self,
        query: MergersAcquisitionsLatestQuery,
    ) -> Result<Vec<MergerAcquisition>> {
        self.execute(&mergers_acquisitions_latest(query)).await
    }

    /// Searches US mergers and acquisitions by representation-preserving company name.
    pub async fn mergers_acquisitions_search(
        &self,
        query: impl Into<MergersAcquisitionsSearchQuery>,
    ) -> Result<Vec<MergerAcquisition>> {
        self.execute(&mergers_acquisitions_search(query.into()))
            .await
    }

    /// Retrieves worldwide company executives by provider ticker.
    pub async fn key_executives(
        &self,
        query: impl Into<KeyExecutivesQuery>,
    ) -> Result<Vec<CompanyExecutive>> {
        self.execute(&key_executives(query.into())).await
    }

    /// Retrieves executive compensation for a US company.
    pub async fn executive_compensation(
        &self,
        query: impl Into<ExecutiveCompensationQuery>,
    ) -> Result<Vec<ExecutiveCompensation>> {
        self.execute(&executive_compensation(query.into())).await
    }

    /// Retrieves US executive-compensation benchmarks with an optional year.
    pub async fn executive_compensation_benchmark(
        &self,
        query: ExecutiveCompensationBenchmarkQuery,
    ) -> Result<Vec<ExecutiveCompensationBenchmark>> {
        self.execute(&executive_compensation_benchmark(query)).await
    }
}
