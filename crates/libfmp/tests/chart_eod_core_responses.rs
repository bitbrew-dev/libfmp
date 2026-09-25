#[macro_use]
#[path = "support/assert_row.rs"]
mod assert_row;

use std::str::FromStr;

use libfmp::{
    responses::chart::{StockChartFullBar, StockChartLightBar},
    types::{Date, Ticker},
};

const LIGHT: &[u8] = include_bytes!("fixtures/stock_chart_light.json");
const FULL: &[u8] = include_bytes!("fixtures/stock_chart_full.json");

#[test]
fn endpoint_source_fixtures_decode_every_exact_required_field() {
    let light_value: serde_json::Value = serde_json::from_slice(LIGHT).unwrap();
    let full_value: serde_json::Value = serde_json::from_slice(FULL).unwrap();
    assert_eq!(light_value[0].as_object().unwrap().len(), 4);
    assert_eq!(full_value[0].as_object().unwrap().len(), 10);

    let light: Vec<StockChartLightBar> = serde_json::from_value(light_value.clone()).unwrap();
    assert_rows!(
        light,
        [StockChartLightBar {
            symbol: Ticker::new("AAPL").unwrap(),
            date: Date::from_str("2026-07-30").unwrap(),
            price: 332.39,
            volume: 29_207_295.0,
        }]
    );
    assert_eq!(serde_json::to_value(light).unwrap(), light_value);

    let full: Vec<StockChartFullBar> = serde_json::from_value(full_value.clone()).unwrap();
    assert_rows!(
        full,
        [StockChartFullBar {
            symbol: Ticker::new("AAPL").unwrap(),
            date: Date::from_str("2026-07-30").unwrap(),
            open: 333.13,
            high: 334.48,
            low: 329.59,
            close: 332.39,
            volume: 29_207_295.0,
            change: -0.74,
            change_percent: -0.2221355,
            vwap: 332.15,
        }]
    );
    assert_eq!(serde_json::to_value(full).unwrap(), full_value);
}

#[test]
fn both_endpoint_contracts_preserve_empty_multiple_and_forward_compatible_arrays() {
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

    let mut light: serde_json::Value = serde_json::from_slice(LIGHT).unwrap();
    light[0]["futureProviderField"] = serde_json::json!({ "nested": true });
    let duplicate = light[0].clone();
    light.as_array_mut().unwrap().push(duplicate);
    assert_eq!(
        serde_json::from_value::<Vec<StockChartLightBar>>(light)
            .unwrap()
            .len(),
        2
    );

    let mut full: serde_json::Value = serde_json::from_slice(FULL).unwrap();
    full[0]["futureProviderField"] = serde_json::json!([1, null]);
    let duplicate = full[0].clone();
    full.as_array_mut().unwrap().push(duplicate);
    assert_eq!(
        serde_json::from_value::<Vec<StockChartFullBar>>(full)
            .unwrap()
            .len(),
        2
    );

    let wrapped = serde_json::json!({ "historical": [] });
    assert!(serde_json::from_value::<Vec<StockChartFullBar>>(wrapped).is_err());
}
