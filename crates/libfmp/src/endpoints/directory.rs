//! Symbol, transcript, and provider-taxonomy directory endpoints.
//!
//! The provider documents the CIK, symbol-change, and earnings-transcript
//! directories as available for US-based companies only, the symbol and
//! exchange directories as worldwide, and the sector, industry, and country
//! taxonomies without any geography, so each descriptor carries the matching
//! [`GeographicAvailability`]. The Python binding exposes the same eleven
//! routes under `client.directory`.

use crate::{
    Client, Result,
    codecs::TrueFalseFlag,
    endpoints::{
        EndpointSpec, QueryEncoder, QueryParameters,
        metadata::{EndpointBounds, EndpointMetadata, GeographicAvailability},
    },
    responses::directory::{
        ActivelyTradingSymbol, AvailableCountry, AvailableExchange, AvailableIndustry,
        AvailableSector, CikEntry, CompanySymbol, EarningsTranscriptAvailability, EtfSymbol,
        FinancialStatementSymbol, SymbolChange,
    },
    types::{Limit, Page},
};

/// Optional parameters for the supported-exchanges directory.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AvailableExchangesQuery {
    extended: Option<bool>,
}

impl AvailableExchangesQuery {
    /// Creates a query without an undocumented value for `extended`.
    pub const fn new() -> Self {
        Self { extended: None }
    }

    /// Sets whether the provider should return its extended exchange list.
    pub const fn with_extended(mut self, extended: bool) -> Self {
        self.extended = Some(extended);
        self
    }

    /// Returns the optional provider `extended` flag.
    pub const fn extended(&self) -> Option<bool> {
        self.extended
    }
}

impl QueryParameters for AvailableExchangesQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.optional("extended", self.extended);
    }
}

/// Optional pagination parameters for the CIK directory.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CikListQuery {
    page: Option<Page>,
    limit: Option<Limit>,
}

impl CikListQuery {
    /// Creates a CIK-directory query without undocumented defaults.
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

impl QueryParameters for CikListQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.optional("page", self.page);
        encoder.optional("limit", self.limit);
    }
}

/// Optional wire parameters for the symbol-change directory.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SymbolChangesQuery {
    invalid: Option<TrueFalseFlag>,
    limit: Option<Limit>,
}

impl SymbolChangesQuery {
    /// Creates a symbol-change query without interpreting the `invalid` flag.
    pub const fn new() -> Self {
        Self {
            invalid: None,
            limit: None,
        }
    }

    /// Sets the provider's ambiguous lowercase-string `invalid` parameter.
    pub const fn with_invalid(mut self, invalid: TrueFalseFlag) -> Self {
        self.invalid = Some(invalid);
        self
    }

    /// Sets the optional provider result limit.
    pub const fn with_limit(mut self, limit: Limit) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Returns the uninterpreted provider `invalid` flag.
    pub const fn invalid(&self) -> Option<TrueFalseFlag> {
        self.invalid
    }

    /// Returns the optional provider result limit.
    pub const fn limit(&self) -> Option<Limit> {
        self.limit
    }
}

impl QueryParameters for SymbolChangesQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.optional("invalid", self.invalid);
        encoder.optional("limit", self.limit);
    }
}

const WORLDWIDE: EndpointMetadata =
    EndpointMetadata::new().with_geography(GeographicAvailability::Worldwide);
const US_ONLY: EndpointMetadata =
    EndpointMetadata::new().with_geography(GeographicAvailability::UsOnly);
const CIK_LIST: EndpointMetadata =
    US_ONLY.with_bounds(EndpointBounds::new().with_response_rows(10_000));

/// Describes `GET stock-list` without binding it to a transport.
pub fn company_symbols() -> EndpointSpec<(), Vec<CompanySymbol>> {
    EndpointSpec::get("stock-list", "stock-list", ()).with_metadata(WORLDWIDE)
}

/// Describes `GET financial-statement-symbol-list` without binding it to a transport.
pub fn financial_statement_symbols() -> EndpointSpec<(), Vec<FinancialStatementSymbol>> {
    EndpointSpec::get(
        "financial-statement-symbol-list",
        "financial-statement-symbol-list",
        (),
    )
    .with_metadata(WORLDWIDE)
}

/// Describes `GET cik-list` without binding it to a transport.
pub fn cik_list(query: CikListQuery) -> EndpointSpec<CikListQuery, Vec<CikEntry>> {
    EndpointSpec::get("cik-list", "cik-list", query).with_metadata(CIK_LIST)
}

