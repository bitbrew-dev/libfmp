#[macro_use]
#[path = "support/assert_row.rs"]
mod assert_row;

use std::str::FromStr;

use libfmp::{
    responses::institutional_ownership::{HolderIndustryBreakdown, HolderPerformanceSummary},
    types::{Cik, Date},
};
use serde::de::DeserializeOwned;

const PERFORMANCE: &[u8] = include_bytes!("fixtures/holder_performance_summary.json");
const INDUSTRY: &[u8] = include_bytes!("fixtures/holder_industry_breakdown.json");

#[test]
fn exact_performance_fixture_decodes_all_33_fields_and_provider_casing() {
    let source: serde_json::Value = serde_json::from_slice(PERFORMANCE).unwrap();
    assert_eq!(source[0].as_object().unwrap().len(), 33);
    let rows: Vec<HolderPerformanceSummary> = serde_json::from_value(source.clone()).unwrap();
    assert_rows!(
        rows,
        [HolderPerformanceSummary {
            date: Date::from_str("2026-03-31").unwrap(),
            cik: Cik::new("0001067983").unwrap(),
            investor_name: "BERKSHIRE HATHAWAY INC".to_owned(),
            portfolio_size: 29,
            securities_added: 3,
            securities_removed: 16,
            market_value: 263_095_703_570.0,
            previous_market_value: 274_160_086_701.0,
            change_in_market_value: -11_064_383_131.0,
            change_in_market_value_percentage: -4.0357,
            average_holding_period: 19.0,
            average_holding_period_top10: 32.0,
            average_holding_period_top20: 25.0,
            turnover: 0.6552,
            turnover_alternate_sell: 9.1702,
            turnover_alternate_buy: 5.8198,
            performance: -2_243_708_176.0,
            performance_percentage: -0.8184,
            last_performance: 12_155_036_983.0,
            change_in_performance: -14_398_745_159.0,
            performance_1_year: 28_972_527_543.0,
            performance_percentage_1_year: 11.3877,
            performance_3_year: 118_145_912_143.0,
            performance_percentage_3_year: 45.9009,
            performance_5_year: 146_867_544_096.0,
            performance_percentage_5_year: 63.1842,
            performance_since_inception: 267_584_180_516.0,
            performance_since_inception_percentage: 203.9112,
            performance_relative_to_sp500_percentage: 3.8118,
            performance_1_year_relative_to_sp500_percentage: -4.9473,
            performance_3_year_relative_to_sp500_percentage: -12.9708,
            performance_5_year_relative_to_sp500_percentage: -1.1428,
            performance_since_inception_relative_to_sp500_percentage: -114.003,
        }]
    );
    assert_eq!(rows[0].cik.as_str(), "0001067983");
    assert_eq!(serde_json::to_value(rows).unwrap(), source);
}

#[test]
fn exact_industry_fixture_decodes_all_12_fields() {
    let source: serde_json::Value = serde_json::from_slice(INDUSTRY).unwrap();
    assert_eq!(source[0].as_object().unwrap().len(), 12);
    let rows: Vec<HolderIndustryBreakdown> = serde_json::from_value(source.clone()).unwrap();
    assert_rows!(
        rows,
        [HolderIndustryBreakdown {
            date: Date::from_str("2023-09-30").unwrap(),
            cik: Cik::new("0001067983").unwrap(),
            investor_name: "BERKSHIRE HATHAWAY INC".to_owned(),
            industry_title: "ELECTRONIC COMPUTERS".to_owned(),
            weight: 49.7704,
            last_weight: 51.0035,
            change_in_weight: -1.2332,
            change_in_weight_percentage: -2.4178,
            performance: -20_838_154_294.0,
            performance_percentage: -178.2938,
            last_performance: 26_615_340_304.0,
            change_in_performance: -47_453_494_598.0,
        }]
    );
    assert_eq!(serde_json::to_value(rows).unwrap(), source);
}

#[test]
fn counts_preserve_the_full_u64_domain_and_values_and_periods_keep_fractions() {
    for field in ["portfolioSize", "securitiesAdded", "securitiesRemoved"] {
        let mut source: serde_json::Value = serde_json::from_slice(PERFORMANCE).unwrap();
        source[0][field] = serde_json::json!(u64::MAX);
        let rows: Vec<HolderPerformanceSummary> = serde_json::from_value(source).unwrap();
        let encoded = serde_json::to_value(&rows[0]).unwrap();
        assert_eq!(encoded[field], serde_json::json!(u64::MAX));

        for invalid in [serde_json::json!(-1), serde_json::json!(1.5)] {
            let mut source: serde_json::Value = serde_json::from_slice(PERFORMANCE).unwrap();
            source[0][field] = invalid;
            assert!(serde_json::from_value::<Vec<HolderPerformanceSummary>>(source).is_err());
        }
    }

    let fields = [
        "marketValue",
        "previousMarketValue",
        "averageHoldingPeriod",
        "averageHoldingPeriodTop10",
        "averageHoldingPeriodTop20",
    ];
    for field in fields {
        for value in [
            serde_json::json!(19.5),
            serde_json::json!(4_294_967_296_u64),
        ] {
            let mut source: serde_json::Value = serde_json::from_slice(PERFORMANCE).unwrap();
            source[0][field] = value.clone();
            let rows: Vec<HolderPerformanceSummary> = serde_json::from_value(source).unwrap();
            assert_eq!(serde_json::to_value(&rows[0]).unwrap()[field], value);
        }
    }
}

