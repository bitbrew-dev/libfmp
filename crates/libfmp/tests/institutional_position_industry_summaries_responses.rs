use std::str::FromStr;

use libfmp::{
    responses::institutional_ownership::{
        InstitutionalIndustrySummary, InstitutionalPositionSummary,
    },
    types::{Cik, Date, Ticker},
};
use serde::de::DeserializeOwned;

const POSITIONS: &[u8] = include_bytes!("fixtures/institutional_positions_summary.json");
const INDUSTRY: &[u8] = include_bytes!("fixtures/institutional_industry_summary.json");

#[test]
fn exact_positions_fixture_decodes_all_36_fields_and_provider_casing() {
    let source: serde_json::Value = serde_json::from_slice(POSITIONS).unwrap();
    assert_eq!(source[0].as_object().unwrap().len(), 36);
    let rows: Vec<InstitutionalPositionSummary> = serde_json::from_value(source.clone()).unwrap();
    assert_eq!(
        rows,
        [InstitutionalPositionSummary {
            symbol: Ticker::new("AAPL").unwrap(),
            cik: Cik::new("0000320193").unwrap(),
            date: Date::from_str("2023-09-30").unwrap(),
            investors_holding: 4_863,
            last_investors_holding: 4_805,
            investors_holding_change: 58,
            number_of_13f_shares: 9_139_920_744.0,
            last_number_of_13f_shares: 9_360_939_709.0,
            number_of_13f_shares_change: -221_018_965.0,
            total_invested: 1_575_774_922_899.0,
            last_total_invested: 1_820_827_010_085.0,
            total_invested_change: -245_052_087_186.0,
            ownership_percent: 58.5914,
            last_ownership_percent: 59.6329,
            ownership_percent_change: -1.0415,
            new_positions: 162,
            last_new_positions: 191,
            new_positions_change: -29,
            increased_positions: 1_941,
            last_increased_positions: 1_789,
            increased_positions_change: 152,
            closed_positions: 158,
            last_closed_positions: 122,
            closed_positions_change: 36,
            reduced_positions: 2_408,
            last_reduced_positions: 2_543,
            reduced_positions_change: -135,
            total_calls: 173_627_138.0,
            last_total_calls: 198_895_582.0,
            total_calls_change: -25_268_444.0,
            total_puts: 192_913_290.0,
            last_total_puts: 177_042_062.0,
            total_puts_change: 15_871_228.0,
            put_call_ratio: 1.1111,
            last_put_call_ratio: 0.8901,
            put_call_ratio_change: 22.0952,
        }]
    );
    assert_eq!(serde_json::to_value(rows).unwrap(), source);
}

#[test]
fn exact_industry_fixture_decodes_all_three_fields() {
    let source: serde_json::Value = serde_json::from_slice(INDUSTRY).unwrap();
    assert_eq!(source[0].as_object().unwrap().len(), 3);
    let rows: Vec<InstitutionalIndustrySummary> = serde_json::from_value(source.clone()).unwrap();
    assert_eq!(
        rows,
        [InstitutionalIndustrySummary {
            industry_title: "ABRASIVE, ASBESTOS & MISC NONMETALLIC MINERAL PRODS".to_owned(),
            industry_value: 11_088_059_691.0,
            date: Date::from_str("2023-09-30").unwrap(),
        }]
    );
    assert_eq!(serde_json::to_value(rows).unwrap(), source);
}

#[test]
fn counts_preserve_u64_max_and_shares_and_values_keep_fractions() {
    let counts = [
        "investorsHolding",
        "lastInvestorsHolding",
        "newPositions",
        "lastNewPositions",
        "increasedPositions",
        "lastIncreasedPositions",
        "closedPositions",
        "lastClosedPositions",
        "reducedPositions",
        "lastReducedPositions",
    ];
    for field in counts {
        let mut source: serde_json::Value = serde_json::from_slice(POSITIONS).unwrap();
        source[0][field] = serde_json::json!(u64::MAX);
        let rows: Vec<InstitutionalPositionSummary> =
            serde_json::from_value(source.clone()).unwrap();
        assert_eq!(
            serde_json::to_value(&rows[0]).unwrap()[field],
            serde_json::json!(u64::MAX)
        );

        for invalid in [serde_json::json!(-1), serde_json::json!(1.5)] {
            let mut invalid_source: serde_json::Value = serde_json::from_slice(POSITIONS).unwrap();
            invalid_source[0][field] = invalid;
            assert!(
                serde_json::from_value::<Vec<InstitutionalPositionSummary>>(invalid_source)
                    .is_err()
            );
        }
    }

    let amounts = [
        "numberOf13Fshares",
        "lastNumberOf13Fshares",
        "totalInvested",
        "lastTotalInvested",
        "totalCalls",
        "lastTotalCalls",
        "totalPuts",
        "lastTotalPuts",
    ];
    for field in amounts {
        let mut source: serde_json::Value = serde_json::from_slice(POSITIONS).unwrap();
        source[0][field] = serde_json::json!(173_627_138.5);
        let rows: Vec<InstitutionalPositionSummary> =
            serde_json::from_value(source.clone()).unwrap();
        assert_eq!(
            serde_json::to_value(&rows[0]).unwrap()[field],
            source[0][field]
        );
    }

    let mut widths: serde_json::Value = serde_json::from_slice(POSITIONS).unwrap();
    widths[0]["investorsHolding"] = serde_json::json!(u64::from(u32::MAX) + 1);
    widths[0]["totalInvested"] = serde_json::json!((1_u64 << 53) + 1);
    let rows: Vec<InstitutionalPositionSummary> = serde_json::from_value(widths).unwrap();
    assert_eq!(rows[0].investors_holding, u64::from(u32::MAX) + 1);
    assert_eq!(rows[0].total_invested, 9_007_199_254_740_992.0);

    let mut industry: serde_json::Value = serde_json::from_slice(INDUSTRY).unwrap();
    industry[0]["industryValue"] = serde_json::json!(u64::MAX);
    let rows: Vec<InstitutionalIndustrySummary> = serde_json::from_value(industry).unwrap();
    assert_eq!(rows[0].industry_value, u64::MAX as f64);
}

