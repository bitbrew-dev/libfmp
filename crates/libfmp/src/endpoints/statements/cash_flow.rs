//! Historical and trailing-twelve-month cash-flow-statement endpoints.

use crate::{Client, Result, endpoints::EndpointSpec, responses::statements::CashFlowStatement};

use super::{CashFlowStatementQuery, CashFlowStatementTtmQuery, WORLDWIDE_STATEMENT_METADATA};

/// Describes `GET cash-flow-statement` without binding it to a transport.
pub fn cash_flow_statement(
    query: CashFlowStatementQuery,
) -> EndpointSpec<CashFlowStatementQuery, Vec<CashFlowStatement>> {
    EndpointSpec::get("cash-flow-statement", "cash-flow-statement", query)
        .with_metadata(WORLDWIDE_STATEMENT_METADATA)
}

/// Describes `GET cash-flow-statement-ttm` without binding it to a transport.
pub fn cash_flow_statement_ttm(
    query: CashFlowStatementTtmQuery,
) -> EndpointSpec<CashFlowStatementTtmQuery, Vec<CashFlowStatement>> {
    EndpointSpec::get("cash-flow-statement-ttm", "cash-flow-statement-ttm", query)
        .with_metadata(WORLDWIDE_STATEMENT_METADATA)
}

impl Client {
    /// Retrieves historical worldwide cash-flow statements for one company.
    pub async fn cash_flow_statement(
        &self,
        query: impl Into<CashFlowStatementQuery>,
    ) -> Result<Vec<CashFlowStatement>> {
        self.execute(&cash_flow_statement(query.into())).await
    }

    /// Retrieves trailing-twelve-month worldwide cash-flow statements for one company.
    pub async fn cash_flow_statement_ttm(
        &self,
        query: impl Into<CashFlowStatementTtmQuery>,
    ) -> Result<Vec<CashFlowStatement>> {
        self.execute(&cash_flow_statement_ttm(query.into())).await
    }
}
