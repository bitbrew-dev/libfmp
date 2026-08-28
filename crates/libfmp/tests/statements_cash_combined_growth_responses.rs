use libfmp::responses::statements::{CashFlowStatementGrowth, FinancialStatementGrowth};
use serde::{Serialize, de::DeserializeOwned};

const CASH: &[u8] = include_bytes!("fixtures/cash_flow_statement_growth.json");
const COMBINED: &[u8] = include_bytes!("fixtures/financial_statement_growth.json");

#[test]
fn documented_cash_flow_growth_decodes_all_42_exact_fields() {
    let value: serde_json::Value = serde_json::from_slice(CASH).unwrap();
    assert_eq!(value[0].as_object().unwrap().len(), 42);

    let rows: Vec<CashFlowStatementGrowth> = serde_json::from_value(value).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].growth_change_in_working_capital, -7.847439057792386);
    assert_eq!(rows[0].growth_inventory, 2.338432122370937);
    assert_eq!(
        rows[0].growth_net_cash_provided_by_operating_activities,
        -0.05726656180763441
    );
    assert_eq!(
        rows[0].growth_net_cash_used_for_investing_activities,
        4.177172061328791
    );
    assert_exact_wire_fields::<CashFlowStatementGrowth>(CASH);
}

#[test]
fn documented_financial_growth_decodes_all_44_exact_mixed_case_fields() {
    let value: serde_json::Value = serde_json::from_slice(COMBINED).unwrap();
    assert_eq!(value[0].as_object().unwrap().len(), 44);

    let rows: Vec<FinancialStatementGrowth> = serde_json::from_value(value).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].ebit_growth, 0.0748592946511722);
    assert_eq!(rows[0].eps_growth, 0.22585924713584285);
    assert_eq!(rows[0].book_value_per_share_growth, 0.3289327621427069);
    assert_eq!(
        rows[0].ten_y_operating_cf_growth_per_share,
        1.1119537209402253
    );
    assert_eq!(
        rows[0].ten_y_dividend_per_share_growth_per_share,
        1.0535518165116085
    );
    assert_exact_wire_fields::<FinancialStatementGrowth>(COMBINED);
}

fn assert_exact_wire_fields<T>(fixture: &[u8])
where
    T: DeserializeOwned + Serialize,
{
    let source: serde_json::Value = serde_json::from_slice(fixture).unwrap();
    let rows: Vec<T> = serde_json::from_value(source.clone()).unwrap();
    let serialized = serde_json::to_value(&rows[0]).unwrap();
    let source = source[0].as_object().unwrap();
    let serialized = serialized.as_object().unwrap();

    assert_eq!(
        source.keys().collect::<Vec<_>>(),
        serialized.keys().collect::<Vec<_>>()
    );
    for (key, source_value) in source {
        let serialized_value = &serialized[key];
        if let Some(number) = source_value.as_f64() {
            assert_eq!(serialized_value.as_f64(), Some(number), "changed {key}");
        } else {
            assert_eq!(serialized_value, source_value, "changed {key}");
        }
    }
}

#[test]
fn every_documented_field_is_required_by_its_distinct_model() {
    assert_every_field_required::<CashFlowStatementGrowth>(CASH);
    assert_every_field_required::<FinancialStatementGrowth>(COMBINED);
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
fn five_header_fields_keep_strict_string_date_period_and_currency_contracts() {
    for fixture in [CASH, COMBINED] {
        assert_header_rejected(fixture, "symbol", serde_json::json!(42));
        assert_header_rejected(fixture, "date", serde_json::json!("2025-09-27T00:00:00Z"));
        assert_header_rejected(fixture, "fiscalYear", serde_json::json!(2025));
        assert_header_rejected(fixture, "period", serde_json::json!("annual"));
        assert_header_rejected(fixture, "reportedCurrency", serde_json::json!(false));
    }
}

fn assert_header_rejected(fixture: &[u8], key: &str, replacement: serde_json::Value) {
    let mut value: serde_json::Value = serde_json::from_slice(fixture).unwrap();
    value[0][key] = replacement;
    let cash_rejected =
        serde_json::from_value::<Vec<CashFlowStatementGrowth>>(value.clone()).is_err();
    let combined_rejected = serde_json::from_value::<Vec<FinancialStatementGrowth>>(value).is_err();
    assert!(cash_rejected && combined_rejected, "accepted invalid {key}");
}

#[test]
fn growth_values_are_raw_f64_ratios_without_percentage_scaling_or_range_limits() {
    let mut cash: serde_json::Value = serde_json::from_slice(CASH).unwrap();
    cash[0]["growthNetIncome"] = serde_json::json!(-1.0);
    cash[0]["growthInventory"] = serde_json::json!(2.5);
    let cash: Vec<CashFlowStatementGrowth> = serde_json::from_value(cash).unwrap();
    assert_eq!(cash[0].growth_net_income, -1.0);
    assert_eq!(cash[0].growth_inventory, 2.5);

    let mut combined: serde_json::Value = serde_json::from_slice(COMBINED).unwrap();
    combined[0]["revenueGrowth"] = serde_json::json!(-1.0);
    combined[0]["tenYRevenueGrowthPerShare"] = serde_json::json!(3.25);
    let combined: Vec<FinancialStatementGrowth> = serde_json::from_value(combined).unwrap();
    assert_eq!(combined[0].revenue_growth, -1.0);
    assert_eq!(combined[0].ten_y_revenue_growth_per_share, 3.25);

    let mut string_ratio: serde_json::Value = serde_json::from_slice(CASH).unwrap();
    string_ratio[0]["growthNetIncome"] = serde_json::json!("0.19");
    assert!(serde_json::from_value::<Vec<CashFlowStatementGrowth>>(string_ratio).is_err());
}

#[test]
fn both_growth_contracts_are_bare_arrays_preserving_empty_and_multiple_rows() {
    assert!(
        serde_json::from_slice::<Vec<CashFlowStatementGrowth>>(b"[]")
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_slice::<Vec<FinancialStatementGrowth>>(b"[]")
            .unwrap()
            .is_empty()
    );

    assert_multiple::<CashFlowStatementGrowth>(CASH);
    assert_multiple::<FinancialStatementGrowth>(COMBINED);

    let wrapped = serde_json::json!({ "financialGrowth": [] });
    assert!(serde_json::from_value::<Vec<FinancialStatementGrowth>>(wrapped).is_err());
}

fn assert_multiple<T: DeserializeOwned>(fixture: &[u8]) {
    let mut value: serde_json::Value = serde_json::from_slice(fixture).unwrap();
    let duplicate = value[0].clone();
    value.as_array_mut().unwrap().push(duplicate);
    assert_eq!(serde_json::from_value::<Vec<T>>(value).unwrap().len(), 2);
}
