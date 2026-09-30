#[macro_use]
#[path = "support/assert_row.rs"]
mod assert_row;

use std::str::FromStr;

use libfmp::{
    codecs::FiscalYearString,
    query::FiscalPeriod,
    responses::statements::CashFlowStatement,
    types::{ApiDateTime, Cik, CurrencyCode, Date, Ticker},
};

const HISTORICAL: &[u8] = include_bytes!("fixtures/cash_flow_statement.json");
const TTM: &[u8] = include_bytes!("fixtures/cash_flow_statement_ttm.json");

#[test]
fn historical_fixture_decodes_all_47_documented_fields_exactly() {
    let value: serde_json::Value = serde_json::from_slice(HISTORICAL).unwrap();
    assert_eq!(value[0].as_object().unwrap().len(), 47);

    let rows: Vec<CashFlowStatement> = serde_json::from_value(value).unwrap();
    assert_eq!(rows.len(), 1);
    assert_historical(&rows[0]);
    assert_eq!(
        rows[0].net_cash_provided_by_operating_activities,
        111_482_000_000.0
    );
    assert_eq!(
        rows[0].net_cash_provided_by_financing_activities,
        Some(-120_686_000_000.0)
    );
    assert_eq!(rows[0].cik.as_str(), "0000320193");
    assert_eq!(rows[0].accepted_date.to_string(), "2025-10-31 06:01:26");
}

#[test]
fn ttm_fixture_reuses_the_same_47_field_bare_array_contract() {
    let value: serde_json::Value = serde_json::from_slice(TTM).unwrap();
    assert_eq!(value[0].as_object().unwrap().len(), 47);

    let rows: Vec<CashFlowStatement> = serde_json::from_value(value).unwrap();
    assert_eq!(rows.len(), 1);
    assert_ttm(&rows[0]);
    assert_eq!(
        rows[0].net_cash_provided_by_investing_activities,
        -8_568_000_000.0
    );
    assert_eq!(rows[0].free_cash_flow, 129_174_000_000.0);

    let wrapped = serde_json::json!({ "cashFlowStatement": rows });
    assert!(serde_json::from_value::<Vec<CashFlowStatement>>(wrapped).is_err());
}

#[test]
fn bare_array_contract_preserves_empty_and_multiple_rows() {
    assert!(
        serde_json::from_slice::<Vec<CashFlowStatement>>(b"[]")
            .unwrap()
            .is_empty()
    );

    for fixture in [HISTORICAL, TTM] {
        let mut value: serde_json::Value = serde_json::from_slice(fixture).unwrap();
        let row = value[0].clone();
        value.as_array_mut().unwrap().push(row);
        assert_eq!(
            serde_json::from_value::<Vec<CashFlowStatement>>(value)
                .unwrap()
                .len(),
            2
        );
    }
}

#[test]
fn fiscal_year_rejects_a_numeric_json_value_for_both_endpoints() {
    for fixture in [HISTORICAL, TTM] {
        let mut value: serde_json::Value = serde_json::from_slice(fixture).unwrap();
        value[0]["fiscalYear"] = serde_json::json!(2026);
        let error = serde_json::from_value::<Vec<CashFlowStatement>>(value).unwrap_err();
        assert!(error.to_string().contains("string"));
    }
}

#[test]
fn statement_amounts_preserve_large_integral_values_for_both_endpoints() {
    for fixture in [HISTORICAL, TTM] {
        for (field, extreme) in [
            (
                "freeCashFlow",
                serde_json::json!(9_000_000_000_000_000_000_i64),
            ),
            (
                "netCashProvidedByFinancingActivities",
                serde_json::json!(i64::MIN),
            ),
        ] {
            let mut value: serde_json::Value = serde_json::from_slice(fixture).unwrap();
            value[0][field] = extreme.clone();
            let rows: Vec<CashFlowStatement> = serde_json::from_value(value).unwrap();
            assert_eq!(serde_json::to_value(rows).unwrap()[0][field], extreme);
        }
    }
}

