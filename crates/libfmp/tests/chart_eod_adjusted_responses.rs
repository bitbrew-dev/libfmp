use std::str::FromStr;

use libfmp::{
    responses::chart::StockChartAdjustedBar,
    types::{Date, Ticker},
};
use serde::de::DeserializeOwned;

const NON_SPLIT: &[u8] = include_bytes!("fixtures/stock_chart_non_split_adjusted.json");
const DIVIDEND: &[u8] = include_bytes!("fixtures/stock_chart_dividend_adjusted.json");

#[test]
fn both_endpoint_source_fixtures_decode_the_exact_shared_seven_field_row() {
    for fixture in [NON_SPLIT, DIVIDEND] {
        let source: serde_json::Value = serde_json::from_slice(fixture).unwrap();
        assert_eq!(source[0].as_object().unwrap().len(), 7);

        let rows: Vec<StockChartAdjustedBar> = serde_json::from_value(source.clone()).unwrap();
        assert_eq!(
            rows,
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
        assert_eq!(serde_json::to_value(rows).unwrap(), source);
    }
}

#[test]
fn adjusted_price_keys_preserve_the_exact_provider_spellings() {
    for fixture in [NON_SPLIT, DIVIDEND] {
        let rows: Vec<StockChartAdjustedBar> = serde_json::from_slice(fixture).unwrap();
        let encoded = serde_json::to_value(rows).unwrap();
        for key in ["adjOpen", "adjHigh", "adjLow", "adjClose"] {
            assert!(encoded[0].get(key).is_some(), "missing {key}");
        }
        for wrong in [
            "open",
            "adjustedOpen",
            "adj_open",
            "adjHighPrice",
            "adjlow",
            "adjclose",
        ] {
            assert!(encoded[0].get(wrong).is_none(), "emitted {wrong}");
        }
    }
}

#[test]
fn every_documented_field_is_required_non_null_and_forward_compatible() {
    assert_required::<StockChartAdjustedBar>(NON_SPLIT);
    assert_required::<StockChartAdjustedBar>(DIVIDEND);

    for fixture in [NON_SPLIT, DIVIDEND] {
        let mut value: serde_json::Value = serde_json::from_slice(fixture).unwrap();
        value[0]["futureProviderField"] = serde_json::json!({ "nested": [true, null] });
        let duplicate = value[0].clone();
        value.as_array_mut().unwrap().push(duplicate);
        assert_eq!(
            serde_json::from_value::<Vec<StockChartAdjustedBar>>(value)
                .unwrap()
                .len(),
            2
        );
    }

    assert!(
        serde_json::from_slice::<Vec<StockChartAdjustedBar>>(b"[]")
            .unwrap()
            .is_empty()
    );
    let wrapped = serde_json::json!({ "historical": [] });
    assert!(serde_json::from_value::<Vec<StockChartAdjustedBar>>(wrapped).is_err());
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
