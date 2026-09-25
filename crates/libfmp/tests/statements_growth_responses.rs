#[macro_use]
#[path = "support/assert_row.rs"]
mod assert_row;

use std::str::FromStr;

use libfmp::{
    codecs::FiscalYearString,
    query::FiscalPeriod,
    responses::statements::{BalanceSheetStatementGrowth, IncomeStatementGrowth},
    types::{CurrencyCode, Date, Ticker},
};

const INCOME: &[u8] = include_bytes!("fixtures/income_statement_growth.json");
const BALANCE: &[u8] = include_bytes!("fixtures/balance_sheet_statement_growth.json");

#[test]
fn income_fixture_decodes_all_34_documented_fields_without_scaling() {
    let value: serde_json::Value = serde_json::from_slice(INCOME).unwrap();
    assert_eq!(value[0].as_object().unwrap().len(), 34);

    let rows: Vec<IncomeStatementGrowth> = serde_json::from_value(value).unwrap();
    assert_eq!(rows.len(), 1);
    assert_income(&rows[0]);
    assert_eq!(rows[0].growth_selling_and_marketing_expenses, -1.0);
    assert_eq!(
        rows[0].growth_general_and_administrative_expenses,
        2.700858138911236
    );
    assert_eq!(
        rows[0].growth_total_other_income_expenses_net,
        -2.193308550185874
    );

    let encoded = serde_json::to_value(&rows[0]).unwrap();
    for exact_acronym in [
        "growthEBITDA",
        "growthEPS",
        "growthEPSDiluted",
        "growthEBIT",
    ] {
        assert!(encoded.get(exact_acronym).is_some());
    }
    for wrong_casing in [
        "growthEbitda",
        "growthEps",
        "growthEpsDiluted",
        "growthEbit",
    ] {
        assert!(encoded.get(wrong_casing).is_none());
    }
}

#[test]
fn balance_fixture_decodes_all_56_documented_fields_without_scaling() {
    let value: serde_json::Value = serde_json::from_slice(BALANCE).unwrap();
    assert_eq!(value[0].as_object().unwrap().len(), 56);

    let rows: Vec<BalanceSheetStatementGrowth> = serde_json::from_value(value).unwrap();
    assert_eq!(rows.len(), 1);
    assert_balance(&rows[0]);
    assert_eq!(rows[0].growth_tax_payables, -1.0);
    assert_eq!(rows[0].growth_other_payables, -0.5106950866508778);

    let encoded = serde_json::to_value(&rows[0]).unwrap();
    assert_eq!(encoded["growthOthertotalStockholdersEquity"], 0.0);
    assert_eq!(
        encoded["growthTotalLiabilitiesAndStockholdersEquity"],
        -0.015724149268453065
    );
    assert!(encoded.get("growthOtherTotalStockholdersEquity").is_none());
}

#[test]
fn every_documented_field_is_required_by_its_distinct_row_contract() {
    assert_every_field_required::<IncomeStatementGrowth>(INCOME, 34);
    assert_every_field_required::<BalanceSheetStatementGrowth>(BALANCE, 56);
}

fn assert_every_field_required<T>(fixture: &[u8], expected_fields: usize)
where
    T: serde::de::DeserializeOwned,
{
    let value: serde_json::Value = serde_json::from_slice(fixture).unwrap();
    let row = value[0].as_object().unwrap();
    assert_eq!(row.len(), expected_fields);

    for key in row.keys() {
        let mut missing = value.clone();
        missing[0].as_object_mut().unwrap().remove(key);
        assert!(
            serde_json::from_value::<Vec<T>>(missing).is_err(),
            "{key} unexpectedly became optional"
        );
    }
}

#[test]
fn strict_headers_and_bare_arrays_are_preserved() {
    let mut numeric_income_year: serde_json::Value = serde_json::from_slice(INCOME).unwrap();
    numeric_income_year[0]["fiscalYear"] = serde_json::json!(2025);
    assert!(serde_json::from_value::<Vec<IncomeStatementGrowth>>(numeric_income_year).is_err());

    let mut numeric_balance_year: serde_json::Value = serde_json::from_slice(BALANCE).unwrap();
    numeric_balance_year[0]["fiscalYear"] = serde_json::json!(2025);
    assert!(
        serde_json::from_value::<Vec<BalanceSheetStatementGrowth>>(numeric_balance_year).is_err()
    );

    assert!(
        serde_json::from_slice::<Vec<IncomeStatementGrowth>>(b"[]")
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_slice::<Vec<BalanceSheetStatementGrowth>>(b"[]")
            .unwrap()
            .is_empty()
    );

    let mut multiple: serde_json::Value = serde_json::from_slice(INCOME).unwrap();
    let row = multiple[0].clone();
    multiple.as_array_mut().unwrap().push(row);
    assert_eq!(
        serde_json::from_value::<Vec<IncomeStatementGrowth>>(multiple)
            .unwrap()
            .len(),
        2
    );

    let wrapped = serde_json::json!({ "incomeStatementGrowth": serde_json::from_slice::<serde_json::Value>(INCOME).unwrap() });
    assert!(serde_json::from_value::<Vec<IncomeStatementGrowth>>(wrapped).is_err());
}

