use std::str::FromStr;

use libfmp::{
    codecs::FiscalYearString,
    query::FiscalPeriod,
    responses::statements::{KeyMetrics, KeyMetricsTtm},
    types::{CurrencyCode, Date, Ticker},
};
use serde::de::DeserializeOwned;

const HISTORICAL: &[u8] = include_bytes!("fixtures/key_metrics.json");
const TTM: &[u8] = include_bytes!("fixtures/key_metrics_ttm.json");

#[test]
fn documented_key_metrics_decodes_all_47_exact_fields() {
    let value: serde_json::Value = serde_json::from_slice(HISTORICAL).unwrap();
    assert_eq!(value[0].as_object().unwrap().len(), 47);

    let rows: Vec<KeyMetrics> = serde_json::from_value(value).unwrap();
    assert_eq!(
        rows,
        [KeyMetrics {
            symbol: Ticker::new("AAPL").unwrap(),
            date: Date::from_str("2025-09-27").unwrap(),
            fiscal_year: FiscalYearString::new("2025").unwrap(),
            period: FiscalPeriod::FullYear,
            reported_currency: CurrencyCode::new("USD").unwrap(),
            market_cap: 3_818_743_810_000.0,
            enterprise_value: 3_895_186_810_000.0,
            ev_to_sales: 9.359807406268247,
            ev_to_operating_cash_flow: 34.94005139843203,
            ev_to_free_cash_flow: 39.43814037077161,
            ev_to_ebitda: 26.969935053694947,
            net_debt_to_ebitda: 0.5292846905357032,
            current_ratio: 0.8932929222186667,
            income_quality: 0.995286135166503,
            graham_number: 28.8371936709443,
            graham_net_net: -11.588738000468274,
            tax_burden: 0.8438999766441395,
            interest_burden: 1.0,
            working_capital: -17_674_000_000.0,
            invested_capital: 32_160_000_000.0,
            return_on_assets: 0.3117962593356549,
            operating_return_on_assets: 0.36742927918411644,
            return_on_tangible_assets: 0.3117962593356549,
            return_on_equity: 1.5191298333175105,
            return_on_invested_capital: 0.5196842110031786,
            return_on_capital_employed: 0.6872062393471412,
            earnings_yield: 0.029331635106467118,
            free_cash_flow_yield: 0.02586374077814872,
            capex_to_operating_cash_flow: 0.11405428679069267,
            capex_to_depreciation: 1.0869379381090785,
            capex_to_revenue: 0.030553079216937677,
            sales_general_and_administrative_to_revenue: 0.06632288945864702,
            research_and_developement_to_revenue: 0.08302075398703868,
            stock_based_compensation_to_revenue: 0.030908710811440764,
            intangibles_to_total_assets: 0.0,
            average_receivables: 69_600_000_000.0,
            average_payables: 69_410_000_000.0,
            average_inventory: 6_502_000_000.0,
            days_of_sales_outstanding: 63.9879878220208,
            days_of_payables_outstanding: 115.40052498189719,
            days_of_inventory_outstanding: 9.445465242577843,
            operating_cycle: 73.43345306459864,
            cash_conversion_cycle: -41.967071917298554,
            free_cash_flow_to_equity: 90_284_000_000.0,
            free_cash_flow_to_firm: 98_767_000_000.0,
            tangible_asset_value: 73_733_000_000.0,
            net_current_asset_value: -137_551_000_000.0,
        }]
    );
}

#[test]
fn documented_key_metrics_ttm_decodes_all_43_exact_fields() {
    let value: serde_json::Value = serde_json::from_slice(TTM).unwrap();
    assert_eq!(value[0].as_object().unwrap().len(), 43);

    let rows: Vec<KeyMetricsTtm> = serde_json::from_value(value).unwrap();
    assert_eq!(
        rows,
        [KeyMetricsTtm {
            symbol: Ticker::new("AAPL").unwrap(),
            market_cap: 4_874_072_686_740.0,
            enterprise_value_ttm: 4_922_455_686_740.0,
            ev_to_sales_ttm: 10.903849634593149,
            ev_to_operating_cash_flow_ttm: 35.10473168789491,
            ev_to_free_cash_flow_ttm: 38.107170845061695,
            ev_to_ebitda_ttm: 30.701642134695508,
            net_debt_to_ebitda_ttm: 0.3017675822667964,
            current_ratio_ttm: 1.07035746912159,
            income_quality_ttm: 1.1439689985723027,
            graham_number_ttm: 36.83959035988331,
            graham_net_net_ttm: -10.37184248926531,
            tax_burden_ttm: 0.8300602695198754,
            interest_burden_ttm: 1.0,
            working_capital_ttm: 9_473_000_000.0,
            invested_capital_ttm: 80_923_000_000.0,
            return_on_assets_ttm: 0.3303178273265747,
            operating_return_on_assets_ttm: 0.3927775164283649,
            return_on_tangible_assets_ttm: 0.3504666216818967,
            return_on_equity_ttm: 1.4668924498270723,
            return_on_invested_capital_ttm: 0.49573922251878827,
            return_on_capital_employed_ttm: 0.6232675382019193,
            earnings_yield_ttm: 0.025108435518318362,
            free_cash_flow_yield_ttm: 0.02650227198117503,
            capex_to_operating_cash_flow_ttm: 0.07878934831909401,
            capex_to_depreciation_ttm: 0.8761300555114988,
            capex_to_revenue_ttm: 0.02447268973644455,
            sales_general_and_administrative_to_revenue_ttm: 0.06350095914868355,
            research_and_developement_to_revenue_ttm: 0.08868913393082611,
            stock_based_compensation_to_revenue_ttm: 0.02984436538913083,
            intangibles_to_total_assets_ttm: 0.057491336146727676,
            average_receivables_ttm: 61_915_500_000.0,
            average_payables_ttm: 63_968_000_000.0,
            average_inventory_ttm: 6_311_000_000.0,
            days_of_sales_outstanding_ttm: 43.26472725178429,
            days_of_payables_outstanding_ttm: 88.93357720364871,
            days_of_inventory_outstanding_ttm: 10.462865008858357,
            operating_cycle_ttm: 53.727592260642645,
            cash_conversion_cycle_ttm: -35.205984943006065,
            free_cash_flow_to_equity_ttm: 114_843_000_000.0,
            free_cash_flow_to_firm_ttm: 129_174_000_000.0,
            tangible_asset_value_ttm: 85_157_000_000.0,
            net_current_asset_value_ttm: -120_477_000_000.0,
        }]
    );
}

