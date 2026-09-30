#[macro_use]
#[path = "support/assert_row.rs"]
mod assert_row;

use std::str::FromStr;

use libfmp::{
    codecs::FiscalYearString,
    query::FiscalPeriod,
    responses::statements::{BalanceSheetStatement, BalanceSheetStatementTtm},
    types::{ApiDateTime, Cik, CurrencyCode, Date, Ticker},
};

const HISTORICAL: &[u8] = include_bytes!("fixtures/balance_sheet_statement.json");
const TTM: &[u8] = include_bytes!("fixtures/balance_sheet_statement_ttm.json");

#[test]
fn historical_fixture_decodes_all_61_documented_fields_exactly() {
    let value: serde_json::Value = serde_json::from_slice(HISTORICAL).unwrap();
    assert_eq!(value[0].as_object().unwrap().len(), 61);

    let rows: Vec<BalanceSheetStatement> = serde_json::from_value(value).unwrap();
    assert_eq!(rows.len(), 1);
    assert_historical(&rows[0]);
    assert_eq!(rows[0].total_assets, Some(359_241_000_000.0));
    assert_eq!(rows[0].retained_earnings, Some(-14_264_000_000.0));
    assert_eq!(rows[0].cik.as_str(), "0000320193");
    assert_eq!(rows[0].accepted_date.to_string(), "2025-10-31 06:01:26");
}

#[test]
fn ttm_fixture_decodes_its_distinct_60_field_contract_exactly() {
    let value: serde_json::Value = serde_json::from_slice(TTM).unwrap();
    assert_eq!(value[0].as_object().unwrap().len(), 60);

    let rows: Vec<BalanceSheetStatementTtm> = serde_json::from_value(value).unwrap();
    assert_eq!(rows.len(), 1);
    assert_ttm(&rows[0]);
    assert_eq!(rows[0].total_assets, 371_082_000_000.0);
    assert_eq!(
        rows[0].accumulated_other_comprehensive_income_loss,
        -5_375_000_000.0
    );

    let wrapped = serde_json::json!({ "balanceSheetStatement": rows });
    assert!(serde_json::from_value::<Vec<BalanceSheetStatementTtm>>(wrapped).is_err());
}

