//! Income-statement-as-reported endpoint.

use crate::{
    Client, Result, endpoints::EndpointSpec, responses::statements::AsReportedFinancialStatement,
};

use super::{AS_REPORTED_METADATA, IncomeStatementAsReportedQuery};

/// Describes `GET income-statement-as-reported` without binding it to a transport.
pub fn income_statement_as_reported(
    query: IncomeStatementAsReportedQuery,
) -> EndpointSpec<IncomeStatementAsReportedQuery, Vec<AsReportedFinancialStatement>> {
    EndpointSpec::get(
        "income-statement-as-reported",
        "income-statement-as-reported",
        query,
    )
    .with_metadata(AS_REPORTED_METADATA)
}

impl Client {
    /// Retrieves income statements as reported by one company.
    pub async fn income_statement_as_reported(
        &self,
        query: impl Into<IncomeStatementAsReportedQuery>,
    ) -> Result<Vec<AsReportedFinancialStatement>> {
        self.execute(&income_statement_as_reported(query.into()))
            .await
    }
}
