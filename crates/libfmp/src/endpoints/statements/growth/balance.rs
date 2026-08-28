//! Balance-sheet-statement-growth endpoint.

use crate::{
    Client, Result, endpoints::EndpointSpec, responses::statements::BalanceSheetStatementGrowth,
};

use super::BalanceSheetStatementGrowthQuery;
use crate::endpoints::statements::WORLDWIDE_STATEMENT_METADATA;

/// Describes `GET balance-sheet-statement-growth` without binding it to a transport.
pub fn balance_sheet_statement_growth(
    query: BalanceSheetStatementGrowthQuery,
) -> EndpointSpec<BalanceSheetStatementGrowthQuery, Vec<BalanceSheetStatementGrowth>> {
    EndpointSpec::get(
        "balance-sheet-statement-growth",
        "balance-sheet-statement-growth",
        query,
    )
    .with_metadata(WORLDWIDE_STATEMENT_METADATA)
}

impl Client {
    /// Retrieves worldwide balance-sheet-statement growth for one company.
    pub async fn balance_sheet_statement_growth(
        &self,
        query: impl Into<BalanceSheetStatementGrowthQuery>,
    ) -> Result<Vec<BalanceSheetStatementGrowth>> {
        self.execute(&balance_sheet_statement_growth(query.into()))
            .await
    }
}
