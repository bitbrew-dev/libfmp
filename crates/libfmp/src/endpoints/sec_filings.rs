//! SEC filing, company lookup, profile, and industry-classification query contracts.
//!
//! The provider documents every SEC-filings route as available for US-based
//! companies only, so each descriptor carries
//! [`GeographicAvailability::UsOnly`]. The Python binding exposes the same
//! twelve routes under `client.sec_filings`.

use crate::{
    Client, Result,
    codecs::DynamicObject,
    endpoints::{
        EndpointSpec, QueryEncoder, QueryParameters,
        metadata::{EndpointBounds, EndpointMetadata, GeographicAvailability},
    },
    responses::sec_filings::{
        SecCompanyProfile, SecCompanySearchResult, SecFiling, SicClassification,
    },
    types::{Cik, Date, FormType, Limit, Page, SearchTerm, Ticker},
};

macro_rules! date_page_limit_query {
    ($name:ident, $description:literal) => {
        #[doc = $description]
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $name {
            from: Date,
            to: Date,
            page: Option<Page>,
            limit: Option<Limit>,
        }

        impl $name {
            /// Creates a query without undocumented pagination defaults.
            pub const fn new(from: Date, to: Date) -> Self {
                Self {
                    from,
                    to,
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

            /// Returns the required inclusive start date.
            pub const fn from(&self) -> Date {
                self.from
            }

            /// Returns the required inclusive end date.
            pub const fn to(&self) -> Date {
                self.to
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

        impl QueryParameters for $name {
            fn encode(&self, encoder: &mut QueryEncoder<'_>) {
                encoder.required("from", self.from);
                encoder.required("to", self.to);
                encoder.optional("page", self.page);
                encoder.optional("limit", self.limit);
            }
        }
    };
}

date_page_limit_query!(
    Latest8kSecFilingsQuery,
    "Required date bounds and optional pagination for the latest US 8-K filings."
);
date_page_limit_query!(
    LatestSecFilingsQuery,
    "Required date bounds and optional pagination for the latest US financial filings."
);

macro_rules! filing_search_query {
    ($name:ident, $field:ident, $field_type:ty, $wire_key:literal, $description:literal) => {
        #[doc = $description]
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $name {
            $field: $field_type,
            from: Date,
            to: Date,
            page: Option<Page>,
            limit: Option<Limit>,
        }

        impl $name {
            /// Creates a filing search without undocumented pagination defaults.
            pub const fn new($field: $field_type, from: Date, to: Date) -> Self {
                Self {
                    $field,
                    from,
                    to,
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

            /// Borrows the required filing-search key.
            pub const fn $field(&self) -> &$field_type {
                &self.$field
            }

            /// Returns the required inclusive start date.
            pub const fn from(&self) -> Date {
                self.from
            }

            /// Returns the required inclusive end date.
            pub const fn to(&self) -> Date {
                self.to
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

        impl QueryParameters for $name {
            fn encode(&self, encoder: &mut QueryEncoder<'_>) {
                encoder.required($wire_key, &self.$field);
                encoder.required("from", self.from);
                encoder.required("to", self.to);
                encoder.optional("page", self.page);
                encoder.optional("limit", self.limit);
            }
        }
    };
}

filing_search_query!(
    SecFilingsByFormTypeQuery,
    form_type,
    FormType,
    "formType",
    "Required form type and dates plus optional pagination for US SEC filings."
);
filing_search_query!(
    SecFilingsBySymbolQuery,
    symbol,
    Ticker,
    "symbol",
    "Required ticker and dates plus optional pagination for US SEC filings."
);
filing_search_query!(
    SecFilingsByCikQuery,
    cik,
    Cik,
    "cik",
    "Required CIK and dates plus optional pagination for US SEC filings."
);

macro_rules! required_search_query {
    ($name:ident, $field:ident, $field_type:ty, $wire_key:literal, $description:literal) => {
        #[doc = $description]
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $name {
            $field: $field_type,
        }

        impl $name {
            /// Creates a company lookup preserving the exact search value.
            pub const fn new($field: $field_type) -> Self {
                Self { $field }
            }

            /// Borrows the required lookup value.
            pub const fn $field(&self) -> &$field_type {
                &self.$field
            }
        }

        impl From<$field_type> for $name {
            fn from(value: $field_type) -> Self {
                Self::new(value)
            }
        }

        impl From<&$field_type> for $name {
            fn from(value: &$field_type) -> Self {
                Self::new(value.clone())
            }
        }

        impl QueryParameters for $name {
            fn encode(&self, encoder: &mut QueryEncoder<'_>) {
                encoder.required($wire_key, &self.$field);
            }
        }
    };
}

required_search_query!(
    SecCompaniesByNameQuery,
    company,
    SearchTerm,
    "company",
    "Required company-name text for a US SEC company lookup."
);
required_search_query!(
    SecCompaniesBySymbolQuery,
    symbol,
    Ticker,
    "symbol",
    "Required ticker for a US SEC company lookup."
);
required_search_query!(
    SecCompaniesByCikQuery,
    cik,
    Cik,
    "cik",
    "Required CIK for a US SEC company lookup."
);

/// Required ticker and optional literal `cik-A` value for a full SEC profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecCompanyProfileQuery {
    symbol: Ticker,
    cik_a: Option<Cik>,
}

impl SecCompanyProfileQuery {
    /// Creates a profile query without an undocumented `cik-A` value.
    pub const fn new(symbol: Ticker) -> Self {
        Self {
            symbol,
            cik_a: None,
        }
    }

    /// Sets the provider's literal optional `cik-A` query value.
    pub fn with_cik_a(mut self, cik_a: Cik) -> Self {
        self.cik_a = Some(cik_a);
        self
    }

    /// Borrows the required ticker.
    pub const fn symbol(&self) -> &Ticker {
        &self.symbol
    }

    /// Borrows the optional literal `cik-A` value.
    pub const fn cik_a(&self) -> Option<&Cik> {
        self.cik_a.as_ref()
    }
}

impl From<Ticker> for SecCompanyProfileQuery {
    fn from(symbol: Ticker) -> Self {
        Self::new(symbol)
    }
}

impl From<&Ticker> for SecCompanyProfileQuery {
    fn from(symbol: &Ticker) -> Self {
        Self::new(symbol.clone())
    }
}

impl QueryParameters for SecCompanyProfileQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.required("symbol", &self.symbol);
        encoder.optional("cik-A", self.cik_a.as_ref());
    }
}

/// Independent optional SIC code and title filters for the classification list.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IndustryClassificationsQuery {
    industry_title: Option<SearchTerm>,
    sic_code: Option<SearchTerm>,
}

impl IndustryClassificationsQuery {
    /// Creates an unfiltered classification-list query.
    pub const fn new() -> Self {
        Self {
            industry_title: None,
            sic_code: None,
        }
    }

    /// Sets the optional industry-title filter.
    pub fn with_industry_title(mut self, industry_title: SearchTerm) -> Self {
        self.industry_title = Some(industry_title);
        self
    }

    /// Sets the optional SIC-code filter.
    pub fn with_sic_code(mut self, sic_code: SearchTerm) -> Self {
        self.sic_code = Some(sic_code);
        self
    }

    /// Borrows the optional industry-title filter.
    pub const fn industry_title(&self) -> Option<&SearchTerm> {
        self.industry_title.as_ref()
    }

    /// Borrows the optional SIC-code filter.
    pub const fn sic_code(&self) -> Option<&SearchTerm> {
        self.sic_code.as_ref()
    }
}

impl QueryParameters for IndustryClassificationsQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.optional("industryTitle", self.industry_title.as_ref());
        encoder.optional("sicCode", self.sic_code.as_ref());
    }
}

/// Independent optional ticker, CIK, and SIC filters for raw classification search.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IndustryClassificationSearchQuery {
    symbol: Option<Ticker>,
    cik: Option<Cik>,
    sic_code: Option<SearchTerm>,
}

impl IndustryClassificationSearchQuery {
    /// Creates an unfiltered industry-classification search.
    pub const fn new() -> Self {
        Self {
            symbol: None,
            cik: None,
            sic_code: None,
        }
    }

    /// Sets the optional ticker filter.
    pub fn with_symbol(mut self, symbol: Ticker) -> Self {
        self.symbol = Some(symbol);
        self
    }

    /// Sets the optional CIK filter.
    pub fn with_cik(mut self, cik: Cik) -> Self {
        self.cik = Some(cik);
        self
    }

    /// Sets the optional SIC-code filter.
    pub fn with_sic_code(mut self, sic_code: SearchTerm) -> Self {
        self.sic_code = Some(sic_code);
        self
    }

    /// Borrows the optional ticker filter.
    pub const fn symbol(&self) -> Option<&Ticker> {
        self.symbol.as_ref()
    }

    /// Borrows the optional CIK filter.
    pub const fn cik(&self) -> Option<&Cik> {
        self.cik.as_ref()
    }

    /// Borrows the optional SIC-code filter.
    pub const fn sic_code(&self) -> Option<&SearchTerm> {
        self.sic_code.as_ref()
    }
}

impl QueryParameters for IndustryClassificationSearchQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.optional("symbol", self.symbol.as_ref());
        encoder.optional("cik", self.cik.as_ref());
        encoder.optional("sicCode", self.sic_code.as_ref());
    }
}