/// Describes `GET symbol-change` without binding it to a transport.
pub fn symbol_changes(
    query: SymbolChangesQuery,
) -> EndpointSpec<SymbolChangesQuery, Vec<SymbolChange>> {
    EndpointSpec::get("symbol-change", "symbol-change", query).with_metadata(US_ONLY)
}

/// Describes `GET etf-list` without binding it to a transport.
pub fn etf_symbols() -> EndpointSpec<(), Vec<EtfSymbol>> {
    EndpointSpec::get("etf-list", "etf-list", ()).with_metadata(WORLDWIDE)
}

/// Describes `GET actively-trading-list` without binding it to a transport.
pub fn actively_trading() -> EndpointSpec<(), Vec<ActivelyTradingSymbol>> {
    EndpointSpec::get("actively-trading-list", "actively-trading-list", ()).with_metadata(WORLDWIDE)
}

/// Describes `GET earnings-transcript-list` without binding it to a transport.
pub fn earnings_transcript_list() -> EndpointSpec<(), Vec<EarningsTranscriptAvailability>> {
    EndpointSpec::get("earnings-transcript-list", "earnings-transcript-list", ())
        .with_metadata(US_ONLY)
}

/// Describes `GET available-exchanges` without binding it to a transport.
pub fn available_exchanges(
    query: AvailableExchangesQuery,
) -> EndpointSpec<AvailableExchangesQuery, Vec<AvailableExchange>> {
    EndpointSpec::get("available-exchanges", "available-exchanges", query).with_metadata(WORLDWIDE)
}

/// Describes `GET available-sectors` without binding it to a transport.
pub fn available_sectors() -> EndpointSpec<(), Vec<AvailableSector>> {
    EndpointSpec::get("available-sectors", "available-sectors", ())
}

/// Describes `GET available-industries` without binding it to a transport.
pub fn available_industries() -> EndpointSpec<(), Vec<AvailableIndustry>> {
    EndpointSpec::get("available-industries", "available-industries", ())
}

/// Describes `GET available-countries` without binding it to a transport.
pub fn available_countries() -> EndpointSpec<(), Vec<AvailableCountry>> {
    EndpointSpec::get("available-countries", "available-countries", ())
}

impl Client {
    /// Lists worldwide company and instrument symbols.
    pub async fn company_symbols(&self) -> Result<Vec<CompanySymbol>> {
        self.execute(&company_symbols()).await
    }

    /// Lists worldwide companies with financial statements available.
    pub async fn financial_statement_symbols(&self) -> Result<Vec<FinancialStatementSymbol>> {
        self.execute(&financial_statement_symbols()).await
    }

    /// Lists US SEC entities with optional provider pagination.
    pub async fn cik_list(&self, query: CikListQuery) -> Result<Vec<CikEntry>> {
        self.execute(&cik_list(query)).await
    }

    /// Lists US symbol changes without interpreting the provider's `invalid` flag.
    pub async fn symbol_changes(&self, query: SymbolChangesQuery) -> Result<Vec<SymbolChange>> {
        self.execute(&symbol_changes(query)).await
    }

    /// Lists worldwide exchange-traded fund symbols.
    pub async fn etf_symbols(&self) -> Result<Vec<EtfSymbol>> {
        self.execute(&etf_symbols()).await
    }

    /// Lists worldwide actively trading companies and instruments.
    pub async fn actively_trading(&self) -> Result<Vec<ActivelyTradingSymbol>> {
        self.execute(&actively_trading()).await
    }

    /// Lists US companies with their available earnings-transcript counts.
    pub async fn earnings_transcript_list(&self) -> Result<Vec<EarningsTranscriptAvailability>> {
        self.execute(&earnings_transcript_list()).await
    }

    /// Lists supported worldwide stock exchanges.
    pub async fn available_exchanges(
        &self,
        query: AvailableExchangesQuery,
    ) -> Result<Vec<AvailableExchange>> {
        self.execute(&available_exchanges(query)).await
    }

    /// Lists the provider's available sectors.
    pub async fn available_sectors(&self) -> Result<Vec<AvailableSector>> {
        self.execute(&available_sectors()).await
    }

    /// Lists the provider's available industries.
    pub async fn available_industries(&self) -> Result<Vec<AvailableIndustry>> {
        self.execute(&available_industries()).await
    }

    /// Lists the provider's available countries.
    pub async fn available_countries(&self) -> Result<Vec<AvailableCountry>> {
        self.execute(&available_countries()).await
    }
}
