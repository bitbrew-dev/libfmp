//! Crowdfunding and Regulation D fundraising endpoint contracts.
//!
//! Future Python bindings reserve the matching methods on `FmpClient`, with
//! response models under `fmp.fundraising`. This crate does not implement
//! those Python bindings.

use crate::{
    Client, Result,
    endpoints::{
        EndpointSpec, QueryEncoder, QueryParameters,
        metadata::{EndpointMetadata, GeographicAvailability},
    },
    responses::fundraising::{
        CrowdfundingOffering, CrowdfundingOfferingSearchResult, RegulationDOffering,
        RegulationDOfferingSearchResult,
    },
    types::{Cik, Limit, Page, SearchTerm},
};

/// Independently optional pagination for the latest crowdfunding offerings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LatestCrowdfundingOfferingsQuery {
    page: Option<Page>,
    limit: Option<Limit>,
}

impl LatestCrowdfundingOfferingsQuery {
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

impl QueryParameters for LatestCrowdfundingOfferingsQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.optional("page", self.page);
        encoder.optional("limit", self.limit);
    }
}

/// Independently optional pagination and issuer CIK for the latest Regulation D offerings.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LatestRegulationDOfferingsQuery {
    page: Option<Page>,
    limit: Option<Limit>,
    cik: Option<Cik>,
}

impl LatestRegulationDOfferingsQuery {
    /// Creates a query without undocumented defaults.
    pub const fn new() -> Self {
        Self {
            page: None,
            limit: None,
            cik: None,
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

    /// Restricts the latest feed to an optional issuer CIK.
    pub fn with_cik(mut self, cik: Cik) -> Self {
        self.cik = Some(cik);
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

    /// Borrows the optional issuer CIK.
    pub const fn cik(&self) -> Option<&Cik> {
        self.cik.as_ref()
    }
}

impl QueryParameters for LatestRegulationDOfferingsQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.optional("page", self.page);
        encoder.optional("limit", self.limit);
        encoder.optional("cik", self.cik.as_ref());
    }
}

/// Required issuer CIK shared by detailed fundraising lookups.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfferingByCikQuery {
    cik: Cik,
}

impl OfferingByCikQuery {
    /// Creates an offering lookup for one issuer CIK.
    pub const fn new(cik: Cik) -> Self {
        Self { cik }
    }

    /// Borrows the required issuer CIK.
    pub const fn cik(&self) -> &Cik {
        &self.cik
    }
}

impl From<Cik> for OfferingByCikQuery {
    fn from(cik: Cik) -> Self {
        Self::new(cik)
    }
}

impl From<&Cik> for OfferingByCikQuery {
    fn from(cik: &Cik) -> Self {
        Self::new(cik.clone())
    }
}

impl QueryParameters for OfferingByCikQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.required("cik", &self.cik);
    }
}

/// Required company, campaign, platform, or symbol text for offering searches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfferingSearchQuery {
    name: SearchTerm,
}

impl OfferingSearchQuery {
    /// Creates an offering search from its required representation-preserving text.
    pub const fn new(name: SearchTerm) -> Self {
        Self { name }
    }

    /// Borrows the required provider search text.
    pub const fn name(&self) -> &SearchTerm {
        &self.name
    }
}

impl From<SearchTerm> for OfferingSearchQuery {
    fn from(name: SearchTerm) -> Self {
        Self::new(name)
    }
}

impl From<&SearchTerm> for OfferingSearchQuery {
    fn from(name: &SearchTerm) -> Self {
        Self::new(name.clone())
    }
}

impl QueryParameters for OfferingSearchQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.required("name", &self.name);
    }
}

const US_ONLY: EndpointMetadata =
    EndpointMetadata::new().with_geography(GeographicAvailability::UsOnly);

/// Describes `GET crowdfunding-offerings-search` without binding a transport.
pub fn search_crowdfunding_offerings(
    query: OfferingSearchQuery,
) -> EndpointSpec<OfferingSearchQuery, Vec<CrowdfundingOfferingSearchResult>> {
    EndpointSpec::get(
        "crowdfunding-offerings-search",
        "crowdfunding-offerings-search",
        query,
    )
    .with_metadata(US_ONLY)
}

/// Describes `GET crowdfunding-offerings-latest` without binding a transport.
pub fn latest_crowdfunding_offerings(
    query: LatestCrowdfundingOfferingsQuery,
) -> EndpointSpec<LatestCrowdfundingOfferingsQuery, Vec<CrowdfundingOffering>> {
    EndpointSpec::get(
        "crowdfunding-offerings-latest",
        "crowdfunding-offerings-latest",
        query,
    )
    .with_metadata(US_ONLY)
}

/// Describes `GET crowdfunding-offerings` without binding a transport.
pub fn crowdfunding_offerings_by_cik(
    query: OfferingByCikQuery,
) -> EndpointSpec<OfferingByCikQuery, Vec<CrowdfundingOffering>> {
    EndpointSpec::get("crowdfunding-offerings", "crowdfunding-offerings", query)
        .with_metadata(US_ONLY)
}

/// Describes `GET fundraising-search` without binding a transport.
pub fn search_regulation_d_offerings(
    query: OfferingSearchQuery,
) -> EndpointSpec<OfferingSearchQuery, Vec<RegulationDOfferingSearchResult>> {
    EndpointSpec::get("fundraising-search", "fundraising-search", query).with_metadata(US_ONLY)
}

