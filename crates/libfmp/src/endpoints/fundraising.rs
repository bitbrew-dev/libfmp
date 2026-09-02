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
    responses::fundraising::{CrowdfundingOfferingSearchResult, RegulationDOfferingSearchResult},
    types::SearchTerm,
};

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

/// Describes `GET fundraising-search` without binding a transport.
pub fn search_regulation_d_offerings(
    query: OfferingSearchQuery,
) -> EndpointSpec<OfferingSearchQuery, Vec<RegulationDOfferingSearchResult>> {
    EndpointSpec::get("fundraising-search", "fundraising-search", query).with_metadata(US_ONLY)
}

impl Client {
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
}
