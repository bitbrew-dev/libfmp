#[path = "support/bulk_csv.rs"]
mod bulk_csv;
#[allow(dead_code)] // The CSV helpers use only the fixture executor.
mod support;

use serde_json::{Map, Value};

use libfmp::{
    endpoints::bulk::{
        bulk_earnings_surprises, bulk_financial_ratios_ttm, bulk_key_metrics_ttm, bulk_stock_peers,
    },
    error::DecodeErrorKind,
    query::Year,
};

use bulk_csv::{
    assert_empty_bodies, assert_required_members, assert_round_trip, decode, with_cells,
};

const KEY_METRICS: &[u8] = include_bytes!("fixtures/bulk_key_metrics_ttm.csv");
const RATIOS: &[u8] = include_bytes!("fixtures/bulk_financial_ratios_ttm.csv");
const PEERS: &[u8] = include_bytes!("fixtures/bulk_stock_peers.csv");
const SURPRISES: &[u8] = include_bytes!("fixtures/bulk_earnings_surprises.csv");

#[test]
fn live_csv_fixtures_round_trip_every_cell_with_exact_field_counts() {
    assert_round_trip(&bulk_key_metrics_ttm(), KEY_METRICS, 43);
    assert_round_trip(&bulk_financial_ratios_ttm(), RATIOS, 60);
    assert_round_trip(&bulk_stock_peers(), PEERS, 2);
    assert_round_trip(&bulk_earnings_surprises(Year(2024).into()), SURPRISES, 5);
}

#[test]
fn every_column_is_required_and_only_identity_members_reject_an_empty_cell() {
    assert_required_members(&bulk_key_metrics_ttm(), KEY_METRICS, &["symbol"]);
    assert_required_members(&bulk_financial_ratios_ttm(), RATIOS, &["symbol"]);
    assert_required_members(&bulk_stock_peers(), PEERS, &["symbol"]);
    assert_required_members(
        &bulk_earnings_surprises(Year(2024).into()),
        SURPRISES,
        &["symbol", "date", "lastUpdated"],
    );
}

#[test]
fn empty_and_header_only_bodies_decode_to_no_rows() {
    assert_empty_bodies(&bulk_key_metrics_ttm(), KEY_METRICS);
    assert_empty_bodies(&bulk_financial_ratios_ttm(), RATIOS);
    assert_empty_bodies(&bulk_stock_peers(), PEERS);
    assert_empty_bodies(&bulk_earnings_surprises(Year(2024).into()), SURPRISES);
}

#[test]
fn numeric_cells_keep_their_exact_text_beyond_u64_and_in_exponent_form() {
    let beyond_u64 = "18446744073709551616";
    let high_precision = "-12345678901234567890123456789.123456789012345678901234567890";
    let body = with_cells(
        KEY_METRICS,
        &[
            ("marketCap", beyond_u64),
            ("freeCashFlowToFirmTTM", high_precision),
            ("evToSalesTTM", "6.9148336e-9"),
        ],
    );
    let decoded = decode(&bulk_key_metrics_ttm(), body).unwrap().remove(0);

    assert_eq!(decoded.market_cap.unwrap().as_str(), beyond_u64);
    assert_eq!(
        decoded.free_cash_flow_to_firm_ttm.unwrap().as_str(),
        high_precision
    );
    assert_eq!(decoded.ev_to_sales_ttm.unwrap().as_str(), "6.9148336e-9");
}

#[test]
fn all_ttm_keys_and_provider_hazards_keep_exact_wire_spelling() {
    let metrics = first_row(&decode(&bulk_key_metrics_ttm(), KEY_METRICS).unwrap());
    let ratios = first_row(&decode(&bulk_financial_ratios_ttm(), RATIOS).unwrap());

    assert_eq!(
        metrics.keys().filter(|key| key.ends_with("TTM")).count(),
        41
    );
    assert_eq!(ratios.keys().filter(|key| key.ends_with("TTM")).count(), 59);
    assert!(metrics.keys().all(|key| !key.contains("Ttm")));
    assert!(ratios.keys().all(|key| !key.contains("Ttm")));
    for key in [
        "evToEBITDATTM",
        "netDebtToEBITDATTM",
        "researchAndDevelopementToRevenueTTM",
    ] {
        assert!(metrics.contains_key(key), "missing exact wire key {key}");
    }
    assert!(ratios.contains_key("netIncomePerEBTTTM"));
    assert!(!metrics.contains_key("researchAndDevelopmentToRevenueTTM"));
    assert!(!ratios.contains_key("priceToEarningsDilutedRatioTTM"));
    assert!(!ratios.contains_key("priceToEarningsDilutedGrowthRatioTTM"));
}

#[test]
fn peers_remain_one_quoted_string_and_earnings_dates_are_typed() {
    let peers = decode(&bulk_stock_peers(), PEERS).unwrap().remove(0);
    assert_eq!(peers.symbol.as_str(), "000001.SZ");
    assert_eq!(
        peers.peers,
        "3698.HK,600000.SS,600015.SS,600016.SS,600036.SS,601166.SS,601658.SS"
    );

    let surprises = decode(&bulk_earnings_surprises(Year(2024).into()), SURPRISES).unwrap();
    assert_eq!(surprises[0].symbol.as_str(), "AUTO.OL");
    assert_eq!(surprises[0].date.to_string(), "2024-12-31");
    assert_eq!(surprises[0].last_updated.to_string(), "2025-10-07");
    let timestamped = with_cells(SURPRISES, &[("date", "2024-12-31T00:00:00")]);
    let error = decode(&bulk_earnings_surprises(Year(2024).into()), timestamped).unwrap_err();
    assert_eq!(error.decode_path(), Some("[0].date"));
    assert_eq!(error.decode_kind(), Some(DecodeErrorKind::InvalidValue));
}

fn first_row<R: serde::Serialize>(rows: &[R]) -> Map<String, Value> {
    match serde_json::to_value(&rows[0]).unwrap() {
        Value::Object(members) => members,
        other => panic!("a row re-encodes as an object, not {other}"),
    }
}

#[test]
fn live_empty_cells_decode_as_absent_metrics_and_empty_text() {
    let metrics = decode(&bulk_key_metrics_ttm(), KEY_METRICS)
        .unwrap()
        .remove(2);
    assert_eq!(metrics.symbol.as_str(), "ADAMO");
    assert_eq!(metrics.enterprise_value_ttm, None);
    assert_eq!(metrics.market_cap.unwrap().as_str(), "711942736");

    let ratios = decode(&bulk_financial_ratios_ttm(), RATIOS)
        .unwrap()
        .remove(2);
    assert_eq!(ratios.price_to_earnings_ratio_ttm, None);
    assert_eq!(ratios.gross_profit_margin_ttm.unwrap().as_str(), "0");

    let surprise = decode(&bulk_earnings_surprises(Year(2024).into()), SURPRISES)
        .unwrap()
        .remove(2);
    assert_eq!(surprise.eps_estimated, None);
    assert_eq!(surprise.eps_actual.unwrap().as_str(), "0.00084");

    let peers = decode(&bulk_stock_peers(), PEERS).unwrap().remove(2);
    assert_eq!(peers.peers, "");
}