/// Describes `GET fundraising-latest` without binding a transport.
pub fn latest_regulation_d_offerings(
    query: LatestRegulationDOfferingsQuery,
) -> EndpointSpec<LatestRegulationDOfferingsQuery, Vec<RegulationDOffering>> {
    EndpointSpec::get("fundraising-latest", "fundraising-latest", query).with_metadata(US_ONLY)
}

/// Describes `GET fundraising` without binding a transport.
pub fn regulation_d_offerings_by_cik(
    query: OfferingByCikQuery,
) -> EndpointSpec<OfferingByCikQuery, Vec<RegulationDOffering>> {
    EndpointSpec::get("fundraising", "fundraising", query).with_metadata(US_ONLY)
}

impl Client {
    /// Retrieves the latest US crowdfunding offerings.
    pub async fn latest_crowdfunding_offerings(
        &self,
        query: LatestCrowdfundingOfferingsQuery,
    ) -> Result<Vec<CrowdfundingOffering>> {
        self.execute(&latest_crowdfunding_offerings(query)).await
    }

    /// Retrieves US crowdfunding offerings for one issuer CIK.
    pub async fn crowdfunding_offerings_by_cik(
        &self,
        query: impl Into<OfferingByCikQuery>,
    ) -> Result<Vec<CrowdfundingOffering>> {
        self.execute(&crowdfunding_offerings_by_cik(query.into()))
            .await
    }

    /// Searches US crowdfunding offerings by company, campaign, or platform text.
    pub async fn search_crowdfunding_offerings(
        &self,
        query: impl Into<OfferingSearchQuery>,
    ) -> Result<Vec<CrowdfundingOfferingSearchResult>> {
        self.execute(&search_crowdfunding_offerings(query.into()))
            .await
    }

    /// Searches US Regulation D offerings by company name or stock symbol.
    pub async fn search_regulation_d_offerings(
        &self,
        query: impl Into<OfferingSearchQuery>,
    ) -> Result<Vec<RegulationDOfferingSearchResult>> {
        self.execute(&search_regulation_d_offerings(query.into()))
            .await
    }

    /// Retrieves the latest US Regulation D offerings.
    pub async fn latest_regulation_d_offerings(
        &self,
        query: LatestRegulationDOfferingsQuery,
    ) -> Result<Vec<RegulationDOffering>> {
        self.execute(&latest_regulation_d_offerings(query)).await
    }

    /// Retrieves US Regulation D offerings for one issuer CIK.
    pub async fn regulation_d_offerings_by_cik(
        &self,
        query: impl Into<OfferingByCikQuery>,
    ) -> Result<Vec<RegulationDOffering>> {
        self.execute(&regulation_d_offerings_by_cik(query.into()))
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encoded(query: &impl QueryParameters) -> Vec<(String, String)> {
        let mut pairs = Vec::new();
        let mut visitor = |name: &str, value: &str| {
            pairs.push((name.to_owned(), value.to_owned()));
        };
        query.encode(&mut QueryEncoder::new(&mut visitor));
        pairs
    }

    #[test]
    fn query_encodes_only_the_exact_required_name_key() {
        let name = SearchTerm::new("NJOY / Class A").unwrap();
        let owned = OfferingSearchQuery::from(name.clone());
        let borrowed = OfferingSearchQuery::from(&name);

        assert_eq!(owned.name(), &name);
        assert_eq!(borrowed.name(), &name);
        assert_eq!(
            encoded(&borrowed),
            [("name".into(), "NJOY / Class A".into())]
        );
    }

    #[test]
    fn detailed_queries_encode_exact_keys_order_and_omissions() {
        assert!(encoded(&LatestCrowdfundingOfferingsQuery::new()).is_empty());
        assert_eq!(
            encoded(
                &LatestCrowdfundingOfferingsQuery::new()
                    .with_page(Page(0))
                    .with_limit(Limit(u32::MAX))
            ),
            [
                ("page".into(), "0".into()),
                ("limit".into(), u32::MAX.to_string()),
            ]
        );
        assert_eq!(
            encoded(&LatestCrowdfundingOfferingsQuery::new().with_limit(Limit(0))),
            [("limit".into(), "0".into())]
        );

        let cik = Cik::new("0001916078").unwrap();
        let owned = OfferingByCikQuery::from(cik.clone());
        let borrowed = OfferingByCikQuery::from(&cik);
        assert_eq!(owned.cik(), &cik);
        assert_eq!(borrowed.cik(), &cik);
        assert_eq!(encoded(&borrowed), [("cik".into(), "0001916078".into())]);

        assert!(encoded(&LatestRegulationDOfferingsQuery::new()).is_empty());
        let latest = LatestRegulationDOfferingsQuery::new()
            .with_page(Page(u32::MAX))
            .with_limit(Limit(0))
            .with_cik(Cik::new("0002013736").unwrap());
        assert_eq!(
            encoded(&latest),
            [
                ("page".into(), u32::MAX.to_string()),
                ("limit".into(), "0".into()),
                ("cik".into(), "0002013736".into()),
            ]
        );
        assert_eq!(
            encoded(
                &LatestRegulationDOfferingsQuery::new().with_cik(Cik::new("0002013736").unwrap())
            ),
            [("cik".into(), "0002013736".into())]
        );
    }
}
