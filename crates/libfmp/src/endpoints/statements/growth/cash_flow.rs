//! Cash-flow-statement growth endpoint.

use crate::{
    Client, Result,
    endpoints::{EndpointSpec, statements::WORLDWIDE_STATEMENT_METADATA},
    responses::statements::CashFlowStatementGrowth,
};

use super::CashFlowStatementGrowthQuery;

/// Describes `GET cash-flow-statement-growth` without binding a transport.
pub fn cash_flow_statement_growth(
    query: CashFlowStatementGrowthQuery,
) -> EndpointSpec<CashFlowStatementGrowthQuery, Vec<CashFlowStatementGrowth>> {
    EndpointSpec::get(
        "cash-flow-statement-growth",
        "cash-flow-statement-growth",
        query,
    )
    .with_metadata(WORLDWIDE_STATEMENT_METADATA)
}

impl Client {
    /// Retrieves worldwide cash-flow-statement growth for one company.
    pub async fn cash_flow_statement_growth(
        &self,
        query: impl Into<CashFlowStatementGrowthQuery>,
    ) -> Result<Vec<CashFlowStatementGrowth>> {
        self.execute(&cash_flow_statement_growth(query.into()))
            .await
    }
}
