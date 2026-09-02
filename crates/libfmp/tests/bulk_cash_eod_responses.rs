use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Map, Value, json};

use libfmp::{
    query::FiscalPeriod,
    responses::bulk::{BulkCashFlowStatement, BulkCashFlowStatementGrowth, BulkEodBar},
};

const CASH_FLOW: &[u8] = include_bytes!("fixtures/bulk_cash_flow_statements.json");
const GROWTH: &[u8] = include_bytes!("fixtures/bulk_cash_flow_statement_growth.json");
const EOD: &[u8] = include_bytes!("fixtures/bulk_eod.json");

#[test]
fn exact_outer_md_fixtures_round_trip_with_exact_unique_key_sets() {
    assert_exact::<BulkCashFlowStatement>(CASH_FLOW, 47);
    assert_exact::<BulkCashFlowStatementGrowth>(GROWTH, 42);
    assert_exact::<BulkEodBar>(EOD, 8);
}

#[test]
fn every_documented_field_is_required_non_null_and_unknown_fields_are_tolerated() {
    assert_required::<BulkCashFlowStatement>(CASH_FLOW);
    assert_required::<BulkCashFlowStatementGrowth>(GROWTH);
    assert_required::<BulkEodBar>(EOD);
}

#[test]
fn every_documented_metric_preserves_numeric_text_and_rejects_json_numbers() {
    assert_all_numbers_rejected::<BulkCashFlowStatement>(
        CASH_FLOW,
        &[
            "date",
            "symbol",
            "reportedCurrency",
            "cik",
            "filingDate",
            "acceptedDate",
            "fiscalYear",
            "period",
        ],
    );
    assert_all_numbers_rejected::<BulkCashFlowStatementGrowth>(
        GROWTH,
        &["symbol", "date", "fiscalYear", "period", "reportedCurrency"],
    );
    assert_all_numbers_rejected::<BulkEodBar>(EOD, &["symbol", "date"]);

    let cash_flow = rows::<BulkCashFlowStatement>(CASH_FLOW).remove(0);
    assert_eq!(cash_flow.other_non_cash_items.as_str(), "162946000000");
    assert_eq!(cash_flow.purchases_of_investments.as_str(), "-227916000000");
    assert_eq!(cash_flow.net_income.as_str(), "0");

    let growth = rows::<BulkCashFlowStatementGrowth>(GROWTH).remove(0);
    assert_eq!(
        growth
            .growth_net_cash_used_provided_by_financing_activities
            .as_str(),
        "-3.2122934677858628"
    );
    assert_eq!(growth.growth_net_debt_issuance.as_str(), "1");

    let eod = rows::<BulkEodBar>(EOD).remove(0);
    assert_eq!(eod.open.as_str(), "2.67");
    assert_eq!(eod.low.as_str(), "2.7");
    assert_eq!(eod.volume.as_str(), "920904");
}

#[test]
fn identity_fields_are_narrow_and_preserve_documented_representations() {
    let cash_flow = rows::<BulkCashFlowStatement>(CASH_FLOW).remove(0);
    assert_eq!(cash_flow.symbol.as_str(), "000001.SZ");
    assert_eq!(cash_flow.reported_currency.as_str(), "CNY");
    assert_eq!(cash_flow.cik.as_str(), "0000000000");
    assert_eq!(cash_flow.date.to_string(), "2025-03-31");
    assert_eq!(cash_flow.filing_date.to_string(), "2025-03-31");
    assert_eq!(cash_flow.accepted_date.to_string(), "2025-03-31 00:00:00");
    assert_eq!(cash_flow.fiscal_year.as_str(), "2025");
    assert_eq!(cash_flow.period, FiscalPeriod::Q1);

    let eod = rows::<BulkEodBar>(EOD).remove(0);
    assert_eq!(eod.symbol.as_str(), "EGS745W1C011.CA");
    assert_eq!(eod.date.to_string(), "2024-10-22");

    let mut numeric_cik = source_row(CASH_FLOW);
    numeric_cik.insert("cik".into(), json!(0));
    assert!(serde_json::from_value::<BulkCashFlowStatement>(Value::Object(numeric_cik)).is_err());
    let mut invalid_datetime = source_row(CASH_FLOW);
    invalid_datetime.insert("acceptedDate".into(), json!("2025-03-31"));
    assert!(
        serde_json::from_value::<BulkCashFlowStatement>(Value::Object(invalid_datetime)).is_err()
    );
    let mut invalid_eod_date = source_row(EOD);
    invalid_eod_date.insert("date".into(), json!("2024-10-22 00:00:00"));
    assert!(serde_json::from_value::<BulkEodBar>(Value::Object(invalid_eod_date)).is_err());
}