/// Optional pagination for the all-company industry-classification feed.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AllIndustryClassificationsQuery {
    page: Option<Page>,
    limit: Option<Limit>,
}

impl AllIndustryClassificationsQuery {
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

impl QueryParameters for AllIndustryClassificationsQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.optional("page", self.page);
        encoder.optional("limit", self.limit);
    }
}

const LATEST_FILING_METADATA: EndpointMetadata = EndpointMetadata::new()
    .with_geography(GeographicAvailability::UsOnly)
    .with_bounds(
        EndpointBounds::new()
            .with_response_rows(1_000)
            .with_page(100)
            .with_date_range_days(90),
    );
const FILING_SEARCH_METADATA: EndpointMetadata = EndpointMetadata::new()
    .with_geography(GeographicAvailability::UsOnly)
    .with_bounds(
        EndpointBounds::new()
            .with_response_rows(1_000)
            .with_page(100),
    );
const SEC_COMPANY_METADATA: EndpointMetadata =
    EndpointMetadata::new().with_geography(GeographicAvailability::UsOnly);

/// Describes `GET sec-filings-8k` without binding a transport.
pub fn latest_8k_sec_filings(
    query: Latest8kSecFilingsQuery,
) -> EndpointSpec<Latest8kSecFilingsQuery, Vec<SecFiling>> {
    EndpointSpec::get("sec-filings-8k", "sec-filings-8k", query)
        .with_metadata(LATEST_FILING_METADATA)
}

