#[macro_use]
#[path = "support/assert_row.rs"]
mod assert_row;

use std::{collections::BTreeSet, str::FromStr};

use libfmp::{responses::chart::StockChartIntradayBar, types::ApiDateTime};

const ONE_MINUTE: &[u8] = include_bytes!("fixtures/stock_chart_one_minute.json");
const FIVE_MINUTES: &[u8] = include_bytes!("fixtures/stock_chart_five_minutes.json");
const FIFTEEN_MINUTES: &[u8] = include_bytes!("fixtures/stock_chart_fifteen_minutes.json");
const THIRTY_MINUTES: &[u8] = include_bytes!("fixtures/stock_chart_thirty_minutes.json");
const ONE_HOUR: &[u8] = include_bytes!("fixtures/stock_chart_one_hour.json");
const FOUR_HOURS: &[u8] = include_bytes!("fixtures/stock_chart_four_hours.json");

const FIXTURES: [(&str, &[u8]); 6] = [
    ("one-minute", ONE_MINUTE),
    ("five-minute", FIVE_MINUTES),
    ("fifteen-minute", FIFTEEN_MINUTES),
    ("thirty-minute", THIRTY_MINUTES),
    ("one-hour", ONE_HOUR),
    ("four-hour", FOUR_HOURS),
];

#[test]
fn every_intraday_route_decodes_its_exact_documented_row() {
    let expected = [
        (
            "2026-07-30 13:16:00",
            332.4,
            332.27499,
            332.48,
            332.47,
            67_660.0,
        ),
        (
            "2026-07-30 13:15:00",
            332.655,
            332.31989,
            332.755,
            332.31989,
            123_020.0,
        ),
        (
            "2026-07-30 13:15:00",
            332.655,
            332.31989,
            332.755,
            332.31989,
            123_020.0,
        ),
        (
            "2026-07-30 13:00:00",
            331.71,
            331.71,
            332.82999,
            332.31989,
            980_442.0,
        ),
        (
            "2026-07-30 12:30:00",
            332.14,
            331.43,
            332.82999,
            332.31989,
            3_285_503.0,
        ),
        (
            "2026-07-30 09:30:00",
            333.13,
            329.70499,
            334.26,
            332.31989,
            28_439_347.0,
        ),
    ];

    for ((route, fixture), (date, open, low, high, close, volume)) in
        FIXTURES.into_iter().zip(expected)
    {
        let rows: Vec<StockChartIntradayBar> = serde_json::from_slice(fixture).unwrap();
        assert_rows!(
            rows,
            [StockChartIntradayBar {
                date: ApiDateTime::from_str(date).unwrap(),
                open,
                low,
                high,
                close,
                volume,
            }],
            "{route}"
        );

        let source: serde_json::Value = serde_json::from_slice(fixture).unwrap();
        assert_eq!(serde_json::to_value(&rows).unwrap(), source, "{route}");
    }
}

#[test]
fn every_intraday_row_has_only_the_six_required_non_null_fields() {
    let expected_keys = ["close", "date", "high", "low", "open", "volume"]
        .into_iter()
        .collect::<BTreeSet<_>>();

    for (route, fixture) in FIXTURES {
        let source: serde_json::Value = serde_json::from_slice(fixture).unwrap();
        let object = source[0].as_object().unwrap();
        assert_eq!(object.len(), 6, "{route}");
        assert_eq!(
            object.keys().map(String::as_str).collect::<BTreeSet<_>>(),
            expected_keys,
            "{route}"
        );
        assert!(object.get("symbol").is_none(), "{route}");
        assert!(object.get("timezone").is_none(), "{route}");

        for key in object.keys() {
            let mut missing = source.clone();
            missing[0].as_object_mut().unwrap().remove(key);
            assert!(
                serde_json::from_value::<Vec<StockChartIntradayBar>>(missing).is_err(),
                "{route} accepted missing {key}"
            );

            let mut null = source.clone();
            null[0][key] = serde_json::Value::Null;
            assert!(
                serde_json::from_value::<Vec<StockChartIntradayBar>>(null).is_err(),
                "{route} accepted null {key}"
            );
        }
    }
}

#[test]
fn every_intraday_route_uses_a_strict_naive_datetime() {
    for (route, fixture) in FIXTURES {
        let rows: Vec<StockChartIntradayBar> = serde_json::from_slice(fixture).unwrap();
        assert_eq!(rows[0].date.to_string().len(), 19, "{route}");

        for zoned in ["2026-07-30T13:16:00Z", "2026-07-30 13:16:00-04:00"] {
            let mut value: serde_json::Value = serde_json::from_slice(fixture).unwrap();
            value[0]["date"] = serde_json::json!(zoned);
            assert!(
                serde_json::from_value::<Vec<StockChartIntradayBar>>(value).is_err(),
                "{route} accepted {zoned}"
            );
        }
    }
}

#[test]
fn every_intraday_route_preserves_empty_multiple_and_unknown_field_arrays() {
    for (route, fixture) in FIXTURES {
        let mut value: serde_json::Value = serde_json::from_slice(fixture).unwrap();
        value[0]["futureProviderField"] = serde_json::json!({"nested": [1, true, null]});
        let duplicate = value[0].clone();
        value.as_array_mut().unwrap().push(duplicate);

        let rows: Vec<StockChartIntradayBar> = serde_json::from_value(value).unwrap();
        assert_eq!(rows.len(), 2, "{route}");
        assert_eq!(rows[0], rows[1], "{route}");

        let empty: Vec<StockChartIntradayBar> = serde_json::from_slice(b"[]").unwrap();
        assert!(empty.is_empty(), "{route}");
    }
}
