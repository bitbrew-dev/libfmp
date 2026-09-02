//! Response rows returned by crowdfunding and Regulation D fundraising endpoints.
//!
//! Future Python bindings reserve these models under `fmp.fundraising`.

use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    codecs::DynamicJson,
    types::{ApiDateTime, Cik},
};

fn required_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

/// One compact crowdfunding-offering search result.
///
/// The only documented `date` value is JSON null. Its non-null wire contract
/// therefore remains deliberately raw until the provider documentation proves
/// a stable representation. The key itself is required.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrowdfundingOfferingSearchResult {
    pub cik: Cik,
    pub name: String,
    #[serde(deserialize_with = "required_option")]
    pub date: Option<DynamicJson>,
}

/// One compact Regulation D offering search result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegulationDOfferingSearchResult {
    pub cik: Cik,
    pub name: String,
    pub date: ApiDateTime,
}
