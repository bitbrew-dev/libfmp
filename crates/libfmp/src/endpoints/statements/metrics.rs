//! Historical and trailing-twelve-month key-metrics endpoints.

use crate::{
    Client, Result,
    endpoints::EndpointSpec,
    responses::statements::{KeyMetrics, KeyMetricsTtm},
};

use super::{
    KeyMetricsQuery, KeyMetricsTtmQuery, WORLDWIDE_FINANCIAL_HISTORY_METADATA,
    WORLDWIDE_FINANCIAL_SUMMARY_METADATA,
};

/// Describes `GET key-metrics` without binding it to a transport.
pub fn key_metrics(query: KeyMetricsQuery) -> EndpointSpec<KeyMetricsQuery, Vec<KeyMetrics>> {
    EndpointSpec::get("key-metrics", "key-metrics", query)
        .with_metadata(WORLDWIDE_FINANCIAL_HISTORY_METADATA)
}

/// Describes `GET key-metrics-ttm` without binding it to a transport.
pub fn key_metrics_ttm(
    query: KeyMetricsTtmQuery,
) -> EndpointSpec<KeyMetricsTtmQuery, Vec<KeyMetricsTtm>> {
    EndpointSpec::get("key-metrics-ttm", "key-metrics-ttm", query)
        .with_metadata(WORLDWIDE_FINANCIAL_SUMMARY_METADATA)
}

impl Client {
    /// Retrieves historical worldwide key metrics for one company.
    pub async fn key_metrics(&self, query: impl Into<KeyMetricsQuery>) -> Result<Vec<KeyMetrics>> {
        self.execute(&key_metrics(query.into())).await
    }

    /// Retrieves worldwide trailing-twelve-month key metrics for one company.
    pub async fn key_metrics_ttm(
        &self,
        query: impl Into<KeyMetricsTtmQuery>,
    ) -> Result<Vec<KeyMetricsTtm>> {
        self.execute(&key_metrics_ttm(query.into())).await
    }
}
