//! Complete-financial-statement-as-reported endpoint.

use crate::{
    Client, Result, endpoints::EndpointSpec, responses::statements::AsReportedFinancialStatement,
};

use super::{AS_REPORTED_METADATA, FinancialStatementFullAsReportedQuery};

/// Describes `GET financial-statement-full-as-reported` without binding a transport.
pub fn financial_statement_full_as_reported(
    query: FinancialStatementFullAsReportedQuery,
) -> EndpointSpec<FinancialStatementFullAsReportedQuery, Vec<AsReportedFinancialStatement>> {
    EndpointSpec::get(
        "financial-statement-full-as-reported",
        "financial-statement-full-as-reported",
        query,
    )
    .with_metadata(AS_REPORTED_METADATA)
}

impl Client {
    /// Retrieves complete financial statements as reported by one company.
    pub async fn financial_statement_full_as_reported(
        &self,
        query: impl Into<FinancialStatementFullAsReportedQuery>,
    ) -> Result<Vec<AsReportedFinancialStatement>> {
        self.execute(&financial_statement_full_as_reported(query.into()))
            .await
    }
}
