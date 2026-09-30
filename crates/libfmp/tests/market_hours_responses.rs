use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};

use libfmp::responses::market_hours::{ExchangeHoliday, ExchangeMarketHours};

const EXCHANGE: &[u8] = include_bytes!("fixtures/exchange_market_hours.json");
const HOLIDAYS: &[u8] = include_bytes!("fixtures/holidays_by_exchange.json");
const ALL_EXCHANGES: &[u8] = include_bytes!("fixtures/all_exchange_market_hours.json");

#[test]
fn all_three_outer_fixtures_round_trip_exactly() {
    assert_exact::<ExchangeMarketHours>(EXCHANGE);
    assert_exact::<ExchangeHoliday>(HOLIDAYS);
    assert_exact::<ExchangeMarketHours>(ALL_EXCHANGES);

    let exchange: Vec<ExchangeMarketHours> = serde_json::from_slice(EXCHANGE).unwrap();
    let holidays: Vec<ExchangeHoliday> = serde_json::from_slice(HOLIDAYS).unwrap();
    let all_exchanges: Vec<ExchangeMarketHours> = serde_json::from_slice(ALL_EXCHANGES).unwrap();

    assert_eq!(exchange[0].exchange.as_str(), "NASDAQ");
    assert_eq!(exchange[0].opening_hour, "09:30 AM -04:00");
    assert_eq!(exchange[0].closing_hour, "04:00 PM -04:00");
    assert_eq!(exchange[0].timezone, "America/New_York");
    assert!(exchange[0].is_market_open);
    assert_eq!(holidays[0].date.to_string(), "2026-07-03");
    assert_eq!(holidays[0].adj_open_time, None);
    assert_eq!(holidays[0].adj_close_time, None);
    assert_eq!(all_exchanges[0].exchange.as_str(), "ASX");
    assert_eq!(all_exchanges[0].opening_hour, "10:00 AM +10:00");
    assert_eq!(all_exchanges[0].timezone, "Australia/Sydney");
    assert!(!all_exchanges[0].is_market_open);
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
fn adjusted_times_are_required_but_nullable_and_future_dynamic() {
    let source: Value = serde_json::from_slice(HOLIDAYS).unwrap();

    for field in ["adjOpenTime", "adjCloseTime"] {
        let mut missing = source.clone();
        missing[0].as_object_mut().unwrap().remove(field);
        assert!(serde_json::from_value::<Vec<ExchangeHoliday>>(missing).is_err());
    }

    let mut future = source;
    future[0]["adjOpenTime"] = json!("09:30 AM -04:00");
    future[0]["adjCloseTime"] = json!({
        "local": "01:00 PM",
        "offset": "-04:00",
        "earlyClose": true
    });
    let decoded: Vec<ExchangeHoliday> = serde_json::from_value(future.clone()).unwrap();

    assert_eq!(decoded[0].adj_open_time, Some(json!("09:30 AM -04:00")));
    assert_eq!(
        decoded[0].adj_close_time,
        Some(json!({
            "local": "01:00 PM",
            "offset": "-04:00",
            "earlyClose": true
        }))
    );
    assert_eq!(serde_json::to_value(decoded).unwrap(), future);
}

#[test]
fn is_closed_is_required_but_null_decodes_to_none() {
    let source: Value = serde_json::from_slice(HOLIDAYS).unwrap();

    let mut missing = source.clone();
    missing[0].as_object_mut().unwrap().remove("isClosed");
    assert!(serde_json::from_value::<Vec<ExchangeHoliday>>(missing).is_err());

    let mut null = source;
    null[0]["isClosed"] = Value::Null;
    let decoded: Vec<ExchangeHoliday> = serde_json::from_value(null.clone()).unwrap();
    assert_eq!(decoded[0].is_closed, None);
    assert_eq!(serde_json::to_value(decoded).unwrap(), null);
}

#[test]
fn market_hours_contracts_preserve_bare_array_roots() {
    assert!(
        serde_json::from_str::<Vec<ExchangeMarketHours>>("[]")
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_str::<Vec<ExchangeHoliday>>("[]")
            .unwrap()
            .is_empty()
    );
    assert!(serde_json::from_str::<Vec<ExchangeMarketHours>>("{}").is_err());
    assert!(serde_json::from_str::<Vec<ExchangeHoliday>>("{}").is_err());
}
