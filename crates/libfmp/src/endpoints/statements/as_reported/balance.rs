//! Balance-sheet-statement-as-reported endpoint.

use crate::{
    Client, Result, endpoints::EndpointSpec, responses::statements::AsReportedFinancialStatement,
};

use super::{AS_REPORTED_METADATA, BalanceSheetStatementAsReportedQuery};

/// Describes `GET balance-sheet-statement-as-reported` without binding it to a transport.
pub fn balance_sheet_statement_as_reported(
    query: BalanceSheetStatementAsReportedQuery,
) -> EndpointSpec<BalanceSheetStatementAsReportedQuery, Vec<AsReportedFinancialStatement>> {
    EndpointSpec::get(
        "balance-sheet-statement-as-reported",
        "balance-sheet-statement-as-reported",
        query,
    )
    .with_metadata(AS_REPORTED_METADATA)
}

impl Client {
    /// Retrieves balance sheets as reported by one company.
    pub async fn balance_sheet_statement_as_reported(
        &self,
        query: impl Into<BalanceSheetStatementAsReportedQuery>,
    ) -> Result<Vec<AsReportedFinancialStatement>> {
        self.execute(&balance_sheet_statement_as_reported(query.into()))
            .await
    }
}