/// Describes `GET sec-filings-financials` without binding a transport.
pub fn latest_sec_filings(
    query: LatestSecFilingsQuery,
) -> EndpointSpec<LatestSecFilingsQuery, Vec<SecFiling>> {
    EndpointSpec::get("sec-filings-financials", "sec-filings-financials", query)
        .with_metadata(LATEST_FILING_METADATA)
}

/// Describes `GET sec-filings-search/form-type` without binding a transport.
pub fn sec_filings_by_form_type(
    query: SecFilingsByFormTypeQuery,
) -> EndpointSpec<SecFilingsByFormTypeQuery, Vec<SecFiling>> {
    EndpointSpec::get(
        "sec-filings-search/form-type",
        "sec-filings-search/form-type",
        query,
    )
    .with_metadata(FILING_SEARCH_METADATA)
}

/// Describes `GET sec-filings-search/symbol` without binding a transport.
pub fn sec_filings_by_symbol(
    query: SecFilingsBySymbolQuery,
) -> EndpointSpec<SecFilingsBySymbolQuery, Vec<SecFiling>> {
    EndpointSpec::get(
        "sec-filings-search/symbol",
        "sec-filings-search/symbol",
        query,
    )
    .with_metadata(FILING_SEARCH_METADATA)
}

/// Describes `GET sec-filings-search/cik` without binding a transport.
pub fn sec_filings_by_cik(
    query: SecFilingsByCikQuery,
) -> EndpointSpec<SecFilingsByCikQuery, Vec<SecFiling>> {
    EndpointSpec::get("sec-filings-search/cik", "sec-filings-search/cik", query)
        .with_metadata(FILING_SEARCH_METADATA)
}

