use std::str::FromStr;

use libfmp::{
    query::FiscalPeriod,
    responses::statements::RevenueSegmentation,
    types::{CalendarYear, CurrencyCode, Date, Ticker},
};
use serde::de::DeserializeOwned;

const PRODUCT: &[u8] = include_bytes!("fixtures/revenue_product_segmentation.json");
const GEOGRAPHIC: &[u8] = include_bytes!("fixtures/revenue_geographic_segmentation.json");

#[test]
fn product_fixture_decodes_exact_envelope_and_arbitrary_product_keys() {
    let value: serde_json::Value = serde_json::from_slice(PRODUCT).unwrap();
    assert_eq!(value[0].as_object().unwrap().len(), 6);

    let rows: Vec<RevenueSegmentation> = serde_json::from_value(value).unwrap();
    let row = &rows[0];
    assert_eq!(row.symbol, Ticker::new("AAPL").unwrap());
    assert_eq!(row.fiscal_year, CalendarYear(2025));
    assert_eq!(row.period, FiscalPeriod::FullYear);
    assert_eq!(row.reported_currency, CurrencyCode::new("USD").unwrap());
    assert_eq!(row.date, Date::from_str("2025-09-27").unwrap());
    assert_eq!(row.data.len(), 5);
    assert_eq!(row.data["Mac"], serde_json::json!(33_708_000_000_u64));
    assert_eq!(
        row.data["Wearables, Home and Accessories"],
        serde_json::json!(35_686_000_000_u64)
    );
    assert_eq!(row.data["iPhone"], serde_json::json!(209_586_000_000_u64));
}

#[test]
fn geographic_fixture_decodes_exact_shared_shape_and_spaced_keys() {
    let value: serde_json::Value = serde_json::from_slice(GEOGRAPHIC).unwrap();
    assert_eq!(value[0].as_object().unwrap().len(), 6);

    let rows: Vec<RevenueSegmentation> = serde_json::from_value(value).unwrap();
    assert_eq!(rows[0].data.len(), 5);
    assert_eq!(
        rows[0].data["Greater China Segment"],
        serde_json::json!(64_377_000_000_u64)
    );
    assert_eq!(
        rows[0].data["Rest of Asia Pacific Segment"],
        serde_json::json!(33_696_000_000_u64)
    );
}

#[test]
fn dynamic_data_preserves_open_keys_values_and_large_integer_tokens() {
    let mut value: serde_json::Value = serde_json::from_slice(PRODUCT).unwrap();
    value[0]["data"]["Consumer / Pro, North America (Rest)"] =
        serde_json::from_str("9007199254740993").unwrap();
    value[0]["data"]["Future Segment: beta + services"] =
        serde_json::json!({ "reported": true, "adjustments": [null, -7] });

    let rows: Vec<RevenueSegmentation> = serde_json::from_value(value).unwrap();
    assert_eq!(
        rows[0].data["Consumer / Pro, North America (Rest)"].to_string(),
        "9007199254740993"
    );
    assert_eq!(
        rows[0].data["Future Segment: beta + services"]["adjustments"][1],
        serde_json::json!(-7)
    );

    let encoded = serde_json::to_string(&rows).unwrap();
    assert!(encoded.contains("9007199254740993"));
    assert!(!encoded.contains("9007199254740992"));
}

#[test]
fn every_envelope_field_is_required_non_null_and_exactly_typed() {
    for fixture in [PRODUCT, GEOGRAPHIC] {
        assert_every_field_required_and_non_null::<RevenueSegmentation>(fixture);
    }

    let mut string_year: serde_json::Value = serde_json::from_slice(PRODUCT).unwrap();
    string_year[0]["fiscalYear"] = serde_json::json!("2025");
    assert!(serde_json::from_value::<Vec<RevenueSegmentation>>(string_year).is_err());

    let mut retrieval_period: serde_json::Value = serde_json::from_slice(PRODUCT).unwrap();
    retrieval_period[0]["period"] = serde_json::json!("annual");
    assert!(serde_json::from_value::<Vec<RevenueSegmentation>>(retrieval_period).is_err());

    let mut array_data: serde_json::Value = serde_json::from_slice(PRODUCT).unwrap();
    array_data[0]["data"] = serde_json::json!([]);
    assert!(serde_json::from_value::<Vec<RevenueSegmentation>>(array_data).is_err());
}

fn assert_every_field_required_and_non_null<T: DeserializeOwned>(fixture: &[u8]) {
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
            "accepted missing field {key}"
        );

        let mut null = source.clone();
        null[0][&key] = serde_json::Value::Null;
        assert!(
            serde_json::from_value::<Vec<T>>(null).is_err(),
            "accepted null field {key}"
        );
    }
}

#[test]
fn both_endpoints_are_bare_arrays_preserving_empty_and_multiple_rows() {
    assert!(
        serde_json::from_slice::<Vec<RevenueSegmentation>>(b"[]")
            .unwrap()
            .is_empty()
    );
    for fixture in [PRODUCT, GEOGRAPHIC] {
        let mut value: serde_json::Value = serde_json::from_slice(fixture).unwrap();
        let duplicate = value[0].clone();
        value.as_array_mut().unwrap().push(duplicate);
        assert_eq!(
            serde_json::from_value::<Vec<RevenueSegmentation>>(value)
                .unwrap()
                .len(),
            2
        );
    }

    assert!(
        serde_json::from_value::<Vec<RevenueSegmentation>>(serde_json::json!({ "segments": [] }))
            .is_err()
    );
}