const STUB_QUARTER_NULL_MEMBERS: [&str; 19] = [
    "accountsPayables",
    "accountsReceivables",
    "cashAtBeginningOfPeriod",
    "commonDividendsPaid",
    "commonStockRepurchased",
    "deferredIncomeTax",
    "incomeTaxesPaid",
    "interestPaid",
    "inventory",
    "longTermNetDebtIssuance",
    "netCashProvidedByFinancingActivities",
    "netCommonStockIssuance",
    "netDividendsPaid",
    "netPreferredStockIssuance",
    "netStockIssuance",
    "preferredDividendsPaid",
    "purchasesOfInvestments",
    "salesMaturitiesOfInvestments",
    "stockBasedCompensation",
];

#[test]
fn pre_ipo_stub_row_decodes_its_null_amounts_as_none() {
    let mut value: serde_json::Value = serde_json::from_slice(HISTORICAL).unwrap();
    for member in STUB_QUARTER_NULL_MEMBERS {
        value[0][member] = serde_json::Value::Null;
    }

    let rows: Vec<CashFlowStatement> = serde_json::from_value(value).unwrap();
    assert_eq!(rows[0].stock_based_compensation, None);
    assert!(rows[0].net_income.is_finite());

    let encoded = serde_json::to_value(&rows).unwrap();
    for member in STUB_QUARTER_NULL_MEMBERS {
        assert!(encoded[0][member].is_null(), "{member}");
    }
}

#[test]
fn null_on_an_amount_outside_the_stub_set_is_still_rejected() {
    let mut value: serde_json::Value = serde_json::from_slice(HISTORICAL).unwrap();
    value[0]["netIncome"] = serde_json::Value::Null;
    assert!(serde_json::from_value::<Vec<CashFlowStatement>>(value).is_err());
}

#[test]
fn missing_stub_amount_key_is_still_rejected() {
    let mut value: serde_json::Value = serde_json::from_slice(HISTORICAL).unwrap();
    value[0]
        .as_object_mut()
        .unwrap()
        .remove("stockBasedCompensation");
    let error = serde_json::from_value::<Vec<CashFlowStatement>>(value).unwrap_err();
    assert!(error.to_string().contains("stockBasedCompensation"));
}

fn assert_historical(row: &CashFlowStatement) {
    assert_row!(
        row,
        CashFlowStatement {
            date: Date::from_str("2025-09-27").unwrap(),
            symbol: Ticker::new("AAPL").unwrap(),
            reported_currency: CurrencyCode::new("USD").unwrap(),
            cik: Cik::new("0000320193").unwrap(),
            filing_date: Date::from_str("2025-10-31").unwrap(),
            accepted_date: ApiDateTime::from_str("2025-10-31 06:01:26").unwrap(),
            fiscal_year: FiscalYearString::new("2025").unwrap(),
            period: FiscalPeriod::FullYear,
            net_income: 112_010_000_000.0,
            depreciation_and_amortization: 11_698_000_000.0,
            deferred_income_tax: Some(0.0),
            stock_based_compensation: Some(12_863_000_000.0),
            change_in_working_capital: -25_000_000_000.0,
            accounts_receivables: Some(-7_029_000_000.0),
            inventory: Some(1_400_000_000.0),
            accounts_payables: Some(902_000_000.0),
            other_working_capital: -20_273_000_000.0,
            other_non_cash_items: -89_000_000.0,
            net_cash_provided_by_operating_activities: 111_482_000_000.0,
            investments_in_property_plant_and_equipment: -12_715_000_000.0,
            acquisitions_net: 0.0,
            purchases_of_investments: Some(-24_407_000_000.0),
            sales_maturities_of_investments: Some(53_797_000_000.0),
            other_investing_activities: -1_480_000_000.0,
            net_cash_provided_by_investing_activities: 15_195_000_000.0,
            net_debt_issuance: -8_483_000_000.0,
            long_term_net_debt_issuance: Some(-6_451_000_000.0),
            short_term_net_debt_issuance: -2_032_000_000.0,
            net_stock_issuance: Some(-90_711_000_000.0),
            net_common_stock_issuance: Some(-90_711_000_000.0),
            common_stock_issuance: 0.0,
            common_stock_repurchased: Some(-90_711_000_000.0),
            net_preferred_stock_issuance: Some(0.0),
            net_dividends_paid: Some(-15_421_000_000.0),
            common_dividends_paid: Some(-15_421_000_000.0),
            preferred_dividends_paid: Some(0.0),
            other_financing_activities: -6_071_000_000.0,
            net_cash_provided_by_financing_activities: Some(-120_686_000_000.0),
            effect_of_forex_changes_on_cash: 0.0,
            net_change_in_cash: 5_991_000_000.0,
            cash_at_end_of_period: 35_934_000_000.0,
            cash_at_beginning_of_period: Some(29_943_000_000.0),
            operating_cash_flow: 111_482_000_000.0,
            capital_expenditure: -12_715_000_000.0,
            free_cash_flow: 98_767_000_000.0,
            income_taxes_paid: Some(43_369_000_000.0),
            interest_paid: Some(0.0),
        }
    );
}

