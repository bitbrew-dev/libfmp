//! Bulk endpoint contracts.
//!
//! A future Python binding reserves the matching `FmpClient` methods with
//! response models under `fmp.bulk`. This crate does not implement those bindings.

use crate::{
    Client, Result,
    endpoints::{
        EndpointSpec, QueryEncoder, QueryParameters,
        metadata::{EndpointMetadata, GeographicAvailability},
    },
    responses::{
        bulk::{
            BulkDcfValuation, BulkEtfHolding, BulkFinancialScore, BulkPriceTargetSummary,
            BulkStockRating, BulkUpgradesDowngradesConsensus,
        },
        company::CompanyProfile,
    },
    types::BulkPart,
};

/// Required provider partition shared by partitioned bulk routes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BulkPartQuery {
    part: BulkPart,
}

impl BulkPartQuery {
    /// Creates a partitioned bulk query without inferring a numeric range.
    pub const fn new(part: BulkPart) -> Self {
        Self { part }
    }

    /// Borrows the original provider partition representation.
    pub const fn part(&self) -> &BulkPart {
        &self.part
    }
}

impl From<BulkPart> for BulkPartQuery {
    fn from(part: BulkPart) -> Self {
        Self::new(part)
    }
}

impl From<&BulkPart> for BulkPartQuery {
    fn from(part: &BulkPart) -> Self {
        Self::new(part.clone())
    }
}

impl QueryParameters for BulkPartQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.required("part", &self.part);
    }
}

const WORLDWIDE: EndpointMetadata =
    EndpointMetadata::new().with_geography(GeographicAvailability::Worldwide);
const US_ONLY: EndpointMetadata =
    EndpointMetadata::new().with_geography(GeographicAvailability::UsOnly);

/// Describes `GET profile-bulk` without binding a transport.
pub fn bulk_company_profiles(
    query: BulkPartQuery,
) -> EndpointSpec<BulkPartQuery, Vec<CompanyProfile>> {
    EndpointSpec::get("profile-bulk", "profile-bulk", query).with_metadata(WORLDWIDE)
}

/// Describes `GET rating-bulk` without binding a transport.
pub fn bulk_stock_ratings() -> EndpointSpec<(), Vec<BulkStockRating>> {
    EndpointSpec::get("rating-bulk", "rating-bulk", ()).with_metadata(WORLDWIDE)
}

/// Describes `GET dcf-bulk` without binding a transport.
pub fn bulk_dcf_valuations() -> EndpointSpec<(), Vec<BulkDcfValuation>> {
    EndpointSpec::get("dcf-bulk", "dcf-bulk", ()).with_metadata(WORLDWIDE)
}

/// Describes `GET scores-bulk` without binding a transport.
pub fn bulk_financial_scores() -> EndpointSpec<(), Vec<BulkFinancialScore>> {
    EndpointSpec::get("scores-bulk", "scores-bulk", ()).with_metadata(WORLDWIDE)
}

/// Describes `GET price-target-summary-bulk` without binding a transport.
pub fn bulk_price_target_summaries() -> EndpointSpec<(), Vec<BulkPriceTargetSummary>> {
    EndpointSpec::get("price-target-summary-bulk", "price-target-summary-bulk", ())
        .with_metadata(US_ONLY)
}

/// Describes `GET etf-holder-bulk` without binding a transport.
pub fn bulk_etf_holdings(query: BulkPartQuery) -> EndpointSpec<BulkPartQuery, Vec<BulkEtfHolding>> {
    EndpointSpec::get("etf-holder-bulk", "etf-holder-bulk", query).with_metadata(WORLDWIDE)
}

/// Describes `GET upgrades-downgrades-consensus-bulk` without binding a transport.
pub fn bulk_upgrades_downgrades_consensus() -> EndpointSpec<(), Vec<BulkUpgradesDowngradesConsensus>>
{
    EndpointSpec::get(
        "upgrades-downgrades-consensus-bulk",
        "upgrades-downgrades-consensus-bulk",
        (),
    )
    .with_metadata(WORLDWIDE)
}

impl Client {
    /// Retrieves one provider partition of worldwide company profiles.
    pub async fn bulk_company_profiles(
        &self,
        query: impl Into<BulkPartQuery>,
    ) -> Result<Vec<CompanyProfile>> {
        self.execute(&bulk_company_profiles(query.into())).await
    }

    /// Retrieves worldwide stock ratings in one provider bulk response.
    pub async fn bulk_stock_ratings(&self) -> Result<Vec<BulkStockRating>> {
        self.execute(&bulk_stock_ratings()).await
    }

    /// Retrieves worldwide discounted-cash-flow valuations in one provider bulk response.
    pub async fn bulk_dcf_valuations(&self) -> Result<Vec<BulkDcfValuation>> {
        self.execute(&bulk_dcf_valuations()).await
    }

    /// Retrieves worldwide financial scores in one provider bulk response.
    pub async fn bulk_financial_scores(&self) -> Result<Vec<BulkFinancialScore>> {
        self.execute(&bulk_financial_scores()).await
    }

    /// Retrieves US price-target summaries in one provider bulk response.
    pub async fn bulk_price_target_summaries(&self) -> Result<Vec<BulkPriceTargetSummary>> {
        self.execute(&bulk_price_target_summaries()).await
    }

    /// Retrieves one provider partition of worldwide ETF holdings.
    pub async fn bulk_etf_holdings(
        &self,
        query: impl Into<BulkPartQuery>,
    ) -> Result<Vec<BulkEtfHolding>> {
        self.execute(&bulk_etf_holdings(query.into())).await
    }

    /// Retrieves worldwide upgrades/downgrades consensus in one provider bulk response.
    pub async fn bulk_upgrades_downgrades_consensus(
        &self,
    ) -> Result<Vec<BulkUpgradesDowngradesConsensus>> {
        self.execute(&bulk_upgrades_downgrades_consensus()).await
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
    fn bulk_part_query_has_one_exact_required_key_and_borrowed_conversion() {
        let part = BulkPart::new("segment 01/alpha").unwrap();
        let owned = BulkPartQuery::from(part.clone());
        let borrowed = BulkPartQuery::from(&part);

        assert_eq!(owned.part(), &part);
        assert_eq!(borrowed.part(), &part);
        assert_eq!(
            encoded(&borrowed),
            [("part".into(), "segment 01/alpha".into())]
        );
    }
}