/// Describes `GET sec-filings-company-search/name` without binding a transport.
pub fn search_sec_companies_by_name(
    query: SecCompaniesByNameQuery,
) -> EndpointSpec<SecCompaniesByNameQuery, Vec<SecCompanySearchResult>> {
    EndpointSpec::get(
        "sec-filings-company-search/name",
        "sec-filings-company-search/name",
        query,
    )
    .with_metadata(SEC_COMPANY_METADATA)
}

/// Describes `GET sec-filings-company-search/symbol` without binding a transport.
pub fn search_sec_companies_by_symbol(
    query: SecCompaniesBySymbolQuery,
) -> EndpointSpec<SecCompaniesBySymbolQuery, Vec<SecCompanySearchResult>> {
    EndpointSpec::get(
        "sec-filings-company-search/symbol",
        "sec-filings-company-search/symbol",
        query,
    )
    .with_metadata(SEC_COMPANY_METADATA)
}

/// Describes `GET sec-filings-company-search/cik` without binding a transport.
pub fn search_sec_companies_by_cik(
    query: SecCompaniesByCikQuery,
) -> EndpointSpec<SecCompaniesByCikQuery, Vec<SecCompanySearchResult>> {
    EndpointSpec::get(
        "sec-filings-company-search/cik",
        "sec-filings-company-search/cik",
        query,
    )
    .with_metadata(SEC_COMPANY_METADATA)
}

/// Describes `GET sec-profile` without binding a transport.
pub fn sec_company_profile(
    query: SecCompanyProfileQuery,
) -> EndpointSpec<SecCompanyProfileQuery, Vec<SecCompanyProfile>> {
    EndpointSpec::get("sec-profile", "sec-profile", query).with_metadata(SEC_COMPANY_METADATA)
}

/// Describes `GET standard-industrial-classification-list` without binding a transport.
pub fn industry_classifications(
    query: IndustryClassificationsQuery,
) -> EndpointSpec<IndustryClassificationsQuery, Vec<SicClassification>> {
    EndpointSpec::get(
        "standard-industrial-classification-list",
        "standard-industrial-classification-list",
        query,
    )
    .with_metadata(SEC_COMPANY_METADATA)
}

/// Describes `GET industry-classification-search` without binding a transport.
pub fn search_industry_classifications(
    query: IndustryClassificationSearchQuery,
) -> EndpointSpec<IndustryClassificationSearchQuery, Vec<DynamicObject>> {
    EndpointSpec::get(
        "industry-classification-search",
        "industry-classification-search",
        query,
    )
    .with_metadata(SEC_COMPANY_METADATA)
}

/// Describes `GET all-industry-classification` without binding a transport.
pub fn all_industry_classifications(
    query: AllIndustryClassificationsQuery,
) -> EndpointSpec<AllIndustryClassificationsQuery, Vec<SecCompanySearchResult>> {
    EndpointSpec::get(
        "all-industry-classification",
        "all-industry-classification",
        query,
    )
    .with_metadata(SEC_COMPANY_METADATA)
}

impl Client {
    /// Retrieves the latest US 8-K SEC filings within the required date range.
    pub async fn latest_8k_sec_filings(
        &self,
        query: impl Into<Latest8kSecFilingsQuery>,
    ) -> Result<Vec<SecFiling>> {
        self.execute(&latest_8k_sec_filings(query.into())).await
    }