#[test]
fn every_documented_field_is_required_and_non_nullable() {
    assert_every_field_required_and_non_null::<KeyMetrics>(HISTORICAL);
    assert_every_field_required_and_non_null::<KeyMetricsTtm>(TTM);
}

fn assert_every_field_required_and_non_null<T: DeserializeOwned>(fixture: &[u8]) {
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

        let mut null = source.clone();
        null[0][&key] = serde_json::Value::Null;
        assert!(
            serde_json::from_value::<Vec<T>>(null).is_err(),
            "accepted null required field {key}"
        );
    }
}

#[test]
fn historical_and_ttm_shapes_are_distinct_bare_arrays() {
    assert!(
        serde_json::from_slice::<Vec<KeyMetrics>>(b"[]")
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_slice::<Vec<KeyMetricsTtm>>(b"[]")
            .unwrap()
            .is_empty()
    );
    assert_multiple::<KeyMetrics>(HISTORICAL);
    assert_multiple::<KeyMetricsTtm>(TTM);

    assert!(serde_json::from_slice::<Vec<KeyMetrics>>(TTM).is_err());
    assert!(serde_json::from_slice::<Vec<KeyMetricsTtm>>(HISTORICAL).is_err());
    assert!(
        serde_json::from_value::<Vec<KeyMetrics>>(serde_json::json!({ "keyMetrics": [] })).is_err()
    );
}

fn assert_multiple<T: DeserializeOwned>(fixture: &[u8]) {
    let mut value: serde_json::Value = serde_json::from_slice(fixture).unwrap();
    let duplicate = value[0].clone();
    value.as_array_mut().unwrap().push(duplicate);
    assert_eq!(serde_json::from_value::<Vec<T>>(value).unwrap().len(), 2);
}

#[test]
fn acronym_ttm_and_provider_typo_keys_round_trip_exactly() {
    let historical: Vec<KeyMetrics> = serde_json::from_slice(HISTORICAL).unwrap();
    let historical = serde_json::to_value(&historical).unwrap();
    assert!(historical[0].get("evToEBITDA").is_some());
    assert!(historical[0].get("netDebtToEBITDA").is_some());
    assert!(
        historical[0]
            .get("researchAndDevelopementToRevenue")
            .is_some()
    );
    assert!(historical[0].get("evToEbitda").is_none());
    assert!(
        historical[0]
            .get("researchAndDevelopmentToRevenue")
            .is_none()
    );

    let ttm: Vec<KeyMetricsTtm> = serde_json::from_slice(TTM).unwrap();
    let ttm = serde_json::to_value(&ttm).unwrap();
    assert!(ttm[0].get("marketCap").is_some());
    assert!(ttm[0].get("marketCapTTM").is_none());
    assert!(ttm[0].get("enterpriseValueTTM").is_some());
    assert!(ttm[0].get("evToEBITDATTM").is_some());
    assert!(ttm[0].get("netDebtToEBITDATTM").is_some());
    assert!(ttm[0].get("researchAndDevelopementToRevenueTTM").is_some());
}

#[test]
fn ratios_accept_documented_integer_and_fractional_json_numbers() {
    let historical: Vec<KeyMetrics> = serde_json::from_slice(HISTORICAL).unwrap();
    assert_eq!(historical[0].interest_burden, 1.0);
    assert_eq!(historical[0].intangibles_to_total_assets, 0.0);
    assert_eq!(historical[0].current_ratio, 0.8932929222186667);

    let ttm: Vec<KeyMetricsTtm> = serde_json::from_slice(TTM).unwrap();
    assert_eq!(ttm[0].interest_burden_ttm, 1.0);
    assert_eq!(ttm[0].current_ratio_ttm, 1.07035746912159);
}

#[test]
fn signed_metric_amounts_preserve_large_negative_values() {
    let mut historical: serde_json::Value = serde_json::from_slice(HISTORICAL).unwrap();
    historical[0]["netCurrentAssetValue"] = serde_json::json!(-9_000_000_000_000_000_000_i64);
    let rows: Vec<KeyMetrics> = serde_json::from_value(historical).unwrap();
    assert_eq!(
        rows[0].net_current_asset_value,
        -9_000_000_000_000_000_000.0
    );

    let mut ttm: serde_json::Value = serde_json::from_slice(TTM).unwrap();
    ttm[0]["workingCapitalTTM"] = serde_json::json!(-9_000_000_000_000_000_000_i64);
    let rows: Vec<KeyMetricsTtm> = serde_json::from_value(ttm).unwrap();
    assert_eq!(rows[0].working_capital_ttm, -9_000_000_000_000_000_000.0);
}
