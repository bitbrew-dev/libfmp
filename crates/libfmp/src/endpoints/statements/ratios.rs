//! Historical and trailing-twelve-month financial-ratio endpoints.

use crate::{
    Client, Result,
    endpoints::EndpointSpec,
    responses::statements::{FinancialRatios, FinancialRatiosTtm},
};

use super::{
    FinancialRatiosQuery, FinancialRatiosTtmQuery, WORLDWIDE_FINANCIAL_HISTORY_METADATA,
    WORLDWIDE_FINANCIAL_SUMMARY_METADATA,
};

/// Describes `GET ratios` without binding it to a transport.
pub fn financial_ratios(
    query: FinancialRatiosQuery,
) -> EndpointSpec<FinancialRatiosQuery, Vec<FinancialRatios>> {
    EndpointSpec::get("ratios", "ratios", query).with_metadata(WORLDWIDE_FINANCIAL_HISTORY_METADATA)
}

/// Describes `GET ratios-ttm` without binding it to a transport.
pub fn financial_ratios_ttm(
    query: FinancialRatiosTtmQuery,
) -> EndpointSpec<FinancialRatiosTtmQuery, Vec<FinancialRatiosTtm>> {
    EndpointSpec::get("ratios-ttm", "ratios-ttm", query)
        .with_metadata(WORLDWIDE_FINANCIAL_SUMMARY_METADATA)
}

impl Client {
    /// Retrieves historical worldwide financial ratios for one company.
    pub async fn financial_ratios(
        &self,
        query: impl Into<FinancialRatiosQuery>,
    ) -> Result<Vec<FinancialRatios>> {
        self.execute(&financial_ratios(query.into())).await
    }

    /// Retrieves trailing-twelve-month worldwide financial ratios for one company.
    pub async fn financial_ratios_ttm(
        &self,
        query: impl Into<FinancialRatiosTtmQuery>,
    ) -> Result<Vec<FinancialRatiosTtm>> {
        self.execute(&financial_ratios_ttm(query.into())).await
    }
}