    /// Retrieves the latest US financial SEC filings within the required date range.
    pub async fn latest_sec_filings(
        &self,
        query: impl Into<LatestSecFilingsQuery>,
    ) -> Result<Vec<SecFiling>> {
        self.execute(&latest_sec_filings(query.into())).await
    }

    /// Retrieves US SEC filings by their open form type.
    pub async fn sec_filings_by_form_type(
        &self,
        query: impl Into<SecFilingsByFormTypeQuery>,
    ) -> Result<Vec<SecFiling>> {
        self.execute(&sec_filings_by_form_type(query.into())).await
    }

    /// Retrieves US SEC filings for a ticker.
    pub async fn sec_filings_by_symbol(
        &self,
        query: impl Into<SecFilingsBySymbolQuery>,
    ) -> Result<Vec<SecFiling>> {
        self.execute(&sec_filings_by_symbol(query.into())).await
    }

    /// Retrieves US SEC filings for a string-backed CIK.
    pub async fn sec_filings_by_cik(
        &self,
        query: impl Into<SecFilingsByCikQuery>,
    ) -> Result<Vec<SecFiling>> {
        self.execute(&sec_filings_by_cik(query.into())).await
    }

    /// Searches US SEC companies by company-name text.
    pub async fn search_sec_companies_by_name(
        &self,
        query: impl Into<SecCompaniesByNameQuery>,
    ) -> Result<Vec<SecCompanySearchResult>> {
        self.execute(&search_sec_companies_by_name(query.into()))
            .await
    }

    /// Searches US SEC companies by ticker.
    pub async fn search_sec_companies_by_symbol(
        &self,
        query: impl Into<SecCompaniesBySymbolQuery>,
    ) -> Result<Vec<SecCompanySearchResult>> {
        self.execute(&search_sec_companies_by_symbol(query.into()))
            .await
    }

    /// Searches US SEC companies by string-backed CIK.
    pub async fn search_sec_companies_by_cik(
        &self,
        query: impl Into<SecCompaniesByCikQuery>,
    ) -> Result<Vec<SecCompanySearchResult>> {
        self.execute(&search_sec_companies_by_cik(query.into()))
            .await
    }

    /// Retrieves the full US SEC company profile for a ticker.
    pub async fn sec_company_profile(
        &self,
        query: impl Into<SecCompanyProfileQuery>,
    ) -> Result<Vec<SecCompanyProfile>> {
        self.execute(&sec_company_profile(query.into())).await
    }

    /// Retrieves the US Standard Industrial Classification directory.
    pub async fn industry_classifications(
        &self,
        query: impl Into<IndustryClassificationsQuery>,
    ) -> Result<Vec<SicClassification>> {
        self.execute(&industry_classifications(query.into())).await
    }

    /// Searches US industry classifications while preserving raw documented rows.
    pub async fn search_industry_classifications(
        &self,
        query: impl Into<IndustryClassificationSearchQuery>,
    ) -> Result<Vec<DynamicObject>> {
        self.execute(&search_industry_classifications(query.into()))
            .await
    }

