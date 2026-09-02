use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Map, Value, json};

use libfmp::{
    query::FiscalPeriod,
    responses::bulk::{BulkBalanceSheetStatement, BulkBalanceSheetStatementGrowth},
};

const BALANCE: &[u8] = include_bytes!("fixtures/bulk_balance_sheet_statements.json");
const GROWTH: &[u8] = include_bytes!("fixtures/bulk_balance_sheet_statement_growth.json");

#[test]
fn exact_outer_md_fixtures_round_trip_with_exact_unique_key_sets() {
    assert_exact::<BulkBalanceSheetStatement>(BALANCE, 61);
    assert_exact::<BulkBalanceSheetStatementGrowth>(GROWTH, 56);
}

#[test]
fn every_documented_field_is_required_non_null_and_unknown_fields_are_tolerated() {
    assert_required::<BulkBalanceSheetStatement>(BALANCE);
    assert_required::<BulkBalanceSheetStatementGrowth>(GROWTH);
}

#[test]
fn every_documented_metric_preserves_numeric_text_and_rejects_json_numbers() {
    assert_all_numbers_rejected::<BulkBalanceSheetStatement>(
        BALANCE,
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
    assert_all_numbers_rejected::<BulkBalanceSheetStatementGrowth>(
        GROWTH,
        &["symbol", "date", "fiscalYear", "period", "reportedCurrency"],
    );

    let balance = rows::<BulkBalanceSheetStatement>(BALANCE).remove(0);
    assert_eq!(balance.total_assets.as_str(), "247871857000");
    assert_eq!(balance.retained_earnings.as_str(), "-5066509000");
    assert_eq!(balance.other_assets.as_str(), "0");
    assert_eq!(balance.net_debt.as_str(), "183764862000");

    let growth = rows::<BulkBalanceSheetStatementGrowth>(GROWTH).remove(0);
    assert_eq!(
        growth.growth_cash_and_cash_equivalents.as_str(),
        "0.09574482145872953"
    );
    assert_eq!(growth.growth_short_term_investments.as_str(), "0");
    assert_eq!(
        growth.growth_total_payables.as_str(),
        "-0.12022416350749959"
    );
}

#[test]
fn identity_fields_are_narrow_and_preserve_documented_representations() {
    let balance = rows::<BulkBalanceSheetStatement>(BALANCE).remove(0);
    assert_eq!(balance.symbol.as_str(), "MTLRP.ME");
    assert_eq!(balance.reported_currency.as_str(), "RUB");
    assert_eq!(balance.cik.as_str(), "0000000000");
    assert_eq!(balance.date.to_string(), "2025-03-31");
    assert_eq!(balance.filing_date.to_string(), "2025-05-31");
    assert_eq!(balance.accepted_date.to_string(), "2025-03-31 07:00:00");
    assert_eq!(balance.fiscal_year.as_str(), "2025");
    assert_eq!(balance.period, FiscalPeriod::Q1);

    let mut invalid_date = source_row(BALANCE);
    invalid_date.insert("date".into(), json!("2025-03-31 00:00:00"));
    assert!(
        serde_json::from_value::<BulkBalanceSheetStatement>(Value::Object(invalid_date)).is_err()
    );
    let mut invalid_datetime = source_row(BALANCE);
    invalid_datetime.insert("acceptedDate".into(), json!("2025-03-31"));
    assert!(
        serde_json::from_value::<BulkBalanceSheetStatement>(Value::Object(invalid_datetime))
            .is_err()
    );
    let mut numeric_cik = source_row(BALANCE);
    numeric_cik.insert("cik".into(), json!(0));
    assert!(
        serde_json::from_value::<BulkBalanceSheetStatement>(Value::Object(numeric_cik)).is_err()
    );
}

#[test]
fn provider_typo_and_liabilities_equity_wording_keep_exact_wire_names() {
    let growth = serialized_row::<BulkBalanceSheetStatementGrowth>(GROWTH);
    for key in [
        "growthOthertotalStockholdersEquity",
        "growthTotalLiabilitiesAndStockholdersEquity",
    ] {
        assert!(growth.contains_key(key), "missing exact wire key {key}");
    }
    for incorrect in [
        "growthOtherTotalStockholdersEquity",
        "growthTotalLiabilitiesAndTotalEquity",
    ] {
        assert!(!growth.contains_key(incorrect));
    }
}

#[test]
fn both_contracts_require_bare_array_roots_and_accept_empty_arrays() {
    assert_array_contract::<BulkBalanceSheetStatement>();
    assert_array_contract::<BulkBalanceSheetStatementGrowth>();
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
