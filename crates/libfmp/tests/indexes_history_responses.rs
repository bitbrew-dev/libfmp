use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

use libfmp::responses::chart::{StockChartFullBar, StockChartIntradayBar, StockChartLightBar};

const LIGHT: &[u8] = include_bytes!("fixtures/indexes_chart_light.json");
const FULL: &[u8] = include_bytes!("fixtures/indexes_chart_full.json");
const ONE_MINUTE: &[u8] = include_bytes!("fixtures/indexes_chart_one_minute.json");
const FIVE_MINUTES: &[u8] = include_bytes!("fixtures/indexes_chart_five_minutes.json");
const ONE_HOUR: &[u8] = include_bytes!("fixtures/indexes_chart_one_hour.json");

#[test]
fn all_five_documented_index_history_fixtures_match_the_shared_chart_contracts_exactly() {
    assert_exact::<StockChartLightBar>(LIGHT);
    assert_exact::<StockChartFullBar>(FULL);
    for fixture in [ONE_MINUTE, FIVE_MINUTES, ONE_HOUR] {
        assert_exact::<StockChartIntradayBar>(fixture);
    }

    let light: Vec<StockChartLightBar> = serde_json::from_slice(LIGHT).unwrap();
    let full: Vec<StockChartFullBar> = serde_json::from_slice(FULL).unwrap();
    let one_minute: Vec<StockChartIntradayBar> = serde_json::from_slice(ONE_MINUTE).unwrap();
    let five_minutes: Vec<StockChartIntradayBar> = serde_json::from_slice(FIVE_MINUTES).unwrap();
    let one_hour: Vec<StockChartIntradayBar> = serde_json::from_slice(ONE_HOUR).unwrap();

    assert_eq!(light[0].symbol.as_str(), "^VIX");
    assert_eq!(light[0].price, 17.9);
    assert_eq!(full[0].symbol.as_str(), "^VIX");
    assert_eq!(full[0].change_percent, -8.48671);
    assert_eq!(one_minute[0].date.to_string(), "2026-07-30 13:17:00");
    assert_eq!(five_minutes[0].date.to_string(), "2026-07-30 13:15:00");
    assert_eq!(one_hour[0].date.to_string(), "2026-07-30 12:30:00");
}

fn assert_exact<T>(fixture: &[u8])
where
    T: DeserializeOwned + Serialize,
{
    let source: Value = serde_json::from_slice(fixture).unwrap();
    let rows: Vec<T> = serde_json::from_slice(fixture).unwrap();
    assert_eq!(serde_json::to_value(rows).unwrap(), source);
}

#[test]
fn index_history_contracts_preserve_bare_array_roots() {
    assert!(
        serde_json::from_str::<Vec<StockChartLightBar>>("[]")
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_str::<Vec<StockChartFullBar>>("[]")
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_str::<Vec<StockChartIntradayBar>>("[]")
            .unwrap()
            .is_empty()
    );
}