fn assert_ttm(row: &CashFlowStatement) {
    assert_row!(
        row,
        CashFlowStatement {
            date: Date::from_str("2026-03-28").unwrap(),
            symbol: Ticker::new("AAPL").unwrap(),
            reported_currency: CurrencyCode::new("USD").unwrap(),
            cik: Cik::new("0000320193").unwrap(),
            filing_date: Date::from_str("2026-05-01").unwrap(),
            accepted_date: ApiDateTime::from_str("2026-05-01 10:01:00").unwrap(),
            fiscal_year: FiscalYearString::new("2026").unwrap(),
            period: FiscalPeriod::Q2,
            net_income: 122_575_000_000.0,
            depreciation_and_amortization: 12_610_000_000.0,
            deferred_income_tax: Some(0.0),
            stock_based_compensation: Some(13_473_000_000.0),
            change_in_working_capital: -8_847_000_000.0,
            accounts_receivables: Some(-4_163_000_000.0),
            inventory: Some(-542_000_000.0),
            accounts_payables: Some(3_209_000_000.0),
            other_working_capital: -7_351_000_000.0,
            other_non_cash_items: 411_000_000.0,
            net_cash_provided_by_operating_activities: 140_222_000_000.0,
            investments_in_property_plant_and_equipment: -11_048_000_000.0,
            acquisitions_net: 0.0,
            purchases_of_investments: Some(-44_397_000_000.0),
            sales_maturities_of_investments: Some(49_306_000_000.0),
            other_investing_activities: -2_429_000_000.0,
            net_cash_provided_by_investing_activities: -8_568_000_000.0,
            net_debt_issuance: -14_331_000_000.0,
            long_term_net_debt_issuance: Some(-10_356_000_000.0),
            short_term_net_debt_issuance: -3_975_000_000.0,
            net_stock_issuance: Some(-78_196_000_000.0),
            net_common_stock_issuance: Some(-78_196_000_000.0),
            common_stock_issuance: 0.0,
            common_stock_repurchased: Some(-78_196_000_000.0),
            net_preferred_stock_issuance: Some(0.0),
            net_dividends_paid: Some(-15_550_000_000.0),
            common_dividends_paid: Some(-15_550_000_000.0),
            preferred_dividends_paid: Some(0.0),
            other_financing_activities: -6_167_000_000.0,
            net_cash_provided_by_financing_activities: Some(-114_244_000_000.0),
            effect_of_forex_changes_on_cash: 0.0,
            net_change_in_cash: 17_410_000_000.0,
            cash_at_end_of_period: 45_572_000_000.0,
            cash_at_beginning_of_period: Some(28_162_000_000.0),
            operating_cash_flow: 140_222_000_000.0,
            capital_expenditure: -11_048_000_000.0,
            free_cash_flow: 129_174_000_000.0,
            income_taxes_paid: Some(-11_286_000_000.0),
            interest_paid: Some(0.0),
        }
    );
}
