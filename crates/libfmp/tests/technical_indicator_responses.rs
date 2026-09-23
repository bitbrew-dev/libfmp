use libfmp::responses::technical_indicators::{
    AverageDirectionalIndexBar, DoubleExponentialMovingAverageBar, ExponentialMovingAverageBar,
    RelativeStrengthIndexBar, SimpleMovingAverageBar, StandardDeviationBar,
    TripleExponentialMovingAverageBar, WeightedMovingAverageBar, WilliamsBar,
};
use serde::{Serialize, de::DeserializeOwned};

const SMA: &[u8] = include_bytes!("fixtures/technical_indicator_sma.json");
const EMA: &[u8] = include_bytes!("fixtures/technical_indicator_ema.json");
const WMA: &[u8] = include_bytes!("fixtures/technical_indicator_wma.json");
const DEMA: &[u8] = include_bytes!("fixtures/technical_indicator_dema.json");
const TEMA: &[u8] = include_bytes!("fixtures/technical_indicator_tema.json");
const RSI: &[u8] = include_bytes!("fixtures/technical_indicator_rsi.json");
const STANDARD_DEVIATION: &[u8] =
    include_bytes!("fixtures/technical_indicator_standard_deviation.json");
const WILLIAMS: &[u8] = include_bytes!("fixtures/technical_indicator_williams.json");
const ADX: &[u8] = include_bytes!("fixtures/technical_indicator_adx.json");

#[test]
fn all_nine_exact_source_rows_round_trip_with_seven_flat_fields() {
    assert_exact::<SimpleMovingAverageBar>(SMA);
    assert_exact::<ExponentialMovingAverageBar>(EMA);
    assert_exact::<WeightedMovingAverageBar>(WMA);
    assert_exact::<DoubleExponentialMovingAverageBar>(DEMA);
    assert_exact::<TripleExponentialMovingAverageBar>(TEMA);
    assert_exact::<RelativeStrengthIndexBar>(RSI);
    assert_exact::<StandardDeviationBar>(STANDARD_DEVIATION);
    assert_exact::<WilliamsBar>(WILLIAMS);
    assert_exact::<AverageDirectionalIndexBar>(ADX);
}

fn assert_exact<T: DeserializeOwned + Serialize>(fixture: &[u8]) {
    let source: serde_json::Value = serde_json::from_slice(fixture).unwrap();
    assert_eq!(source[0].as_object().unwrap().len(), 7);
    let rows: Vec<T> = serde_json::from_value(source.clone()).unwrap();
    assert_eq!(serde_json::to_value(rows).unwrap(), source);
}

#[test]
fn source_values_preserve_second_precision_large_volume_and_negative_williams() {
    let sma: Vec<SimpleMovingAverageBar> = serde_json::from_slice(SMA).unwrap();
    assert_eq!(sma[0].date.to_string(), "2026-07-30 00:00:00");
    assert_eq!(sma[0].volume, 29_207_295.0);
    assert_eq!(sma[0].sma, 331.621);

    let williams: Vec<WilliamsBar> = serde_json::from_slice(WILLIAMS).unwrap();
    assert_eq!(williams[0].williams, -48.29500396510714);

    let adx: Vec<AverageDirectionalIndexBar> = serde_json::from_slice(ADX).unwrap();
    assert_eq!(adx[0].adx, 34.69458756515438);
}

#[test]
fn every_documented_field_is_required_non_null_and_unknowns_are_accepted() {
    assert_contract::<SimpleMovingAverageBar>(SMA);
    assert_contract::<ExponentialMovingAverageBar>(EMA);
    assert_contract::<WeightedMovingAverageBar>(WMA);
    assert_contract::<DoubleExponentialMovingAverageBar>(DEMA);
    assert_contract::<TripleExponentialMovingAverageBar>(TEMA);
    assert_contract::<RelativeStrengthIndexBar>(RSI);
    assert_contract::<StandardDeviationBar>(STANDARD_DEVIATION);
    assert_contract::<WilliamsBar>(WILLIAMS);
    assert_contract::<AverageDirectionalIndexBar>(ADX);
}

fn assert_contract<T: DeserializeOwned>(fixture: &[u8]) {
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

    let mut forward = source;
    forward[0]["futureProviderField"] = serde_json::json!({ "nested": [1, true, null] });
    assert_eq!(serde_json::from_value::<Vec<T>>(forward).unwrap().len(), 1);
}

