use libfmp::{
    codecs::IsoTimestamp,
    responses::tipranks::TipRanksPointInTimeRating,
    types::{CurrencyCode, Date, Ticker, TipRanksExpertUid},
};
use serde_json::{Map, Value, json};

const SYMBOL: &[u8] = include_bytes!("fixtures/tipranks_point_in_time_symbol.json");
const ANALYST: &[u8] = include_bytes!("fixtures/tipranks_point_in_time_analyst.json");

#[test]
fn exact_outer_md_fixtures_decode_with_all_sixteen_fields_and_narrow_types() {
    let symbol_source: Value = serde_json::from_slice(SYMBOL).unwrap();
    let analyst_source: Value = serde_json::from_slice(ANALYST).unwrap();
    assert_eq!(symbol_source[0].as_object().unwrap().len(), 16);
    assert_eq!(analyst_source[0].as_object().unwrap().len(), 16);

    let symbol_rows: Vec<TipRanksPointInTimeRating> = serde_json::from_slice(SYMBOL).unwrap();
    let symbol = &symbol_rows[0];
    assert_eq!(symbol.symbol, Ticker::new("AAPL").unwrap());
    assert_eq!(
        symbol.date,
        IsoTimestamp::parse("2026-07-29T09:30:14.797Z").unwrap()
    );
    assert_eq!(
        symbol.expert_uid,
        TipRanksExpertUid::new("d970a430f313c453df61608e96e87edee44e3dce").unwrap()
    );
    assert_eq!(symbol.stock_success_rate.to_string(), "0.788");
    assert_eq!(symbol.last_recommendation, "buy");
    assert_eq!(
        symbol.last_recommendation_date,
        Date::parse("2026-07-28").unwrap()
    );
    assert_eq!(symbol.price_target.as_ref().unwrap().to_string(), "380");
    assert_eq!(
        symbol.price_target_currency,
        Some(CurrencyCode::new("USD").unwrap())
    );
    assert_eq!(symbol.stock_return.as_ref().unwrap().to_string(), "-0.1253");
    assert_eq!(symbol.beat_target, Some(false));
    assert_eq!(serde_json::to_value(symbol_rows).unwrap(), symbol_source);

    let analyst_rows: Vec<TipRanksPointInTimeRating> = serde_json::from_slice(ANALYST).unwrap();
    let analyst = &analyst_rows[0];
    assert_eq!(analyst.symbol, Ticker::new("0J3K.L").unwrap());
    assert_eq!(analyst.stock_success_rate.to_string(), "0");
    assert_eq!(analyst.last_recommendation, "Hold");
    assert_eq!(analyst.price_target, None);
    assert_eq!(analyst.price_target_currency, None);
    assert_eq!(analyst.stock_return, None);
    assert_eq!(analyst.beat_target, None);
    assert_eq!(serde_json::to_value(analyst_rows).unwrap(), analyst_source);
}

#[test]
fn nullable_fields_are_required_present_but_accept_null_and_values() {
    for source in [SYMBOL, ANALYST] {
        let row = source_row(source);
        for field in [
            "priceTarget",
            "priceTargetCurrency",
            "stockReturn",
            "beatTarget",
        ] {
            let mut missing = row.clone();
            missing.remove(field);
            assert!(
                serde_json::from_value::<TipRanksPointInTimeRating>(Value::Object(missing))
                    .is_err(),
                "missing nullable field {field} unexpectedly decoded"
            );
        }
    }

    let mut row = source_row(SYMBOL);
    for field in [
        "priceTarget",
        "priceTargetCurrency",
        "stockReturn",
        "beatTarget",
    ] {
        row.insert(field.into(), Value::Null);
    }
    let decoded = serde_json::from_value::<TipRanksPointInTimeRating>(Value::Object(row)).unwrap();
    assert_eq!(decoded.price_target, None);
    assert_eq!(decoded.price_target_currency, None);
    assert_eq!(decoded.stock_return, None);
    assert_eq!(decoded.beat_target, None);
}