#[test]
fn all_performance_and_change_amounts_preserve_large_signed_values() {
    let summary_fields = [
        "changeInMarketValue",
        "performance",
        "lastPerformance",
        "changeInPerformance",
        "performance1year",
        "performance3year",
        "performance5year",
        "performanceSinceInception",
    ];
    for (index, field) in summary_fields.into_iter().enumerate() {
        let mut source: serde_json::Value = serde_json::from_slice(PERFORMANCE).unwrap();
        source[0][field] = if index % 2 == 0 {
            serde_json::json!(i64::MIN)
        } else {
            serde_json::json!(12_155_036_983.25)
        };
        let rows: Vec<HolderPerformanceSummary> = serde_json::from_value(source.clone()).unwrap();
        assert_eq!(
            serde_json::to_value(&rows[0]).unwrap()[field],
            source[0][field]
        );
    }

    for field in ["performance", "lastPerformance", "changeInPerformance"] {
        let mut source: serde_json::Value = serde_json::from_slice(INDUSTRY).unwrap();
        source[0][field] = serde_json::json!(i64::MIN);
        let rows: Vec<HolderIndustryBreakdown> = serde_json::from_value(source).unwrap();
        assert_eq!(serde_json::to_value(&rows[0]).unwrap()[field], i64::MIN);
    }
}

#[test]
fn turnover_percentage_relative_and_weight_values_are_raw_f64() {
    let summary_fields = [
        "changeInMarketValuePercentage",
        "turnover",
        "turnoverAlternateSell",
        "turnoverAlternateBuy",
        "performancePercentage",
        "performancePercentage1year",
        "performancePercentage3year",
        "performancePercentage5year",
        "performanceSinceInceptionPercentage",
        "performanceRelativeToSP500Percentage",
        "performance1yearRelativeToSP500Percentage",
        "performance3yearRelativeToSP500Percentage",
        "performance5yearRelativeToSP500Percentage",
        "performanceSinceInceptionRelativeToSP500Percentage",
    ];
    let mut summary: serde_json::Value = serde_json::from_slice(PERFORMANCE).unwrap();
    for (index, field) in summary_fields.into_iter().enumerate() {
        summary[0][field] = if index % 2 == 0 {
            serde_json::json!(-250.75)
        } else {
            serde_json::json!(500.125)
        };
    }
    let rows: Vec<HolderPerformanceSummary> = serde_json::from_value(summary).unwrap();
    assert_eq!(rows[0].turnover, 500.125);
    assert_eq!(rows[0].performance_relative_to_sp500_percentage, 500.125);
    assert_eq!(
        rows[0].performance_1_year_relative_to_sp500_percentage,
        -250.75
    );

    let mut industry: serde_json::Value = serde_json::from_slice(INDUSTRY).unwrap();
    industry[0]["weight"] = serde_json::json!(250.5);
    industry[0]["lastWeight"] = serde_json::json!(-1.25);
    industry[0]["changeInWeight"] = serde_json::json!(-251.75);
    industry[0]["changeInWeightPercentage"] = serde_json::json!(999.125);
    industry[0]["performancePercentage"] = serde_json::json!(-500.5);
    let rows: Vec<HolderIndustryBreakdown> = serde_json::from_value(industry).unwrap();
    assert_eq!(rows[0].weight, 250.5);
    assert_eq!(rows[0].last_weight, -1.25);
    assert_eq!(rows[0].performance_percentage, -500.5);
}

#[test]
fn every_documented_field_is_required_non_null_and_unknowns_are_accepted() {
    assert_contract::<HolderPerformanceSummary>(PERFORMANCE);
    assert_contract::<HolderIndustryBreakdown>(INDUSTRY);

    let mut performance_timestamp: serde_json::Value = serde_json::from_slice(PERFORMANCE).unwrap();
    performance_timestamp[0]["date"] = serde_json::json!("2026-03-31 00:00:00");
    assert!(
        serde_json::from_value::<Vec<HolderPerformanceSummary>>(performance_timestamp).is_err()
    );

    let mut industry_timestamp: serde_json::Value = serde_json::from_slice(INDUSTRY).unwrap();
    industry_timestamp[0]["date"] = serde_json::json!("2023-09-30 00:00:00");
    assert!(serde_json::from_value::<Vec<HolderIndustryBreakdown>>(industry_timestamp).is_err());
}

#[test]
fn both_contracts_are_bare_vecs_preserving_empty_and_multiple_rows() {
    assert!(
        serde_json::from_slice::<Vec<HolderPerformanceSummary>>(b"[]")
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_slice::<Vec<HolderIndustryBreakdown>>(b"[]")
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_value::<Vec<HolderPerformanceSummary>>(serde_json::json!({
            "summaries": []
        }))
        .is_err()
    );
    assert_multiple::<HolderPerformanceSummary>(PERFORMANCE);
    assert_multiple::<HolderIndustryBreakdown>(INDUSTRY);
}

fn assert_contract<T: DeserializeOwned>(fixture: &[u8]) {
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
            "accepted missing {key}"
        );
        let mut null = source.clone();
        null[0][&key] = serde_json::Value::Null;
        assert!(
            serde_json::from_value::<Vec<T>>(null).is_err(),
            "accepted null {key}"
        );
    }
    let mut forward = source;
    forward[0]["futureProviderField"] = serde_json::json!({ "nested": [1, true, null] });
    assert_eq!(serde_json::from_value::<Vec<T>>(forward).unwrap().len(), 1);
}

fn assert_multiple<T: DeserializeOwned>(fixture: &[u8]) {
    let mut value: serde_json::Value = serde_json::from_slice(fixture).unwrap();
    let duplicate = value[0].clone();
    value.as_array_mut().unwrap().push(duplicate);
    assert_eq!(serde_json::from_value::<Vec<T>>(value).unwrap().len(), 2);
}
