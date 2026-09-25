use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Map, Value, json};

use libfmp::responses::bulk::{
    BulkEarningsSurprise, BulkFinancialRatiosTtm, BulkKeyMetricsTtm, BulkStockPeer,
};

const KEY_METRICS: &[u8] = include_bytes!("fixtures/bulk_key_metrics_ttm.json");
const RATIOS: &[u8] = include_bytes!("fixtures/bulk_financial_ratios_ttm.json");
const PEERS: &[u8] = include_bytes!("fixtures/bulk_stock_peers.json");
const SURPRISES: &[u8] = include_bytes!("fixtures/bulk_earnings_surprises.json");

#[test]
fn exact_outer_md_fixtures_round_trip_with_exact_unique_key_sets() {
    assert_exact::<BulkKeyMetricsTtm>(KEY_METRICS, 43);
    assert_exact::<BulkFinancialRatiosTtm>(RATIOS, 60);
    assert_exact::<BulkStockPeer>(PEERS, 2);
    assert_exact::<BulkEarningsSurprise>(SURPRISES, 5);
}

#[test]
fn every_documented_field_is_required_non_null_and_unknown_fields_are_tolerated() {
    assert_required::<BulkKeyMetricsTtm>(KEY_METRICS);
    assert_required::<BulkFinancialRatiosTtm>(RATIOS);
    assert_required::<BulkStockPeer>(PEERS);
    assert_required::<BulkEarningsSurprise>(SURPRISES);
}

#[test]
fn every_documented_numeric_string_is_lexical_and_rejects_json_numbers() {
    assert_all_numbers_rejected::<BulkKeyMetricsTtm>(KEY_METRICS, &["symbol"]);
    assert_all_numbers_rejected::<BulkFinancialRatiosTtm>(RATIOS, &["symbol"]);
    assert_all_numbers_rejected::<BulkEarningsSurprise>(
        SURPRISES,
        &["symbol", "date", "lastUpdated"],
    );

    let metrics = rows::<BulkKeyMetricsTtm>(KEY_METRICS).remove(0);
    assert_eq!(metrics.market_cap.as_str(), "249171756000");
    assert_eq!(metrics.enterprise_value_ttm.as_str(), "-496959244000");
    assert_eq!(metrics.current_ratio_ttm.as_str(), "0");
    assert_eq!(
        metrics.free_cash_flow_to_firm_ttm.as_str(),
        "-35237570137.11014"
    );
    let ratios = rows::<BulkFinancialRatiosTtm>(RATIOS).remove(0);
    assert_eq!(ratios.enterprise_value_ttm.as_str(), "-496959244000");
    assert_eq!(ratios.receivables_turnover_ttm.as_str(), "0");
    assert_eq!(
        ratios.gross_profit_margin_ttm.as_str(),
        "1.1622776732779352"
    );

    let beyond_u64 = "18446744073709551616";
    let high_precision = "-12345678901234567890123456789.123456789012345678901234567890";
    let mut huge = source_row(KEY_METRICS);
    huge.insert("marketCap".into(), json!(beyond_u64));
    huge.insert("freeCashFlowToFirmTTM".into(), json!(high_precision));
    let decoded: BulkKeyMetricsTtm = serde_json::from_value(Value::Object(huge)).unwrap();
    assert_eq!(decoded.market_cap.as_str(), beyond_u64);
    assert_eq!(decoded.free_cash_flow_to_firm_ttm.as_str(), high_precision);
    let encoded = serde_json::to_value(&decoded).unwrap();
    assert_eq!(encoded["marketCap"], beyond_u64);
    assert_eq!(encoded["freeCashFlowToFirmTTM"], high_precision);
}

#[test]
fn all_ttm_keys_and_provider_hazards_keep_exact_wire_spelling() {
    let metrics = serialized_row::<BulkKeyMetricsTtm>(KEY_METRICS);
    let ratios = serialized_row::<BulkFinancialRatiosTtm>(RATIOS);

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
    assert!(!ratios.contains_key("priceToEarningsRatioDilutedTTM"));
    assert!(!ratios.contains_key("priceToEarningsGrowthRatioDilutedTTM"));
}

