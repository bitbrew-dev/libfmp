use std::str::FromStr;

use libfmp::{
    codecs::FiscalYearString,
    query::FiscalPeriod,
    responses::statements::{
        EnterpriseValue, FinancialScore, LatestFinancialStatement, OwnerEarnings,
    },
    types::{ApiDateTime, CalendarYear, CurrencyCode, Date, Ticker},
};
use serde::de::DeserializeOwned;

const LATEST: &[u8] = include_bytes!("fixtures/latest_financial_statements.json");
const SCORES: &[u8] = include_bytes!("fixtures/financial_scores.json");
const OWNER: &[u8] = include_bytes!("fixtures/owner_earnings.json");
const ENTERPRISE: &[u8] = include_bytes!("fixtures/enterprise_values.json");

#[test]
fn documented_latest_statement_decodes_all_five_exact_fields() {
    let value: serde_json::Value = serde_json::from_slice(LATEST).unwrap();
    assert_eq!(value[0].as_object().unwrap().len(), 5);

    let rows: Vec<LatestFinancialStatement> = serde_json::from_value(value).unwrap();
    assert_eq!(
        rows,
        [LatestFinancialStatement {
            symbol: Ticker::new("UFPI").unwrap(),
            calendar_year: CalendarYear(2026),
            period: FiscalPeriod::Q2,
            date: Date::from_str("2026-06-27").unwrap(),
            date_added: ApiDateTime::from_str("2026-07-30 13:17:26").unwrap(),
        }]
    );
    assert_eq!(rows[0].calendar_year.to_string(), "2026");
    assert_eq!(rows[0].date_added.to_string(), "2026-07-30 13:17:26");
}

#[test]
fn documented_financial_score_decodes_all_eleven_exact_fields() {
    let value: serde_json::Value = serde_json::from_slice(SCORES).unwrap();
    assert_eq!(value[0].as_object().unwrap().len(), 11);

    let rows: Vec<FinancialScore> = serde_json::from_value(value).unwrap();
    assert_eq!(
        rows,
        [FinancialScore {
            symbol: Ticker::new("AAPL").unwrap(),
            reported_currency: CurrencyCode::new("USD").unwrap(),
            altman_z_score: 14.041374927993303,
            piotroski_score: 9,
            working_capital: 9_473_000_000,
            total_assets: 371_082_000_000,
            retained_earnings: 12_359_000_000,
            ebit: 147_722_000_000,
            market_cap: 5_042_169_135_511,
            total_liabilities: 264_591_000_000,
            revenue: 451_442_000_000,
        }]
    );
}

#[test]
fn documented_owner_earnings_decodes_exact_average_ppe_and_signed_fields() {
    let value: serde_json::Value = serde_json::from_slice(OWNER).unwrap();
    assert_eq!(value[0].as_object().unwrap().len(), 10);

    let rows: Vec<OwnerEarnings> = serde_json::from_value(value).unwrap();
    assert_eq!(
        rows,
        [OwnerEarnings {
            symbol: Ticker::new("AAPL").unwrap(),
            reported_currency: CurrencyCode::new("USD").unwrap(),
            fiscal_year: FiscalYearString::new("2026").unwrap(),
            period: FiscalPeriod::Q2,
            date: Date::from_str("2026-03-28").unwrap(),
            average_ppe: 0.13466,
            maintenance_capex: 159_994_500,
            owners_earnings: 28_861_994_500,
            growth_capex: -2_130_994_500,
            owners_earnings_per_share: 1.95,
        }]
    );

    let serialized = serde_json::to_value(&rows[0]).unwrap();
    assert_eq!(serialized["averagePPE"], serde_json::json!(0.13466));
    assert!(serialized.get("averagePpe").is_none());
}

#[test]
fn documented_enterprise_value_decodes_all_eight_exact_fields() {
    let value: serde_json::Value = serde_json::from_slice(ENTERPRISE).unwrap();
    assert_eq!(value[0].as_object().unwrap().len(), 8);

    let rows: Vec<EnterpriseValue> = serde_json::from_value(value).unwrap();
    assert_eq!(
        rows,
        [EnterpriseValue {
            symbol: Ticker::new("AAPL").unwrap(),
            date: Date::from_str("2025-09-27").unwrap(),
            stock_price: 255.46,
            number_of_shares: 14_948_500_000,
            market_capitalization: 3_818_743_810_000,
            minus_cash_and_cash_equivalents: 35_934_000_000,
            add_total_debt: 112_377_000_000,
            enterprise_value: 3_895_186_810_000,
        }]
    );
}

