//! Combined financial-statement growth endpoint.

use crate::{
    Client, Result,
    endpoints::{EndpointSpec, statements::WORLDWIDE_STATEMENT_METADATA},
    responses::statements::FinancialStatementGrowth,
};

use super::FinancialStatementGrowthQuery;

/// Describes `GET financial-growth` without binding a transport.
pub fn financial_statement_growth(
    query: FinancialStatementGrowthQuery,
) -> EndpointSpec<FinancialStatementGrowthQuery, Vec<FinancialStatementGrowth>> {
    EndpointSpec::get("financial-growth", "financial-growth", query)
        .with_metadata(WORLDWIDE_STATEMENT_METADATA)
}

impl Client {
    /// Retrieves combined worldwide financial-statement growth for one company.
    pub async fn financial_statement_growth(
        &self,
        query: impl Into<FinancialStatementGrowthQuery>,
    ) -> Result<Vec<FinancialStatementGrowth>> {
        self.execute(&financial_statement_growth(query.into()))
            .await
    }
}
