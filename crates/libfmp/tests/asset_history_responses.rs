#[path = "support/exact_json.rs"]
mod exact_json;

use libfmp::responses::chart::{StockChartFullBar, StockChartIntradayBar, StockChartLightBar};
use serde::{Serialize, de::DeserializeOwned};

use exact_json::assert_json_wire_equivalent_with_f64_fields;

const CHART_F64_FIELDS: &[&str] = &[
    "price",
    "open",
    "high",
    "low",
    "close",
    "change",
    "changePercent",
    "vwap",
];

const COMMODITY_LIGHT: &[u8] = include_bytes!("fixtures/commodity_chart_light.json");
const COMMODITY_FULL: &[u8] = include_bytes!("fixtures/commodity_chart_full.json");
const COMMODITY_ONE_MINUTE: &[u8] = include_bytes!("fixtures/commodity_chart_one_minute.json");
const COMMODITY_FIVE_MINUTES: &[u8] = include_bytes!("fixtures/commodity_chart_five_minutes.json");
const COMMODITY_ONE_HOUR: &[u8] = include_bytes!("fixtures/commodity_chart_one_hour.json");
const FOREX_LIGHT: &[u8] = include_bytes!("fixtures/forex_chart_light.json");
const FOREX_FULL: &[u8] = include_bytes!("fixtures/forex_chart_full.json");
const FOREX_ONE_MINUTE: &[u8] = include_bytes!("fixtures/forex_chart_one_minute.json");
const FOREX_FIVE_MINUTES: &[u8] = include_bytes!("fixtures/forex_chart_five_minutes.json");
const FOREX_ONE_HOUR: &[u8] = include_bytes!("fixtures/forex_chart_one_hour.json");
const CRYPTO_LIGHT: &[u8] = include_bytes!("fixtures/crypto_chart_light.json");
const CRYPTO_FULL: &[u8] = include_bytes!("fixtures/crypto_chart_full.json");
const CRYPTO_ONE_MINUTE: &[u8] = include_bytes!("fixtures/crypto_chart_one_minute.json");
const CRYPTO_FIVE_MINUTES: &[u8] = include_bytes!("fixtures/crypto_chart_five_minutes.json");
const CRYPTO_ONE_HOUR: &[u8] = include_bytes!("fixtures/crypto_chart_one_hour.json");

#[test]
fn all_fifteen_outer_fixtures_round_trip_through_the_existing_chart_rows_exactly() {
    for fixture in [COMMODITY_LIGHT, FOREX_LIGHT, CRYPTO_LIGHT] {
        assert_exact::<StockChartLightBar>(fixture);
    }
    for fixture in [COMMODITY_FULL, FOREX_FULL, CRYPTO_FULL] {
        assert_exact::<StockChartFullBar>(fixture);
    }
    for fixture in [
        COMMODITY_ONE_MINUTE,
        COMMODITY_FIVE_MINUTES,
        COMMODITY_ONE_HOUR,
        FOREX_ONE_MINUTE,
        FOREX_FIVE_MINUTES,
        FOREX_ONE_HOUR,
        CRYPTO_ONE_MINUTE,
        CRYPTO_FIVE_MINUTES,
        CRYPTO_ONE_HOUR,
    ] {
        assert_exact::<StockChartIntradayBar>(fixture);
    }
}

#[test]
fn asset_history_preserves_timezone_less_times_large_crypto_volume_and_zero_volume() {
    let commodity: Vec<StockChartIntradayBar> =
        serde_json::from_slice(COMMODITY_ONE_MINUTE).unwrap();
    let forex: Vec<StockChartIntradayBar> = serde_json::from_slice(FOREX_ONE_MINUTE).unwrap();
    let crypto_light: Vec<StockChartLightBar> = serde_json::from_slice(CRYPTO_LIGHT).unwrap();
    let crypto_full: Vec<StockChartFullBar> = serde_json::from_slice(CRYPTO_FULL).unwrap();
    let crypto_intraday: Vec<StockChartIntradayBar> =
        serde_json::from_slice(CRYPTO_ONE_MINUTE).unwrap();

    assert_eq!(commodity[0].date.to_string(), "2026-07-30 13:06:00");
    assert_eq!(forex[0].date.to_string(), "2026-07-30 13:17:00");
    assert_eq!(crypto_light[0].volume, 32_030_003_200);
    assert_eq!(crypto_full[0].volume, 32_030_003_200);
    assert_eq!(crypto_intraday[0].volume, 0);
}

fn assert_exact<T>(fixture: &[u8])
where
    T: DeserializeOwned + Serialize,
{
    let source: serde_json::Value = serde_json::from_slice(fixture).unwrap();
    let rows: Vec<T> = serde_json::from_slice(fixture).unwrap();
    assert_json_wire_equivalent_with_f64_fields(
        &serde_json::to_value(rows).unwrap(),
        &source,
        CHART_F64_FIELDS,
    );
}
