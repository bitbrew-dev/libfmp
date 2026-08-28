use std::str::FromStr;

use libfmp::{
    responses::chart::{
        StockChartAdjustedBar, StockChartFullBar, StockChartIntradayBar, StockChartLightBar,
    },
    types::{ApiDateTime, Date, Ticker},
};
use serde::de::DeserializeOwned;

const LIGHT: &[u8] = include_bytes!("fixtures/stock_chart_light.json");
const FULL: &[u8] = include_bytes!("fixtures/stock_chart_full.json");
const ADJUSTED: &[u8] = include_bytes!("fixtures/stock_chart_adjusted.json");
const INTRADAY: &[u8] = include_bytes!("fixtures/stock_chart_intraday.json");

#[test]
fn exact_source_rows_decode_all_required_fields() {
    let light: Vec<StockChartLightBar> = serde_json::from_slice(LIGHT).unwrap();
    assert_eq!(
        light,
        [StockChartLightBar {
            symbol: Ticker::new("AAPL").unwrap(),
            date: Date::from_str("2026-07-30").unwrap(),
            price: 332.39,
            volume: 29_207_295,
        }]
    );

    let full: Vec<StockChartFullBar> = serde_json::from_slice(FULL).unwrap();
    assert_eq!(
        full,
        [StockChartFullBar {
            symbol: Ticker::new("AAPL").unwrap(),
            date: Date::from_str("2026-07-30").unwrap(),
            open: 333.13,
            high: 334.48,
            low: 329.59,
            close: 332.39,
            volume: 29_207_295,
            change: -0.74,
            change_percent: -0.2221355,
            vwap: 332.15,
        }]
    );

    let adjusted: Vec<StockChartAdjustedBar> = serde_json::from_slice(ADJUSTED).unwrap();
    assert_eq!(
        adjusted,
        [StockChartAdjustedBar {
            symbol: Ticker::new("AAPL").unwrap(),
            date: Date::from_str("2026-07-30").unwrap(),
            adj_open: 333.13,
            adj_high: 334.48,
            adj_low: 329.59,
            adj_close: 332.39,
            volume: 29_207_295,
        }]
    );

    let intraday: Vec<StockChartIntradayBar> = serde_json::from_slice(INTRADAY).unwrap();
    assert_eq!(
        intraday,
        [StockChartIntradayBar {
            date: ApiDateTime::from_str("2026-07-30 13:16:00").unwrap(),
            open: 332.4,
            low: 332.27499,
            high: 332.48,
            close: 332.47,
            volume: 67_660,
        }]
    );
}

#[test]
fn exact_source_shapes_have_four_ten_seven_and_six_fields() {
    for (fixture, expected) in [(LIGHT, 4), (FULL, 10), (ADJUSTED, 7), (INTRADAY, 6)] {
        let value: serde_json::Value = serde_json::from_slice(fixture).unwrap();
        assert_eq!(value[0].as_object().unwrap().len(), expected);
    }
}

#[test]
fn adjusted_wire_names_preserve_the_exact_adj_open_family() {
    let rows: Vec<StockChartAdjustedBar> = serde_json::from_slice(ADJUSTED).unwrap();
    let encoded = serde_json::to_value(rows).unwrap();

    for key in ["adjOpen", "adjHigh", "adjLow", "adjClose"] {
        assert!(encoded[0].get(key).is_some(), "missing {key}");
    }
    for wrong in ["adjustedOpen", "adj_open", "adjhigh", "adjclose"] {
        assert!(encoded[0].get(wrong).is_none(), "emitted {wrong}");
    }
}

#[test]
fn intraday_datetime_is_naive_and_row_has_no_symbol_or_timezone_contract() {
    let rows: Vec<StockChartIntradayBar> = serde_json::from_slice(INTRADAY).unwrap();
    assert_eq!(rows[0].date.to_string(), "2026-07-30 13:16:00");
    let encoded = serde_json::to_value(rows).unwrap();
    assert!(encoded[0].get("symbol").is_none());
    assert!(encoded[0].get("timezone").is_none());

    for zoned in ["2026-07-30T13:16:00Z", "2026-07-30 13:16:00-04:00"] {
        let mut value: serde_json::Value = serde_json::from_slice(INTRADAY).unwrap();
        value[0]["date"] = serde_json::json!(zoned);
        assert!(serde_json::from_value::<Vec<StockChartIntradayBar>>(value).is_err());
    }
}

#[test]
fn every_documented_field_is_required_and_non_null() {
    assert_required::<StockChartLightBar>(LIGHT);
    assert_required::<StockChartFullBar>(FULL);
    assert_required::<StockChartAdjustedBar>(ADJUSTED);
    assert_required::<StockChartIntradayBar>(INTRADAY);
}

fn assert_required<T: DeserializeOwned>(fixture: &[u8]) {
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
}

#[test]
fn all_chart_rows_accept_unknown_fields_and_preserve_bare_array_shapes() {
    assert_unknown::<StockChartLightBar>(LIGHT);
    assert_unknown::<StockChartFullBar>(FULL);
    assert_unknown::<StockChartAdjustedBar>(ADJUSTED);
    assert_unknown::<StockChartIntradayBar>(INTRADAY);

    assert!(
        serde_json::from_slice::<Vec<StockChartLightBar>>(b"[]")
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_slice::<Vec<StockChartFullBar>>(b"[]")
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_slice::<Vec<StockChartAdjustedBar>>(b"[]")
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_slice::<Vec<StockChartIntradayBar>>(b"[]")
            .unwrap()
            .is_empty()
    );
}

fn assert_unknown<T: DeserializeOwned>(fixture: &[u8]) {
    let mut value: serde_json::Value = serde_json::from_slice(fixture).unwrap();
    value[0]["futureProviderField"] = serde_json::json!({ "nested": [1, true, null] });
    let duplicate = value[0].clone();
    value.as_array_mut().unwrap().push(duplicate);
    assert_eq!(serde_json::from_value::<Vec<T>>(value).unwrap().len(), 2);
}
