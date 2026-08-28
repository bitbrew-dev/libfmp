//! Income-statement-growth endpoint.

use crate::{
    Client, Result, endpoints::EndpointSpec, responses::statements::IncomeStatementGrowth,
};

use super::IncomeStatementGrowthQuery;
use crate::endpoints::statements::WORLDWIDE_STATEMENT_METADATA;

/// Describes `GET income-statement-growth` without binding it to a transport.
pub fn income_statement_growth(
    query: IncomeStatementGrowthQuery,
) -> EndpointSpec<IncomeStatementGrowthQuery, Vec<IncomeStatementGrowth>> {
    EndpointSpec::get("income-statement-growth", "income-statement-growth", query)
        .with_metadata(WORLDWIDE_STATEMENT_METADATA)
}

impl Client {
    /// Retrieves worldwide income-statement growth for one company.
    pub async fn income_statement_growth(
        &self,
        query: impl Into<IncomeStatementGrowthQuery>,
    ) -> Result<Vec<IncomeStatementGrowth>> {
        self.execute(&income_statement_growth(query.into())).await
    }
}