fn assert_income(row: &IncomeStatementGrowth) {
    assert_row!(
        row,
        IncomeStatementGrowth {
            symbol: Ticker::new("AAPL").unwrap(),
            date: Date::from_str("2025-09-27").unwrap(),
            fiscal_year: FiscalYearString::new("2025").unwrap(),
            period: FiscalPeriod::FullYear,
            reported_currency: CurrencyCode::new("USD").unwrap(),
            growth_revenue: 0.0642551178283274,
            growth_cost_of_revenue: 0.050429755837833726,
            growth_gross_profit: 0.08035066940442653,
            growth_gross_profit_ratio: 0.015123771791588903,
            growth_research_and_development_expenses: 0.10137073637233025,
            growth_general_and_administrative_expenses: 2.700858138911236,
            growth_selling_and_marketing_expenses: -1.0,
            growth_other_expenses: 0.0,
            growth_operating_expenses: 0.08150764786747176,
            growth_cost_and_expenses: 0.057098264126144896,
            growth_interest_income: 0.0,
            growth_interest_expense: 0.0,
            growth_depreciation_and_amortization: 0.022105723023154215,
            growth_ebitda: 0.07038464388942414,
            growth_operating_income: 0.07981106349824699,
            growth_income_before_tax: 0.0748592946511722,
            growth_income_tax_expense: -0.3035396147769673,
            growth_net_income: 0.19495177946573355,
            growth_eps: 0.22585924713584285,
            growth_eps_diluted: 0.2269736842105263,
            growth_weighted_average_shs_out: -0.025761769441082424,
            growth_weighted_average_shs_out_dil: -0.02618091334457634,
            growth_ebit: 0.0748592946511722,
            growth_non_operating_income_excluding_interest: 2.193308550185874,
            growth_net_interest_income: 0.0,
            growth_total_other_income_expenses_net: -2.193308550185874,
            growth_net_income_from_continuing_operations: 0.19495177946573355,
            growth_other_adjustments_to_net_income: 0.0,
            growth_net_income_deductions: 0.0,
        }
    );
}

fn assert_balance(row: &BalanceSheetStatementGrowth) {
    assert_row!(
        row,
        BalanceSheetStatementGrowth {
            symbol: Ticker::new("AAPL").unwrap(),
            date: Date::from_str("2025-09-27").unwrap(),
            fiscal_year: FiscalYearString::new("2025").unwrap(),
            period: FiscalPeriod::FullYear,
            reported_currency: CurrencyCode::new("USD").unwrap(),
            growth_cash_and_cash_equivalents: 0.20008015228934978,
            growth_short_term_investments: -0.4673838991711139,
            growth_cash_and_short_term_investments: -0.1607156557364472,
            growth_net_receivables: 0.1013541053394321,
            growth_inventory: -0.2152072467746363,
            growth_other_current_assets: 0.020858122768950795,
            growth_total_current_assets: -0.03287861060090073,
            growth_property_plant_equipment_net: 0.0909369527145359,
            growth_goodwill: 0.0,
            growth_intangible_assets: 0.0,
            growth_goodwill_and_intangible_assets: 0.0,
            growth_long_term_investments: -0.15037330972135682,
            growth_tax_assets: 0.06554182265757218,
            growth_other_non_current_assets: 0.13761633685732358,
            growth_total_non_current_assets: -0.0033444500525960765,
            growth_other_assets: 0.0,
            growth_total_assets: -0.015724149268453065,
            growth_account_payables: 0.013051044083526682,
            growth_short_term_debt: -0.026342257770966042,
            growth_tax_payables: -1.0,
            growth_deferred_revenue: 0.09770881318947751,
            growth_other_current_liabilities: -0.15450060913502825,
            growth_total_current_liabilities: -0.061006168080185046,
            growth_long_term_debt: -0.08655393586005831,
            growth_deferred_revenue_non_current: 0.0,
            growth_deferred_tax_liabilities_non_current: 0.0,
            growth_other_non_current_liabilities: -0.14659447135936163,
            growth_total_non_current_liabilities: -0.08934350263601695,
            growth_other_liabilities: 0.0,
            growth_total_liabilities: -0.07311625491023602,
            growth_preferred_stock: 0.0,
            growth_common_stock: 0.12358902925212546,
            growth_retained_earnings: 0.25529915422366084,
            growth_accumulated_other_comprehensive_income_loss: 0.22322922476296708,
            growth_other_total_stockholders_equity: 0.0,
            growth_total_stockholders_equity: 0.2946971027216857,
            growth_minority_interest: 0.0,
            growth_total_equity: 0.2946971027216857,
            growth_total_liabilities_and_stockholders_equity: -0.015724149268453065,
            growth_total_investments: -0.2385108952149447,
            growth_total_debt: -0.05612343459965227,
            growth_net_debt: -0.1422079087930338,
            growth_accounts_receivables: 0.1905716851242143,
            growth_other_receivables: 0.010568635214570707,
            growth_prepaids: 0.0,
            growth_total_payables: -0.13274243676813763,
            growth_other_payables: -0.5106950866508778,
            growth_accrued_expenses: 0.0,
            growth_capital_lease_obligations_current: 0.2971813725490196,
            growth_additional_paid_in_capital: 0.0,
            growth_treasury_stock: 0.0,
        }
    );
}