#[test]
fn every_change_preserves_negative_values_and_count_changes_keep_i64_extremes() {
    let fields = [
        "investorsHoldingChange",
        "numberOf13FsharesChange",
        "totalInvestedChange",
        "newPositionsChange",
        "increasedPositionsChange",
        "closedPositionsChange",
        "reducedPositionsChange",
        "totalCallsChange",
        "totalPutsChange",
    ];
    for (index, field) in fields.into_iter().enumerate() {
        let mut source: serde_json::Value = serde_json::from_slice(POSITIONS).unwrap();
        let amount = matches!(
            field,
            "numberOf13FsharesChange"
                | "totalInvestedChange"
                | "totalCallsChange"
                | "totalPutsChange"
        );
        source[0][field] = if index % 2 == 0 {
            serde_json::json!(i64::MIN)
        } else if amount {
            serde_json::json!(-25_268_444.5)
        } else {
            serde_json::json!(i64::MAX)
        };
        let rows: Vec<InstitutionalPositionSummary> =
            serde_json::from_value(source.clone()).unwrap();
        assert_eq!(
            serde_json::to_value(&rows[0]).unwrap()[field],
            source[0][field]
        );
    }
}

#[test]
fn ownership_and_put_call_values_remain_raw_f64() {
    let fields = [
        "ownershipPercent",
        "lastOwnershipPercent",
        "ownershipPercentChange",
        "putCallRatio",
        "lastPutCallRatio",
        "putCallRatioChange",
    ];
    let mut source: serde_json::Value = serde_json::from_slice(POSITIONS).unwrap();
    for (index, field) in fields.into_iter().enumerate() {
        source[0][field] = if index % 2 == 0 {
            serde_json::json!(-250.75)
        } else {
            serde_json::json!(500.125)
        };
    }
    let rows: Vec<InstitutionalPositionSummary> = serde_json::from_value(source).unwrap();
    assert_eq!(rows[0].ownership_percent, -250.75);
    assert_eq!(rows[0].last_ownership_percent, 500.125);
    assert_eq!(rows[0].put_call_ratio_change, 500.125);
}

#[test]
fn every_documented_field_is_required_non_null_and_unknowns_are_accepted() {
    assert_contract::<InstitutionalPositionSummary>(POSITIONS);
    assert_contract::<InstitutionalIndustrySummary>(INDUSTRY);

    for fixture in [POSITIONS, INDUSTRY] {
        let mut timestamp: serde_json::Value = serde_json::from_slice(fixture).unwrap();
        timestamp[0]["date"] = serde_json::json!("2023-09-30 00:00:00");
        if fixture == POSITIONS {
            assert!(
                serde_json::from_value::<Vec<InstitutionalPositionSummary>>(timestamp).is_err()
            );
        } else {
            assert!(
                serde_json::from_value::<Vec<InstitutionalIndustrySummary>>(timestamp).is_err()
            );
        }
    }
}

#[test]
fn both_contracts_are_bare_vecs_preserving_empty_and_multiple_rows() {
    assert!(
        serde_json::from_slice::<Vec<InstitutionalPositionSummary>>(b"[]")
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_slice::<Vec<InstitutionalIndustrySummary>>(b"[]")
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_value::<Vec<InstitutionalPositionSummary>>(serde_json::json!({
            "summaries": []
        }))
        .is_err()
    );
    assert_multiple::<InstitutionalPositionSummary>(POSITIONS);
    assert_multiple::<InstitutionalIndustrySummary>(INDUSTRY);
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