#[test]
fn peers_remain_one_scalar_string_and_earnings_dates_are_typed() {
    let peers = rows::<BulkStockPeer>(PEERS).remove(0);
    assert_eq!(peers.symbol.as_str(), "000001.SZ");
    assert_eq!(peers.peers, "600036.SS");
    let mut peer_row = source_row(PEERS);
    peer_row.insert("peers".into(), json!(["600036.SS"]));
    assert!(serde_json::from_value::<BulkStockPeer>(Value::Object(peer_row)).is_err());

    let surprise = rows::<BulkEarningsSurprise>(SURPRISES).remove(0);
    assert_eq!(surprise.symbol.as_str(), "AMKYF");
    assert_eq!(surprise.date.to_string(), "2025-07-09");
    assert_eq!(surprise.last_updated.to_string(), "2025-07-09");
    let mut surprise_row = source_row(SURPRISES);
    surprise_row.insert("date".into(), json!("2025-07-09T00:00:00"));
    assert!(serde_json::from_value::<BulkEarningsSurprise>(Value::Object(surprise_row)).is_err());
}

#[test]
fn all_four_contracts_require_bare_array_roots_and_accept_empty_arrays() {
    assert_array_contract::<BulkKeyMetricsTtm>();
    assert_array_contract::<BulkFinancialRatiosTtm>();
    assert_array_contract::<BulkStockPeer>();
    assert_array_contract::<BulkEarningsSurprise>();
}

fn rows<T: DeserializeOwned>(fixture: &[u8]) -> Vec<T> {
    serde_json::from_slice(fixture).unwrap()
}

fn source_row(fixture: &[u8]) -> Map<String, Value> {
    serde_json::from_slice::<Value>(fixture).unwrap()[0]
        .as_object()
        .unwrap()
        .clone()
}

fn serialized_row<T>(fixture: &[u8]) -> Map<String, Value>
where
    T: DeserializeOwned + Serialize,
{
    serde_json::to_value(rows::<T>(fixture)).unwrap()[0]
        .as_object()
        .unwrap()
        .clone()
}

fn assert_exact<T>(fixture: &[u8], fields: usize)
where
    T: DeserializeOwned + Serialize,
{
    let source: Value = serde_json::from_slice(fixture).unwrap();
    let row = source[0].as_object().unwrap();
    assert_eq!(row.len(), fields);
    assert_eq!(
        row.keys().collect::<std::collections::BTreeSet<_>>().len(),
        fields
    );
    assert_eq!(serde_json::to_value(rows::<T>(fixture)).unwrap(), source);
}

fn assert_required<T>(fixture: &[u8])
where
    T: DeserializeOwned,
{
    let row = source_row(fixture);
    for field in row.keys() {
        let mut missing = row.clone();
        missing.remove(field);
        assert!(
            serde_json::from_value::<T>(Value::Object(missing)).is_err(),
            "missing {field} unexpectedly decoded"
        );

        let mut null = row.clone();
        null.insert(field.clone(), Value::Null);
        assert!(
            serde_json::from_value::<T>(Value::Object(null)).is_err(),
            "null {field} unexpectedly decoded"
        );
    }

    let mut future = row;
    future.insert("futureField".into(), json!({"nested": true}));
    assert!(serde_json::from_value::<T>(Value::Object(future)).is_ok());
}

fn assert_all_numbers_rejected<T>(fixture: &[u8], non_numeric: &[&str])
where
    T: DeserializeOwned,
{
    let row = source_row(fixture);
    for field in row
        .keys()
        .filter(|field| !non_numeric.contains(&field.as_str()))
    {
        let mut numeric = row.clone();
        numeric.insert(field.clone(), json!(123.5));
        assert!(
            serde_json::from_value::<T>(Value::Object(numeric)).is_err(),
            "JSON number unexpectedly decoded for {field}"
        );
    }
}

fn assert_array_contract<T>()
where
    T: DeserializeOwned,
{
    assert!(serde_json::from_slice::<Vec<T>>(b"[]").unwrap().is_empty());
    assert!(serde_json::from_slice::<Vec<T>>(b"{}").is_err());
}
