use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Map, Value, json};

use libfmp::{
    query::FiscalPeriod,
    responses::bulk::{BulkIncomeStatement, BulkIncomeStatementGrowth},
};

const INCOME: &[u8] = include_bytes!("fixtures/bulk_income_statements.json");
const GROWTH: &[u8] = include_bytes!("fixtures/bulk_income_statement_growth.json");

#[test]
fn exact_outer_md_fixtures_round_trip_with_exact_unique_key_sets() {
    assert_exact::<BulkIncomeStatement>(INCOME, 39);
    assert_exact::<BulkIncomeStatementGrowth>(GROWTH, 34);
}

#[test]
fn every_documented_field_is_required_non_null_and_unknown_fields_are_tolerated() {
    assert_required::<BulkIncomeStatement>(INCOME);
    assert_required::<BulkIncomeStatementGrowth>(GROWTH);
}

#[test]
fn every_documented_metric_preserves_numeric_text_and_rejects_json_numbers() {
    assert_all_numbers_rejected::<BulkIncomeStatement>(
        INCOME,
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
    assert_all_numbers_rejected::<BulkIncomeStatementGrowth>(
        GROWTH,
        &["symbol", "date", "fiscalYear", "period", "reportedCurrency"],
    );

    let income = rows::<BulkIncomeStatement>(INCOME).remove(0);
    assert_eq!(income.revenue.as_str(), "33644000000");
    assert_eq!(income.cost_of_revenue.as_str(), "0");
    assert_eq!(
        income.total_other_income_expenses_net.as_str(),
        "-7392000000"
    );
    assert_eq!(income.eps.as_str(), "0.62");
    assert_eq!(income.weighted_average_shs_out.as_str(), "22735483871");

    let growth = rows::<BulkIncomeStatementGrowth>(GROWTH).remove(0);
    assert_eq!(growth.growth_ebit.as_str(), "1");
    assert_eq!(growth.growth_cost_of_revenue.as_str(), "0");
    assert_eq!(growth.growth_other_expenses.as_str(), "-0.9860376183912135");
    assert_eq!(
        growth.growth_operating_income.as_str(),
        "-0.018874787810201278"
    );
}

#[test]
fn identity_fields_are_narrow_and_preserve_documented_representations() {
    let income = rows::<BulkIncomeStatement>(INCOME).remove(0);
    assert_eq!(income.symbol.as_str(), "000001.SZ");
    assert_eq!(income.reported_currency.as_str(), "CNY");
    assert_eq!(income.cik.as_str(), "0000000000");
    assert_eq!(income.date.to_string(), "2025-03-31");
    assert_eq!(income.filing_date.to_string(), "2025-03-31");
    assert_eq!(income.accepted_date.to_string(), "2025-03-31 00:00:00");
    assert_eq!(income.fiscal_year.as_str(), "2025");
    assert_eq!(income.period, FiscalPeriod::Q1);

    let mut invalid_date = source_row(INCOME);
    invalid_date.insert("date".into(), json!("2025-03-31 00:00:00"));
    assert!(serde_json::from_value::<BulkIncomeStatement>(Value::Object(invalid_date)).is_err());
    let mut invalid_datetime = source_row(INCOME);
    invalid_datetime.insert("acceptedDate".into(), json!("2025-03-31"));
    assert!(
        serde_json::from_value::<BulkIncomeStatement>(Value::Object(invalid_datetime)).is_err()
    );
    let mut numeric_cik = source_row(INCOME);
    numeric_cik.insert("cik".into(), json!(0));
    assert!(serde_json::from_value::<BulkIncomeStatement>(Value::Object(numeric_cik)).is_err());
}

#[test]
fn acronym_hazards_keep_exact_documented_wire_names() {
    let growth = serialized_row::<BulkIncomeStatementGrowth>(GROWTH);
    for key in [
        "growthEBITDA",
        "growthEPS",
        "growthEPSDiluted",
        "growthEBIT",
    ] {
        assert!(growth.contains_key(key), "missing exact wire key {key}");
    }
    for incorrect in [
        "growthEbitda",
        "growthEps",
        "growthEpsDiluted",
        "growthEbit",
    ] {
        assert!(!growth.contains_key(incorrect));
    }
}

#[test]
fn both_contracts_require_bare_array_roots_and_accept_empty_arrays() {
    assert_array_contract::<BulkIncomeStatement>();
    assert_array_contract::<BulkIncomeStatementGrowth>();
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
