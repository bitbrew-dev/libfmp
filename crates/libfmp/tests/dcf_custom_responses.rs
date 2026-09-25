use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};

use libfmp::responses::dcf::{CustomDcfValuation, CustomLeveredDcfValuation};

const CUSTOM: &[u8] = include_bytes!("fixtures/custom_discounted_cash_flow.json");
const LEVERED: &[u8] = include_bytes!("fixtures/custom_levered_discounted_cash_flow.json");

const CUSTOM_FIELDS: &[&str] = &[
    "year",
    "symbol",
    "revenue",
    "revenuePercentage",
    "ebitda",
    "ebitdaPercentage",
    "ebit",
    "ebitPercentage",
    "depreciation",
    "depreciationPercentage",
    "totalCash",
    "totalCashPercentage",
    "receivables",
    "receivablesPercentage",
    "inventories",
    "inventoriesPercentage",
    "payable",
    "payablePercentage",
    "capitalExpenditure",
    "capitalExpenditurePercentage",
    "price",
    "beta",
    "dilutedSharesOutstanding",
    "costofDebt",
    "taxRate",
    "afterTaxCostOfDebt",
    "riskFreeRate",
    "marketRiskPremium",
    "costOfEquity",
    "totalDebt",
    "totalEquity",
    "totalCapital",
    "debtWeighting",
    "equityWeighting",
    "wacc",
    "taxRateCash",
    "ebiat",
    "ufcf",
    "sumPvUfcf",
    "longTermGrowthRate",
    "terminalValue",
    "presentTerminalValue",
    "enterpriseValue",
    "netDebt",
    "equityValue",
    "equityValuePerShare",
    "freeCashFlowT1",
];

const LEVERED_FIELDS: &[&str] = &[
    "year",
    "symbol",
    "revenue",
    "revenuePercentage",
    "capitalExpenditure",
    "capitalExpenditurePercentage",
    "price",
    "beta",
    "dilutedSharesOutstanding",
    "costofDebt",
    "taxRate",
    "afterTaxCostOfDebt",
    "riskFreeRate",
    "marketRiskPremium",
    "costOfEquity",
    "totalDebt",
    "totalEquity",
    "totalCapital",
    "debtWeighting",
    "equityWeighting",
    "wacc",
    "operatingCashFlow",
    "pvLfcf",
    "sumPvLfcf",
    "longTermGrowthRate",
    "freeCashFlow",
    "terminalValue",
    "presentTerminalValue",
    "enterpriseValue",
    "netDebt",
    "equityValue",
    "equityValuePerShare",
    "freeCashFlowT1",
    "operatingCashFlowPercentage",
];

#[test]
fn both_outer_fixtures_round_trip_exactly_into_distinct_models() {
    assert_exact::<CustomDcfValuation>(CUSTOM);
    assert_exact::<CustomLeveredDcfValuation>(LEVERED);

    let custom: Vec<CustomDcfValuation> = serde_json::from_slice(CUSTOM).unwrap();
    let levered: Vec<CustomLeveredDcfValuation> = serde_json::from_slice(LEVERED).unwrap();
    assert_eq!(custom[0].year.as_str(), "2030");
    assert_eq!(custom[0].symbol.as_str(), "AAPL");
    assert_eq!(custom[0].capital_expenditure, -14_907_445_037.0);
    assert_eq!(custom[0].diluted_shares_outstanding, 15_004_697_000.0);
    assert_eq!(custom[0].equity_value_per_share, 147.18);
    assert_eq!(levered[0].operating_cash_flow, 153_867_620_418.0);
    assert_eq!(levered[0].pv_lfcf, 88_605_139_549.0);
    assert_eq!(levered[0].equity_value_per_share, 140.71);
}

#[test]
fn all_47_custom_fields_are_required_non_null_and_future_fields_are_accepted() {
    assert_required::<CustomDcfValuation>(CUSTOM, CUSTOM_FIELDS);
    assert_eq!(CUSTOM_FIELDS.len(), 47);
}

#[test]
fn all_34_levered_fields_are_required_non_null_and_future_fields_are_accepted() {
    assert_required::<CustomLeveredDcfValuation>(LEVERED, LEVERED_FIELDS);
    assert_eq!(LEVERED_FIELDS.len(), 34);
}

#[test]
fn both_contracts_require_bare_arrays_and_preserve_exact_costof_debt_wire_key() {
    assert_bare_array::<CustomDcfValuation>();
    assert_bare_array::<CustomLeveredDcfValuation>();

    let custom: Vec<CustomDcfValuation> = serde_json::from_slice(CUSTOM).unwrap();
    let levered: Vec<CustomLeveredDcfValuation> = serde_json::from_slice(LEVERED).unwrap();
    for encoded in [
        serde_json::to_value(&custom[0]).unwrap(),
        serde_json::to_value(&levered[0]).unwrap(),
    ] {
        assert_eq!(encoded["costofDebt"], json!(4.37));
        assert!(encoded.get("costOfDebt").is_none());
        assert!(encoded.get("cost_of_debt").is_none());
    }
}

fn assert_exact<T>(fixture: &[u8])
where
    T: DeserializeOwned + Serialize,
{
    let source: Value = serde_json::from_slice(fixture).unwrap();
    let rows: Vec<T> = serde_json::from_slice(fixture).unwrap();
    assert_json_values(&serde_json::to_value(rows).unwrap(), &source);
}

fn assert_json_values(actual: &Value, expected: &Value) {
    match (actual, expected) {
        (Value::Array(actual), Value::Array(expected)) => {
            assert_eq!(actual.len(), expected.len());
            for (actual, expected) in actual.iter().zip(expected) {
                assert_json_values(actual, expected);
            }
        }
        (Value::Object(actual), Value::Object(expected)) => {
            assert_eq!(actual.len(), expected.len());
            for (key, expected) in expected {
                assert_json_values(&actual[key], expected);
            }
        }
        (Value::Number(actual), Value::Number(expected)) => {
            assert_eq!(actual.as_f64(), expected.as_f64());
        }
        _ => assert_eq!(actual, expected),
    }
}

fn assert_required<T>(fixture: &[u8], fields: &[&str])
where
    T: DeserializeOwned,
{
    let source: Value = serde_json::from_slice(fixture).unwrap();
    let row = source[0].clone();
    assert_eq!(row.as_object().unwrap().len(), fields.len());

    for field in fields {
        let mut missing = row.clone();
        missing.as_object_mut().unwrap().remove(*field);
        assert!(
            serde_json::from_value::<T>(missing).is_err(),
            "{field} unexpectedly accepted when missing"
        );

        let mut null = row.clone();
        null[*field] = Value::Null;
        assert!(
            serde_json::from_value::<T>(null).is_err(),
            "{field} unexpectedly accepted when null"
        );
    }

    let mut future = row;
    future["futureField"] = json!({"nested": true});
    assert!(serde_json::from_value::<T>(future).is_ok());
}

fn assert_bare_array<T>()
where
    T: DeserializeOwned,
{
    assert!(serde_json::from_str::<Vec<T>>("[]").unwrap().is_empty());
    assert!(serde_json::from_str::<Vec<T>>("{}").is_err());
}