#[test]
fn every_non_nullable_field_is_required_and_non_null() {
    let row = source_row(SYMBOL);
    for field in row.keys().filter(|field| {
        ![
            "priceTarget",
            "priceTargetCurrency",
            "stockReturn",
            "beatTarget",
        ]
        .contains(&field.as_str())
    }) {
        let mut missing = row.clone();
        missing.remove(field);
        assert!(
            serde_json::from_value::<TipRanksPointInTimeRating>(Value::Object(missing)).is_err(),
            "missing {field} unexpectedly decoded"
        );

        let mut null = row.clone();
        null.insert(field.clone(), Value::Null);
        assert!(
            serde_json::from_value::<TipRanksPointInTimeRating>(Value::Object(null)).is_err(),
            "null {field} unexpectedly decoded"
        );
    }
}

#[test]
fn numeric_fields_are_strict_numbers_and_preserve_integer_decimal_and_sign() {
    for (source, expected_success, expected_target, expected_return) in [
        (SYMBOL, "0.788", Some("380"), Some("-0.1253")),
        (ANALYST, "0", None, None),
    ] {
        let row: TipRanksPointInTimeRating =
            serde_json::from_value(Value::Object(source_row(source))).unwrap();
        assert_eq!(row.stock_success_rate.to_string(), expected_success);
        assert_eq!(
            row.price_target
                .as_ref()
                .map(ToString::to_string)
                .as_deref(),
            expected_target
        );
        assert_eq!(
            row.stock_return
                .as_ref()
                .map(ToString::to_string)
                .as_deref(),
            expected_return
        );
    }

    for field in ["stockSuccessRate", "priceTarget", "stockReturn"] {
        let mut row = source_row(SYMBOL);
        row.insert(field.into(), json!("0.5"));
        assert!(
            serde_json::from_value::<TipRanksPointInTimeRating>(Value::Object(row)).is_err(),
            "accepted numeric string for {field}"
        );
    }
}

#[test]
fn identifier_timestamp_date_currency_and_boolean_wire_kinds_are_strict() {
    for (field, replacement) in [
        ("symbol", json!("")),
        ("date", json!("2026-07-29 09:30:14")),
        ("expertUID", json!(123)),
        ("lastRecommendationDate", json!("2026-05-21T00:00:00Z")),
        ("priceTargetCurrency", json!("")),
        ("beatTarget", json!(0)),
    ] {
        let mut row = source_row(SYMBOL);
        row.insert(field.into(), replacement);
        assert!(
            serde_json::from_value::<TipRanksPointInTimeRating>(Value::Object(row)).is_err(),
            "accepted invalid {field}"
        );
    }
}

#[test]
fn exact_acronym_and_stock_return_keys_serialize_without_promised_absent_fields() {
    let rows: Vec<TipRanksPointInTimeRating> = serde_json::from_slice(SYMBOL).unwrap();
    let encoded = serde_json::to_value(rows).unwrap();
    assert!(encoded[0].get("expertUID").is_some());
    assert!(encoded[0].get("stockReturn").is_some());
    for absent in [
        "expertUid",
        "expert_uid",
        "stock_return",
        "analystRank",
        "stockAvgReturn",
    ] {
        assert!(encoded[0].get(absent).is_none(), "emitted {absent}");
    }
}

#[test]
fn open_text_preserves_recommendation_casing_and_unknown_fields_are_tolerated() {
    let symbol: TipRanksPointInTimeRating =
        serde_json::from_value(Value::Object(source_row(SYMBOL))).unwrap();
    let analyst: TipRanksPointInTimeRating =
        serde_json::from_value(Value::Object(source_row(ANALYST))).unwrap();
    assert_eq!(symbol.last_recommendation, "buy");
    assert_eq!(analyst.last_recommendation, "Hold");

    let mut future = source_row(SYMBOL);
    future.insert("futureField".into(), json!({"nested": true}));
    assert!(serde_json::from_value::<TipRanksPointInTimeRating>(Value::Object(future)).is_ok());
}

#[test]
fn response_contract_is_a_bare_array_and_accepts_an_empty_array() {
    assert!(
        serde_json::from_slice::<Vec<TipRanksPointInTimeRating>>(b"[]")
            .unwrap()
            .is_empty()
    );
    assert!(serde_json::from_slice::<Vec<TipRanksPointInTimeRating>>(b"{}").is_err());
}

fn source_row(source: &[u8]) -> Map<String, Value> {
    serde_json::from_slice::<Value>(source).unwrap()[0]
        .as_object()
        .unwrap()
        .clone()
}