#[test]
fn provider_activity_typos_keep_exact_wire_names_with_corrected_rust_fields() {
    let growth = serialized_row::<BulkCashFlowStatementGrowth>(GROWTH);
    for key in [
        "growthNetCashProvidedByOperatingActivites",
        "growthOtherInvestingActivites",
        "growthNetCashUsedForInvestingActivites",
        "growthOtherFinancingActivites",
        "growthNetCashUsedProvidedByFinancingActivities",
    ] {
        assert!(growth.contains_key(key), "missing exact wire key {key}");
    }
    for incorrect in [
        "growthNetCashProvidedByOperatingActivities",
        "growthOtherInvestingActivities",
        "growthNetCashUsedForInvestingActivities",
        "growthOtherFinancingActivities",
        "growthNetCashUsedProvidedByFinancingActivites",
    ] {
        assert!(!growth.contains_key(incorrect));
    }
}

#[test]
fn all_three_contracts_require_bare_array_roots_and_accept_empty_arrays() {
    assert_array_contract::<BulkCashFlowStatement>();
    assert_array_contract::<BulkCashFlowStatementGrowth>();
    assert_array_contract::<BulkEodBar>();
}

fn rows<T: DeserializeOwned>(fixture: &[u8]) -> Vec<T> {
    serde_json::from_slice(fixture).unwrap()
}

fn source_row(fixture: &[u8]) -> Map<String, Value> {
    serde_json::from_slice::<Value>(fixture).unwrap()[0]
        .as_object()
        .unwrap()
        .clone()
}

fn serialized_row<T>(fixture: &[u8]) -> Map<String, Value>
where
    T: DeserializeOwned + Serialize,
{
    serde_json::to_value(rows::<T>(fixture)).unwrap()[0]
        .as_object()
        .unwrap()
        .clone()
}

fn assert_exact<T>(fixture: &[u8], fields: usize)
where
    T: DeserializeOwned + Serialize,
{
    let source: Value = serde_json::from_slice(fixture).unwrap();
    let row = source[0].as_object().unwrap();
    assert_eq!(row.len(), fields);
    assert_eq!(
        row.keys().collect::<std::collections::BTreeSet<_>>().len(),
        fields
    );
    assert_eq!(serde_json::to_value(rows::<T>(fixture)).unwrap(), source);
}

fn assert_required<T>(fixture: &[u8])
where
    T: DeserializeOwned,
{
    let row = source_row(fixture);
    for field in row.keys() {
        let mut missing = row.clone();
        missing.remove(field);
        assert!(
            serde_json::from_value::<T>(Value::Object(missing)).is_err(),
            "missing {field} unexpectedly decoded"
        );

        let mut null = row.clone();
        null.insert(field.clone(), Value::Null);
        assert!(
            serde_json::from_value::<T>(Value::Object(null)).is_err(),
            "null {field} unexpectedly decoded"
        );
    }

    let mut future = row;
    future.insert("futureField".into(), json!({"nested": true}));
    assert!(serde_json::from_value::<T>(Value::Object(future)).is_ok());
}

fn assert_all_numbers_rejected<T>(fixture: &[u8], non_numeric: &[&str])
where
    T: DeserializeOwned,
{
    let row = source_row(fixture);
    for field in row
        .keys()
        .filter(|field| !non_numeric.contains(&field.as_str()))
    {
        let mut numeric = row.clone();
        numeric.insert(field.clone(), json!(123.5));
        assert!(
            serde_json::from_value::<T>(Value::Object(numeric)).is_err(),
            "JSON number unexpectedly decoded for {field}"
        );
    }
}

fn assert_array_contract<T>()
where
    T: DeserializeOwned,
{
    assert!(serde_json::from_slice::<Vec<T>>(b"[]").unwrap().is_empty());
    assert!(serde_json::from_slice::<Vec<T>>(b"{}").is_err());
}
