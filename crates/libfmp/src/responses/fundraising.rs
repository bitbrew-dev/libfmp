//! Response rows returned by crowdfunding and Regulation D fundraising endpoints.
//!
//! Future Python bindings reserve these models under `fmp.fundraising`.

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Number;

use crate::{
    codecs::{DynamicJson, UsDate, YnFlag},
    types::{ApiDateTime, Cik, Count, FormType, StatementAmount},
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

/// One detailed crowdfunding offering.
///
/// All 48 keys documented by the provider are required. The security's free-form
/// alternative description is the sole nullable value, but its key must remain
/// present. [`Number`] preserves whether the provider spells `offeringPrice` as
/// an integer or decimal JSON number.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrowdfundingOffering {
    pub cik: Cik,
    pub company_name: String,
    pub date: UsDate,
    pub filing_date: ApiDateTime,
    pub accepted_date: ApiDateTime,
    pub form_type: FormType,
    pub form_signification: String,
    pub name_of_issuer: String,
    pub legal_status_form: String,
    pub jurisdiction_organization: String,
    pub issuer_street: String,
    pub issuer_city: String,
    pub issuer_state_or_country: String,
    pub issuer_zip_code: String,
    pub issuer_website: String,
    pub intermediary_company_name: String,
    pub intermediary_commission_cik: Cik,
    pub intermediary_commission_file_number: String,
    pub compensation_amount: String,
    pub financial_interest: String,
    pub security_offered_type: String,
    #[serde(deserialize_with = "required_option")]
    pub security_offered_other_description: Option<String>,
    pub number_of_security_offered: Count,
    pub offering_price: Number,
    pub offering_amount: StatementAmount,
    pub over_subscription_accepted: YnFlag,
    pub over_subscription_allocation_type: String,
    pub maximum_offering_amount: StatementAmount,
    pub offering_deadline_date: UsDate,
    pub current_number_of_employees: Count,
    pub total_asset_most_recent_fiscal_year: StatementAmount,
    pub total_asset_prior_fiscal_year: StatementAmount,
    #[serde(rename = "cashAndCashEquiValentMostRecentFiscalYear")]
    pub cash_and_cash_equivalent_most_recent_fiscal_year: StatementAmount,
    #[serde(rename = "cashAndCashEquiValentPriorFiscalYear")]
    pub cash_and_cash_equivalent_prior_fiscal_year: StatementAmount,
    pub accounts_receivable_most_recent_fiscal_year: StatementAmount,
    pub accounts_receivable_prior_fiscal_year: StatementAmount,
    pub short_term_debt_most_recent_fiscal_year: StatementAmount,
    pub short_term_debt_prior_fiscal_year: StatementAmount,
    pub long_term_debt_most_recent_fiscal_year: StatementAmount,
    pub long_term_debt_prior_fiscal_year: StatementAmount,
    pub revenue_most_recent_fiscal_year: StatementAmount,
    pub revenue_prior_fiscal_year: StatementAmount,
    pub cost_goods_sold_most_recent_fiscal_year: StatementAmount,
    pub cost_goods_sold_prior_fiscal_year: StatementAmount,
    pub taxes_paid_most_recent_fiscal_year: StatementAmount,
    pub taxes_paid_prior_fiscal_year: StatementAmount,
    pub net_income_most_recent_fiscal_year: StatementAmount,
    pub net_income_prior_fiscal_year: StatementAmount,
}
