//! Historical and trailing-twelve-month income-statement endpoints.

use crate::{Client, Result, endpoints::EndpointSpec, responses::statements::IncomeStatement};

use super::{IncomeStatementQuery, IncomeStatementTtmQuery, WORLDWIDE_STATEMENT_METADATA};

/// Describes `GET income-statement` without binding it to a transport.
pub fn income_statement(
    query: IncomeStatementQuery,
) -> EndpointSpec<IncomeStatementQuery, Vec<IncomeStatement>> {
    EndpointSpec::get("income-statement", "income-statement", query)
        .with_metadata(WORLDWIDE_STATEMENT_METADATA)
}

/// Describes `GET income-statement-ttm` without binding it to a transport.
pub fn income_statement_ttm(
    query: IncomeStatementTtmQuery,
) -> EndpointSpec<IncomeStatementTtmQuery, Vec<IncomeStatement>> {
    EndpointSpec::get("income-statement-ttm", "income-statement-ttm", query)
        .with_metadata(WORLDWIDE_STATEMENT_METADATA)
}

impl Client {
    /// Retrieves historical worldwide income statements for one company.
    pub async fn income_statement(
        &self,
        query: impl Into<IncomeStatementQuery>,
    ) -> Result<Vec<IncomeStatement>> {
        self.execute(&income_statement(query.into())).await
    }

    /// Retrieves trailing-twelve-month worldwide income statements for one company.
    pub async fn income_statement_ttm(
        &self,
        query: impl Into<IncomeStatementTtmQuery>,
    ) -> Result<Vec<IncomeStatement>> {
        self.execute(&income_statement_ttm(query.into())).await
    }
}
