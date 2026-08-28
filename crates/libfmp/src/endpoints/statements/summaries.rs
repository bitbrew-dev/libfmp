//! Compact worldwide financial-summary endpoints.

use crate::{
    Client, Result,
    endpoints::EndpointSpec,
    responses::statements::{
        EnterpriseValue, FinancialScore, LatestFinancialStatement, OwnerEarnings,
    },
};

use super::{
    EnterpriseValuesQuery, FinancialScoresQuery, LATEST_FINANCIAL_STATEMENTS_METADATA,
    LatestFinancialStatementsQuery, OwnerEarningsQuery, WORLDWIDE_FINANCIAL_HISTORY_METADATA,
    WORLDWIDE_FINANCIAL_SUMMARY_METADATA,
};

/// Describes `GET latest-financial-statements` without binding it to a transport.
pub fn latest_financial_statements(
    query: LatestFinancialStatementsQuery,
) -> EndpointSpec<LatestFinancialStatementsQuery, Vec<LatestFinancialStatement>> {
    EndpointSpec::get(
        "latest-financial-statements",
        "latest-financial-statements",
        query,
    )
    .with_metadata(LATEST_FINANCIAL_STATEMENTS_METADATA)
}

/// Describes `GET financial-scores` without binding it to a transport.
pub fn financial_scores(
    query: FinancialScoresQuery,
) -> EndpointSpec<FinancialScoresQuery, Vec<FinancialScore>> {
    EndpointSpec::get("financial-scores", "financial-scores", query)
        .with_metadata(WORLDWIDE_FINANCIAL_SUMMARY_METADATA)
}

/// Describes `GET owner-earnings` without binding it to a transport.
pub fn owner_earnings(
    query: OwnerEarningsQuery,
) -> EndpointSpec<OwnerEarningsQuery, Vec<OwnerEarnings>> {
    EndpointSpec::get("owner-earnings", "owner-earnings", query)
        .with_metadata(WORLDWIDE_FINANCIAL_SUMMARY_METADATA)
}

/// Describes `GET enterprise-values` without binding it to a transport.
pub fn enterprise_values(
    query: EnterpriseValuesQuery,
) -> EndpointSpec<EnterpriseValuesQuery, Vec<EnterpriseValue>> {
    EndpointSpec::get("enterprise-values", "enterprise-values", query)
        .with_metadata(WORLDWIDE_FINANCIAL_HISTORY_METADATA)
}

impl Client {
    /// Retrieves the latest worldwide financial-statement filings.
    pub async fn latest_financial_statements(
        &self,
        query: LatestFinancialStatementsQuery,
    ) -> Result<Vec<LatestFinancialStatement>> {
        self.execute(&latest_financial_statements(query)).await
    }

    /// Retrieves worldwide financial-health scores for one company.
    pub async fn financial_scores(
        &self,
        query: impl Into<FinancialScoresQuery>,
    ) -> Result<Vec<FinancialScore>> {
        self.execute(&financial_scores(query.into())).await
    }

    /// Retrieves worldwide owner earnings for one company.
    pub async fn owner_earnings(
        &self,
        query: impl Into<OwnerEarningsQuery>,
    ) -> Result<Vec<OwnerEarnings>> {
        self.execute(&owner_earnings(query.into())).await
    }

    /// Retrieves worldwide enterprise values for one company.
    pub async fn enterprise_values(
        &self,
        query: impl Into<EnterpriseValuesQuery>,
    ) -> Result<Vec<EnterpriseValue>> {
        self.execute(&enterprise_values(query.into())).await
    }
}