#[test]
fn both_bare_array_contracts_preserve_empty_and_multiple_rows() {
    assert!(
        serde_json::from_slice::<Vec<BalanceSheetStatement>>(b"[]")
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_slice::<Vec<BalanceSheetStatementTtm>>(b"[]")
            .unwrap()
            .is_empty()
    );

    let mut historical: serde_json::Value = serde_json::from_slice(HISTORICAL).unwrap();
    let row = historical[0].clone();
    historical.as_array_mut().unwrap().push(row);
    assert_eq!(
        serde_json::from_value::<Vec<BalanceSheetStatement>>(historical)
            .unwrap()
            .len(),
        2
    );

    let mut ttm: serde_json::Value = serde_json::from_slice(TTM).unwrap();
    let row = ttm[0].clone();
    ttm.as_array_mut().unwrap().push(row);
    assert_eq!(
        serde_json::from_value::<Vec<BalanceSheetStatementTtm>>(ttm)
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn historical_requires_the_field_that_the_documented_ttm_shape_omits() {
    let ttm_shape: serde_json::Value = serde_json::from_slice(TTM).unwrap();
    let error = serde_json::from_value::<Vec<BalanceSheetStatement>>(ttm_shape).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("capitalLeaseObligationsNonCurrent")
    );

    let mut historical: serde_json::Value = serde_json::from_slice(HISTORICAL).unwrap();
    historical[0]
        .as_object_mut()
        .unwrap()
        .remove("capitalLeaseObligationsNonCurrent");
    assert!(serde_json::from_value::<Vec<BalanceSheetStatement>>(historical).is_err());
}

#[test]
fn fiscal_year_rejects_a_numeric_json_value_in_both_contracts() {
    let mut historical: serde_json::Value = serde_json::from_slice(HISTORICAL).unwrap();
    historical[0]["fiscalYear"] = serde_json::json!(2025);
    let error = serde_json::from_value::<Vec<BalanceSheetStatement>>(historical).unwrap_err();
    assert!(error.to_string().contains("string"));

    let mut ttm: serde_json::Value = serde_json::from_slice(TTM).unwrap();
    ttm[0]["fiscalYear"] = serde_json::json!(2026);
    let error = serde_json::from_value::<Vec<BalanceSheetStatementTtm>>(ttm).unwrap_err();
    assert!(error.to_string().contains("string"));
}

#[test]
fn statement_amounts_preserve_large_integral_values_in_both_contracts() {
    for (field, extreme) in [
        (
            "totalAssets",
            serde_json::json!(9_000_000_000_000_000_000_i64),
        ),
        ("retainedEarnings", serde_json::json!(i64::MIN)),
    ] {
        let mut historical: serde_json::Value = serde_json::from_slice(HISTORICAL).unwrap();
        historical[0][field] = extreme.clone();
        let rows: Vec<BalanceSheetStatement> = serde_json::from_value(historical).unwrap();
        assert_eq!(serde_json::to_value(rows).unwrap()[0][field], extreme);

        let mut ttm: serde_json::Value = serde_json::from_slice(TTM).unwrap();
        ttm[0][field] = extreme.clone();
        let rows: Vec<BalanceSheetStatementTtm> = serde_json::from_value(ttm).unwrap();
        assert_eq!(serde_json::to_value(rows).unwrap()[0][field], extreme);
    }
}

const STUB_QUARTER_NULL_MEMBERS: [&str; 38] = [
    "accountPayables",
    "accountsReceivables",
    "accruedExpenses",
    "accumulatedOtherComprehensiveIncomeLoss",
    "additionalPaidInCapital",
    "capitalLeaseObligationsCurrent",
    "capitalLeaseObligationsNonCurrent",
    "cashAndCashEquivalents",
    "cashAndShortTermInvestments",
    "commonStock",
    "deferredRevenue",
    "deferredRevenueNonCurrent",
    "deferredTaxLiabilitiesNonCurrent",
    "goodwill",
    "goodwillAndIntangibleAssets",
    "intangibleAssets",
    "inventory",
    "longTermDebt",
    "longTermInvestments",
    "netReceivables",
    "otherPayables",
    "prepaids",
    "propertyPlantEquipmentNet",
    "retainedEarnings",
    "shortTermDebt",
    "shortTermInvestments",
    "taxAssets",
    "taxPayables",
    "totalAssets",
    "totalCurrentAssets",
    "totalCurrentLiabilities",
    "totalEquity",
    "totalLiabilities",
    "totalLiabilitiesAndTotalEquity",
    "totalNonCurrentAssets",
    "totalNonCurrentLiabilities",
    "totalStockholdersEquity",
    "treasuryStock",
];

#[test]
fn pre_ipo_stub_row_decodes_its_null_amounts_as_none() {
    let mut value: serde_json::Value = serde_json::from_slice(HISTORICAL).unwrap();
    for member in STUB_QUARTER_NULL_MEMBERS {
        value[0][member] = serde_json::Value::Null;
    }

    let rows: Vec<BalanceSheetStatement> = serde_json::from_value(value).unwrap();
    assert_eq!(rows[0].total_assets, None);
    assert!(rows[0].other_receivables.is_finite());

    let encoded = serde_json::to_value(&rows).unwrap();
    for member in STUB_QUARTER_NULL_MEMBERS {
        assert!(encoded[0][member].is_null(), "{member}");
    }
}

#[test]
fn null_on_an_amount_outside_the_stub_set_is_still_rejected() {
    let mut value: serde_json::Value = serde_json::from_slice(HISTORICAL).unwrap();
    value[0]["otherReceivables"] = serde_json::Value::Null;
    assert!(serde_json::from_value::<Vec<BalanceSheetStatement>>(value).is_err());
}

fn assert_historical(row: &BalanceSheetStatement) {
    assert_row!(
        row,
        BalanceSheetStatement {
            date: Date::from_str("2025-09-27").unwrap(),
            symbol: Ticker::new("AAPL").unwrap(),
            reported_currency: CurrencyCode::new("USD").unwrap(),
            cik: Cik::new("0000320193").unwrap(),
            filing_date: Date::from_str("2025-10-31").unwrap(),
            accepted_date: ApiDateTime::from_str("2025-10-31 06:01:26").unwrap(),
            fiscal_year: FiscalYearString::new("2025").unwrap(),
            period: FiscalPeriod::FullYear,
            cash_and_cash_equivalents: Some(35_934_000_000.0),
            short_term_investments: Some(18_763_000_000.0),
            cash_and_short_term_investments: Some(54_697_000_000.0),
            net_receivables: Some(72_957_000_000.0),
            accounts_receivables: Some(39_777_000_000.0),
            other_receivables: 33_180_000_000.0,
            inventory: Some(5_718_000_000.0),
            prepaids: Some(0.0),
            other_current_assets: 14_585_000_000.0,
            total_current_assets: Some(147_957_000_000.0),
            property_plant_equipment_net: Some(49_834_000_000.0),
            goodwill: Some(0.0),
            intangible_assets: Some(0.0),
            goodwill_and_intangible_assets: Some(0.0),
            long_term_investments: Some(77_723_000_000.0),
            tax_assets: Some(20_777_000_000.0),
            other_non_current_assets: 62_950_000_000.0,
            total_non_current_assets: Some(211_284_000_000.0),
            other_assets: 0.0,
            total_assets: Some(359_241_000_000.0),
            total_payables: 82_876_000_000.0,
            account_payables: Some(69_860_000_000.0),
            other_payables: Some(13_016_000_000.0),
            accrued_expenses: Some(8_919_000_000.0),
            short_term_debt: Some(20_329_000_000.0),
            capital_lease_obligations_current: Some(2_117_000_000.0),
            tax_payables: Some(0.0),
            deferred_revenue: Some(9_055_000_000.0),
            other_current_liabilities: 42_335_000_000.0,
            total_current_liabilities: Some(165_631_000_000.0),
            long_term_debt: Some(78_328_000_000.0),
            capital_lease_obligations_non_current: Some(11_603_000_000.0),
            deferred_revenue_non_current: Some(0.0),
            deferred_tax_liabilities_non_current: Some(0.0),
            other_non_current_liabilities: 29_946_000_000.0,
            total_non_current_liabilities: Some(119_877_000_000.0),
            other_liabilities: 0.0,
            capital_lease_obligations: 13_720_000_000.0,
            total_liabilities: Some(285_508_000_000.0),
            treasury_stock: Some(0.0),
            preferred_stock: 0.0,
            common_stock: Some(93_568_000_000.0),
            retained_earnings: Some(-14_264_000_000.0),
            additional_paid_in_capital: Some(0.0),
            accumulated_other_comprehensive_income_loss: Some(-5_571_000_000.0),
            other_total_stockholders_equity: 0.0,
            total_stockholders_equity: Some(73_733_000_000.0),
            total_equity: Some(73_733_000_000.0),
            minority_interest: 0.0,
            total_liabilities_and_total_equity: Some(359_241_000_000.0),
            total_investments: 96_486_000_000.0,
            total_debt: 112_377_000_000.0,
            net_debt: 76_443_000_000.0,
        }
    );
}

fn assert_ttm(row: &BalanceSheetStatementTtm) {
    assert_row!(
        row,
        BalanceSheetStatementTtm {
            date: Date::from_str("2026-03-28").unwrap(),
            symbol: Ticker::new("AAPL").unwrap(),
            reported_currency: CurrencyCode::new("USD").unwrap(),
            cik: Cik::new("0000320193").unwrap(),
            filing_date: Date::from_str("2026-05-01").unwrap(),
            accepted_date: ApiDateTime::from_str("2026-05-01 10:01:00").unwrap(),
            fiscal_year: FiscalYearString::new("2026").unwrap(),
            period: FiscalPeriod::Q2,
            cash_and_cash_equivalents: 36_328_000_000.0,
            short_term_investments: 32_179_000_000.0,
            cash_and_short_term_investments: 68_507_000_000.0,
            net_receivables: 53_511_000_000.0,
            accounts_receivables: 30_339_000_000.0,
            other_receivables: 23_172_000_000.0,
            inventory: 6_747_000_000.0,
            prepaids: 0.0,
            other_current_assets: 15_349_000_000.0,
            total_current_assets: 144_114_000_000.0,
            property_plant_equipment_net: 50_116_000_000.0,
            goodwill: 0.0,
            intangible_assets: 21_334_000_000.0,
            goodwill_and_intangible_assets: 21_334_000_000.0,
            long_term_investments: 78_088_000_000.0,
            tax_assets: 0.0,
            other_non_current_assets: 77_430_000_000.0,
            total_non_current_assets: 226_968_000_000.0,
            other_assets: 0.0,
            total_assets: 371_082_000_000.0,
            total_payables: 57_349_000_000.0,
            account_payables: 57_349_000_000.0,
            other_payables: 0.0,
            accrued_expenses: 0.0,
            short_term_debt: 10_307_000_000.0,
            capital_lease_obligations_current: 0.0,
            tax_payables: 0.0,
            deferred_revenue: 9_331_000_000.0,
            other_current_liabilities: 57_654_000_000.0,
            total_current_liabilities: 134_641_000_000.0,
            long_term_debt: 74_404_000_000.0,
            deferred_revenue_non_current: 0.0,
            deferred_tax_liabilities_non_current: 0.0,
            other_non_current_liabilities: 55_546_000_000.0,
            total_non_current_liabilities: 129_950_000_000.0,
            other_liabilities: 0.0,
            capital_lease_obligations: 0.0,
            total_liabilities: 264_591_000_000.0,
            treasury_stock: 0.0,
            preferred_stock: 0.0,
            common_stock: 99_507_000_000.0,
            retained_earnings: 12_359_000_000.0,
            additional_paid_in_capital: 0.0,
            accumulated_other_comprehensive_income_loss: -5_375_000_000.0,
            other_total_stockholders_equity: 0.0,
            total_stockholders_equity: 106_491_000_000.0,
            total_equity: 106_491_000_000.0,
            minority_interest: 0.0,
            total_liabilities_and_total_equity: 371_082_000_000.0,
            total_investments: 110_267_000_000.0,
            total_debt: 84_711_000_000.0,
            net_debt: 48_383_000_000.0,
        }
    );
}
