//! Cash-flow-statement-as-reported endpoint.

use crate::{
    Client, Result, endpoints::EndpointSpec, responses::statements::AsReportedFinancialStatement,
};

use super::{AS_REPORTED_METADATA, CashFlowStatementAsReportedQuery};

/// Describes `GET cash-flow-statement-as-reported` without binding a transport.
pub fn cash_flow_statement_as_reported(
    query: CashFlowStatementAsReportedQuery,
) -> EndpointSpec<CashFlowStatementAsReportedQuery, Vec<AsReportedFinancialStatement>> {
    EndpointSpec::get(
        "cash-flow-statement-as-reported",
        "cash-flow-statement-as-reported",
        query,
    )
    .with_metadata(AS_REPORTED_METADATA)
}

impl Client {
    /// Retrieves cash-flow statements as reported by one company.
    pub async fn cash_flow_statement_as_reported(
        &self,
        query: impl Into<CashFlowStatementAsReportedQuery>,
    ) -> Result<Vec<AsReportedFinancialStatement>> {
        self.execute(&cash_flow_statement_as_reported(query.into()))
            .await
    }
}