#[test]
fn every_documented_field_is_required_by_its_exact_response_model() {
    assert_every_field_required::<LatestFinancialStatement>(LATEST);
    assert_every_field_required::<FinancialScore>(SCORES);
    assert_every_field_required::<OwnerEarnings>(OWNER);
    assert_every_field_required::<EnterpriseValue>(ENTERPRISE);
}

fn assert_every_field_required<T: DeserializeOwned>(fixture: &[u8]) {
    let source: serde_json::Value = serde_json::from_slice(fixture).unwrap();
    let keys = source[0]
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect::<Vec<_>>();

    for key in keys {
        let mut missing = source.clone();
        missing[0].as_object_mut().unwrap().remove(&key);
        assert!(
            serde_json::from_value::<Vec<T>>(missing).is_err(),
            "accepted missing required field {key}"
        );
    }
}

#[test]
fn every_compact_contract_is_a_bare_array_that_preserves_empty_and_multiple_rows() {
    assert!(
        serde_json::from_slice::<Vec<LatestFinancialStatement>>(b"[]")
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_slice::<Vec<FinancialScore>>(b"[]")
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_slice::<Vec<OwnerEarnings>>(b"[]")
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_slice::<Vec<EnterpriseValue>>(b"[]")
            .unwrap()
            .is_empty()
    );

    assert_multiple::<LatestFinancialStatement>(LATEST);
    assert_multiple::<FinancialScore>(SCORES);
    assert_multiple::<OwnerEarnings>(OWNER);
    assert_multiple::<EnterpriseValue>(ENTERPRISE);

    let wrapped = serde_json::json!({ "financialScores": [] });
    assert!(serde_json::from_value::<Vec<FinancialScore>>(wrapped).is_err());
}

fn assert_multiple<T: DeserializeOwned>(fixture: &[u8]) {
    let mut value: serde_json::Value = serde_json::from_slice(fixture).unwrap();
    let duplicate = value[0].clone();
    value.as_array_mut().unwrap().push(duplicate);
    assert_eq!(serde_json::from_value::<Vec<T>>(value).unwrap().len(), 2);
}

#[test]
fn calendar_year_and_date_added_keep_their_exact_numeric_and_naive_wire_kinds() {
    let mut string_year: serde_json::Value = serde_json::from_slice(LATEST).unwrap();
    string_year[0]["calendarYear"] = serde_json::json!("2026");
    assert!(serde_json::from_value::<Vec<LatestFinancialStatement>>(string_year).is_err());

    let mut zoned_date: serde_json::Value = serde_json::from_slice(LATEST).unwrap();
    zoned_date[0]["dateAdded"] = serde_json::json!("2026-07-30T13:17:26Z");
    assert!(serde_json::from_value::<Vec<LatestFinancialStatement>>(zoned_date).is_err());
}

#[test]
fn owner_fiscal_year_and_average_ppe_keep_their_exact_wire_contracts() {
    let mut numeric_year: serde_json::Value = serde_json::from_slice(OWNER).unwrap();
    numeric_year[0]["fiscalYear"] = serde_json::json!(2026);
    assert!(serde_json::from_value::<Vec<OwnerEarnings>>(numeric_year).is_err());

    let mut wrong_acronym: serde_json::Value = serde_json::from_slice(OWNER).unwrap();
    let average = wrong_acronym[0]
        .as_object_mut()
        .unwrap()
        .remove("averagePPE")
        .unwrap();
    wrong_acronym[0]["averagePpe"] = average;
    assert!(serde_json::from_value::<Vec<OwnerEarnings>>(wrong_acronym).is_err());
}

#[test]
fn signed_statement_amounts_preserve_large_negative_values() {
    let mut owner: serde_json::Value = serde_json::from_slice(OWNER).unwrap();
    owner[0]["growthCapex"] = serde_json::json!(-9_000_000_000_000_000_000_i64);

    let rows: Vec<OwnerEarnings> = serde_json::from_value(owner).unwrap();
    assert_eq!(rows[0].growth_capex, -9_000_000_000_000_000_000_i64);

    let mut score: serde_json::Value = serde_json::from_slice(SCORES).unwrap();
    score[0]["piotroskiScore"] = serde_json::json!(-1);
    assert!(serde_json::from_value::<Vec<FinancialScore>>(score).is_err());
}
