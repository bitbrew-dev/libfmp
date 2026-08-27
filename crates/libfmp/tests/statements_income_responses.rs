use std::str::FromStr;

use libfmp::{
    codecs::FiscalYearString,
    query::FiscalPeriod,
    responses::statements::IncomeStatement,
    types::{ApiDateTime, Cik, CurrencyCode, Date, Ticker},
};

const HISTORICAL: &[u8] = include_bytes!("fixtures/income_statement.json");
const TTM: &[u8] = include_bytes!("fixtures/income_statement_ttm.json");

#[test]
fn historical_fixture_decodes_all_39_documented_fields_exactly() {
    let rows: Vec<IncomeStatement> = serde_json::from_slice(HISTORICAL).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0], historical_expected());

    assert_eq!(rows[0].revenue, 416_161_000_000);
    assert_eq!(rows[0].total_other_income_expenses_net, -321_000_000);
    assert_eq!(rows[0].cik.as_str(), "0000320193");
    assert_eq!(rows[0].accepted_date.to_string(), "2025-10-31 06:01:26");
}

#[test]
fn ttm_fixture_uses_the_identical_bare_array_row_contract() {
    let rows: Vec<IncomeStatement> = serde_json::from_slice(TTM).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0], ttm_expected());
    assert_eq!(
        rows[0].non_operating_income_excluding_interest,
        -408_000_000
    );

    let wrapped = serde_json::json!({ "incomeStatement": rows });
    assert!(serde_json::from_value::<Vec<IncomeStatement>>(wrapped).is_err());
}

#[test]
fn fiscal_year_rejects_a_numeric_json_value() {
    let mut value: serde_json::Value = serde_json::from_slice(HISTORICAL).unwrap();
    value[0]["fiscalYear"] = serde_json::json!(2025);

    let error = serde_json::from_value::<Vec<IncomeStatement>>(value).unwrap_err();
    assert!(error.to_string().contains("string"));
}

fn historical_expected() -> IncomeStatement {
    IncomeStatement {
        date: Date::from_str("2025-09-27").unwrap(),
        symbol: Ticker::new("AAPL").unwrap(),
        reported_currency: CurrencyCode::new("USD").unwrap(),
        cik: Cik::new("0000320193").unwrap(),
        filing_date: Date::from_str("2025-10-31").unwrap(),
        accepted_date: ApiDateTime::from_str("2025-10-31 06:01:26").unwrap(),
        fiscal_year: FiscalYearString::new("2025").unwrap(),
        period: FiscalPeriod::FullYear,
        revenue: 416_161_000_000,
        cost_of_revenue: 220_960_000_000,
        gross_profit: 195_201_000_000,
        research_and_development_expenses: 34_550_000_000,
        general_and_administrative_expenses: 27_601_000_000,
        selling_and_marketing_expenses: 0,
        selling_general_and_administrative_expenses: 27_601_000_000,
        other_expenses: 0,
        operating_expenses: 62_151_000_000,
        cost_and_expenses: 283_111_000_000,
        net_interest_income: 0,
        interest_income: 0,
        interest_expense: 0,
        depreciation_and_amortization: 11_698_000_000,
        ebitda: 144_427_000_000,
        ebit: 132_729_000_000,
        non_operating_income_excluding_interest: 321_000_000,
        operating_income: 133_050_000_000,
        total_other_income_expenses_net: -321_000_000,
        income_before_tax: 132_729_000_000,
        income_tax_expense: 20_719_000_000,
        net_income_from_continuing_operations: 112_010_000_000,
        net_income_from_discontinued_operations: 0,
        other_adjustments_to_net_income: 0,
        net_income: 112_010_000_000,
        net_income_deductions: 0,
        bottom_line_net_income: 112_010_000_000,
        eps: 7.49,
        eps_diluted: 7.46,
        weighted_average_shs_out: 14_948_500_000,
        weighted_average_shs_out_dil: 15_004_697_000,
    }
}

fn ttm_expected() -> IncomeStatement {
    IncomeStatement {
        date: Date::from_str("2026-03-28").unwrap(),
        symbol: Ticker::new("AAPL").unwrap(),
        reported_currency: CurrencyCode::new("USD").unwrap(),
        cik: Cik::new("0000320193").unwrap(),
        filing_date: Date::from_str("2026-05-01").unwrap(),
        accepted_date: ApiDateTime::from_str("2026-05-01 10:01:00").unwrap(),
        fiscal_year: FiscalYearString::new("2026").unwrap(),
        period: FiscalPeriod::Q2,
        revenue: 451_442_000_000,
        cost_of_revenue: 235_371_000_000,
        gross_profit: 216_071_000_000,
        research_and_development_expenses: 40_038_000_000,
        general_and_administrative_expenses: 2_095_000_000,
        selling_and_marketing_expenses: 5_397_000_000,
        selling_general_and_administrative_expenses: 28_667_000_000,
        other_expenses: 0,
        operating_expenses: 68_705_000_000,
        cost_and_expenses: 304_076_000_000,
        net_interest_income: 0,
        interest_income: 0,
        interest_expense: 0,
        depreciation_and_amortization: 12_610_000_000,
        ebitda: 160_332_000_000,
        ebit: 147_670_000_000,
        non_operating_income_excluding_interest: -408_000_000,
        operating_income: 147_366_000_000,
        total_other_income_expenses_net: 304_000_000,
        income_before_tax: 147_670_000_000,
        income_tax_expense: 25_095_000_000,
        net_income_from_continuing_operations: 122_575_000_000,
        net_income_from_discontinued_operations: 0,
        other_adjustments_to_net_income: 0,
        net_income: 122_575_000_000,
        net_income_deductions: 0,
        bottom_line_net_income: 122_575_000_000,
        eps: 8.29,
        eps_diluted: 8.27,
        weighted_average_shs_out: 14_710_718_000,
        weighted_average_shs_out_dil: 14_768_115_000,
    }
}
