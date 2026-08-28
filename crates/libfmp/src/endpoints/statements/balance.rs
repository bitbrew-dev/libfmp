//! Historical and trailing-twelve-month balance-sheet endpoints.

use crate::{
    Client, Result,
    endpoints::EndpointSpec,
    responses::statements::{BalanceSheetStatement, BalanceSheetStatementTtm},
};

use super::{
    BalanceSheetStatementQuery, BalanceSheetStatementTtmQuery, WORLDWIDE_STATEMENT_METADATA,
};

/// Describes `GET balance-sheet-statement` without binding it to a transport.
pub fn balance_sheet_statement(
    query: BalanceSheetStatementQuery,
) -> EndpointSpec<BalanceSheetStatementQuery, Vec<BalanceSheetStatement>> {
    EndpointSpec::get("balance-sheet-statement", "balance-sheet-statement", query)
        .with_metadata(WORLDWIDE_STATEMENT_METADATA)
}

/// Describes `GET balance-sheet-statement-ttm` without binding it to a transport.
pub fn balance_sheet_statement_ttm(
    query: BalanceSheetStatementTtmQuery,
) -> EndpointSpec<BalanceSheetStatementTtmQuery, Vec<BalanceSheetStatementTtm>> {
    EndpointSpec::get(
        "balance-sheet-statement-ttm",
        "balance-sheet-statement-ttm",
        query,
    )
    .with_metadata(WORLDWIDE_STATEMENT_METADATA)
}

impl Client {
    /// Retrieves historical worldwide balance sheets for one company.
    pub async fn balance_sheet_statement(
        &self,
        query: impl Into<BalanceSheetStatementQuery>,
    ) -> Result<Vec<BalanceSheetStatement>> {
        self.execute(&balance_sheet_statement(query.into())).await
    }

    /// Retrieves trailing-twelve-month worldwide balance sheets for one company.
    pub async fn balance_sheet_statement_ttm(
        &self,
        query: impl Into<BalanceSheetStatementTtmQuery>,
    ) -> Result<Vec<BalanceSheetStatementTtm>> {
        self.execute(&balance_sheet_statement_ttm(query.into()))
            .await
    }
}