#[test]
fn every_contract_is_a_bare_array_preserving_empty_and_multiple_rows() {
    assert_bare_array::<SimpleMovingAverageBar>(SMA);
    assert_bare_array::<ExponentialMovingAverageBar>(EMA);
    assert_bare_array::<WeightedMovingAverageBar>(WMA);
    assert_bare_array::<DoubleExponentialMovingAverageBar>(DEMA);
    assert_bare_array::<TripleExponentialMovingAverageBar>(TEMA);
    assert_bare_array::<RelativeStrengthIndexBar>(RSI);
    assert_bare_array::<StandardDeviationBar>(STANDARD_DEVIATION);
    assert_bare_array::<WilliamsBar>(WILLIAMS);
    assert_bare_array::<AverageDirectionalIndexBar>(ADX);
    assert!(
        serde_json::from_value::<Vec<SimpleMovingAverageBar>>(serde_json::json!({ "sma": [] }))
            .is_err()
    );
}

fn assert_bare_array<T: DeserializeOwned>(fixture: &[u8]) {
    assert!(serde_json::from_slice::<Vec<T>>(b"[]").unwrap().is_empty());
    let mut source: serde_json::Value = serde_json::from_slice(fixture).unwrap();
    let duplicate = source[0].clone();
    source.as_array_mut().unwrap().push(duplicate);
    assert_eq!(serde_json::from_value::<Vec<T>>(source).unwrap().len(), 2);
}

#[test]
fn volume_preserves_values_above_u32_and_float_metrics_accept_integers_and_fractions() {
    let mut source: serde_json::Value = serde_json::from_slice(SMA).unwrap();
    source[0]["volume"] = serde_json::json!(u64::MAX);
    source[0]["sma"] = serde_json::json!(7);
    let integer: Vec<SimpleMovingAverageBar> = serde_json::from_value(source).unwrap();
    assert_eq!(integer[0].volume, u64::MAX as f64);
    assert_eq!(integer[0].sma, 7.0);

    let mut rsi: serde_json::Value = serde_json::from_slice(RSI).unwrap();
    rsi[0]["rsi"] = serde_json::json!(59.25);
    assert_eq!(
        serde_json::from_value::<Vec<RelativeStrengthIndexBar>>(rsi).unwrap()[0].rsi,
        59.25
    );

    let mut williams: serde_json::Value = serde_json::from_slice(WILLIAMS).unwrap();
    williams[0]["williams"] = serde_json::json!(-100);
    assert_eq!(
        serde_json::from_value::<Vec<WilliamsBar>>(williams).unwrap()[0].williams,
        -100.0
    );
}

#[test]
fn metric_keys_are_distinct_and_exactly_cased() {
    let fixtures = [
        (SMA, "sma", "SMA"),
        (EMA, "ema", "EMA"),
        (WMA, "wma", "WMA"),
        (DEMA, "dema", "DEMA"),
        (TEMA, "tema", "TEMA"),
        (RSI, "rsi", "RSI"),
        (
            STANDARD_DEVIATION,
            "standardDeviation",
            "standard_deviation",
        ),
        (WILLIAMS, "williams", "Williams"),
        (ADX, "adx", "ADX"),
    ];
    for (fixture, exact, wrong) in fixtures {
        let source: serde_json::Value = serde_json::from_slice(fixture).unwrap();
        assert!(source[0].get(exact).is_some());
        assert!(source[0].get(wrong).is_none());
    }

    let ema_source: serde_json::Value = serde_json::from_slice(EMA).unwrap();
    assert!(serde_json::from_value::<Vec<SimpleMovingAverageBar>>(ema_source).is_err());

    let mut standard: serde_json::Value = serde_json::from_slice(STANDARD_DEVIATION).unwrap();
    let value = standard[0]
        .as_object_mut()
        .unwrap()
        .remove("standardDeviation")
        .unwrap();
    standard[0]["standard_deviation"] = value;
    assert!(serde_json::from_value::<Vec<StandardDeviationBar>>(standard).is_err());
}

#[test]
fn timestamps_require_the_documented_naive_seconds_shape() {
    for invalid in ["2026-07-30 00:00", "2026-07-30T00:00:00Z"] {
        let mut source: serde_json::Value = serde_json::from_slice(SMA).unwrap();
        source[0]["date"] = serde_json::json!(invalid);
        assert!(serde_json::from_value::<Vec<SimpleMovingAverageBar>>(source).is_err());
    }
}