    /// Retrieves the paginated US company industry-classification feed.
    pub async fn all_industry_classifications(
        &self,
        query: impl Into<AllIndustryClassificationsQuery>,
    ) -> Result<Vec<SecCompanySearchResult>> {
        self.execute(&all_industry_classifications(query.into()))
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pairs(query: &impl QueryParameters) -> Vec<(String, String)> {
        let mut pairs = Vec::new();
        query.encode(&mut QueryEncoder::new(&mut |name, value| {
            pairs.push((name.to_owned(), value.to_owned()));
        }));
        pairs
    }

    #[test]
    fn queries_emit_exact_keys_in_documented_order_and_omit_absent_values() {
        let from = Date::parse("2024-01-01").unwrap();
        let to = Date::parse("2024-03-01").unwrap();
        assert_eq!(
            pairs(&Latest8kSecFilingsQuery::new(from, to)),
            [
                ("from".into(), "2024-01-01".into()),
                ("to".into(), "2024-03-01".into())
            ]
        );
        assert_eq!(
            pairs(
                &LatestSecFilingsQuery::new(from, to)
                    .with_page(Page(0))
                    .with_limit(Limit(100))
            ),
            [
                ("from".into(), "2024-01-01".into()),
                ("to".into(), "2024-03-01".into()),
                ("page".into(), "0".into()),
                ("limit".into(), "100".into()),
            ]
        );
        assert_eq!(
            pairs(
                &SecFilingsByFormTypeQuery::new(FormType::new("8-K").unwrap(), from, to)
                    .with_page(Page(0))
                    .with_limit(Limit(u32::MAX))
            ),
            [
                ("formType".into(), "8-K".into()),
                ("from".into(), "2024-01-01".into()),
                ("to".into(), "2024-03-01".into()),
                ("page".into(), "0".into()),
                ("limit".into(), u32::MAX.to_string()),
            ]
        );
        assert_eq!(
            pairs(&SecFilingsBySymbolQuery::new(
                Ticker::new("AAPL").unwrap(),
                from,
                to,
            )),
            [
                ("symbol".into(), "AAPL".into()),
                ("from".into(), "2024-01-01".into()),
                ("to".into(), "2024-03-01".into()),
            ]
        );
        assert_eq!(
            pairs(
                &SecFilingsByCikQuery::new(Cik::new("0000320193").unwrap(), from, to,)
                    .with_page(Page(0))
                    .with_limit(Limit(100))
            ),
            [
                ("cik".into(), "0000320193".into()),
                ("from".into(), "2024-01-01".into()),
                ("to".into(), "2024-03-01".into()),
                ("page".into(), "0".into()),
                ("limit".into(), "100".into()),
            ]
        );
        assert_eq!(
            pairs(&SecCompaniesByNameQuery::new(
                SearchTerm::new("Berkshire").unwrap(),
            )),
            [("company".into(), "Berkshire".into())]
        );
        assert_eq!(
            pairs(&SecCompaniesBySymbolQuery::new(
                Ticker::new("AAPL").unwrap(),
            )),
            [("symbol".into(), "AAPL".into())]
        );
        assert_eq!(
            pairs(&SecCompaniesByCikQuery::new(
                Cik::new("0000320193").unwrap(),
            )),
            [("cik".into(), "0000320193".into())]
        );
        assert_eq!(
            pairs(
                &SecCompanyProfileQuery::new(Ticker::new("AAPL").unwrap())
                    .with_cik_a(Cik::new("0000320193").unwrap())
            ),
            [
                ("symbol".into(), "AAPL".into()),
                ("cik-A".into(), "0000320193".into()),
            ]
        );
        assert!(pairs(&IndustryClassificationsQuery::new()).is_empty());
        assert!(pairs(&IndustryClassificationSearchQuery::new()).is_empty());
        assert!(pairs(&AllIndustryClassificationsQuery::new()).is_empty());
        assert_eq!(
            pairs(
                &AllIndustryClassificationsQuery::new()
                    .with_page(Page(0))
                    .with_limit(Limit(100))
            ),
            [("page".into(), "0".into()), ("limit".into(), "100".into())]
        );
    }

    #[test]
    fn optional_classification_filters_remain_independent() {
        let list = IndustryClassificationsQuery::new()
            .with_industry_title(SearchTerm::new("SERVICES, NEC").unwrap())
            .with_sic_code(SearchTerm::new("07371").unwrap());
        assert_eq!(
            pairs(&list),
            [
                ("industryTitle".into(), "SERVICES, NEC".into()),
                ("sicCode".into(), "07371".into()),
            ]
        );

        let search = IndustryClassificationSearchQuery::new()
            .with_symbol(Ticker::new("000001.SZ").unwrap())
            .with_cik(Cik::new("0000320193").unwrap())
            .with_sic_code(SearchTerm::new("07371").unwrap());
        assert_eq!(
            pairs(&search),
            [
                ("symbol".into(), "000001.SZ".into()),
                ("cik".into(), "0000320193".into()),
                ("sicCode".into(), "07371".into()),
            ]
        );
    }
}
