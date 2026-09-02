use serde::de::DeserializeOwned;
use serde_json::{Map, Value, json};

use libfmp::{
    responses::tipranks::{TipRanksAnalystSummary, TipRanksFirmSummary, TipRanksSymbolSummary},
    types::{Date, Ticker, TipRanksExpertUid},
};

const SYMBOL: &[u8] = include_bytes!("fixtures/tipranks_symbol_summary.json");
const ANALYST: &[u8] = include_bytes!("fixtures/tipranks_analyst_summary.json");
const FIRM: &[u8] = include_bytes!("fixtures/tipranks_firm_summary.json");

#[test]
fn exact_outer_md_fixtures_decode_with_fifteen_top_level_and_exact_nested_keys() {
    for source in [SYMBOL, ANALYST, FIRM] {
        let row = source_row(source);
        assert_eq!(row.len(), 15);
        assert_eq!(row["recommendations"].as_object().unwrap().len(), 3);
        assert_eq!(row["analystAction"].as_object().unwrap().len(), 6);
    }

    let source: Value = serde_json::from_slice(SYMBOL).unwrap();
    let rows: Vec<TipRanksSymbolSummary> = serde_json::from_slice(SYMBOL).unwrap();
    let row = &rows[0];
    assert_eq!(row.symbol, Ticker::new("AAPL").unwrap());
    assert_eq!(row.from, Date::parse("2025-07-30").unwrap());
    assert_eq!(row.to, Date::parse("2026-07-30").unwrap());
    assert_eq!(row.total_recommendations, 425);
    assert_eq!(row.distinct_symbols, 1);
    assert_eq!(row.distinct_analysts, 45);
    assert_eq!(row.valid_price_targets, 379);
    assert_eq!(row.recommendations.buy, 277);
    assert_eq!(row.recommendations.hold, 122);
    assert_eq!(row.recommendations.sell, 26);
    assert_eq!(row.analyst_action.maintained, 309);
    assert_eq!(row.analyst_action.resumed, 0);
    assert_eq!(row.average_return.to_string(), "0.1697");
    assert_eq!(row.top_return.to_string(), "0.8466");
    assert_eq!(row.worst_return.to_string(), "-0.169");
    assert_eq!(serde_json::to_value(rows).unwrap(), source);

    let source: Value = serde_json::from_slice(ANALYST).unwrap();
    let rows: Vec<TipRanksAnalystSummary> = serde_json::from_slice(ANALYST).unwrap();
    let row = &rows[0];
    assert_eq!(
        row.expert_uid,
        TipRanksExpertUid::new("3c6eb8cf1347a4e5757e628fccb684a93abee587").unwrap()
    );
    assert_eq!(row.total_recommendations, 22);
    assert_eq!(row.recommendations.sell, 0);
    assert_eq!(row.beats, 2);
    assert_eq!(row.misses, 11);
    assert_eq!(row.average_return.to_string(), "-0.1206");
    assert_eq!(serde_json::to_value(rows).unwrap(), source);

    let source: Value = serde_json::from_slice(FIRM).unwrap();
    let rows: Vec<TipRanksFirmSummary> = serde_json::from_slice(FIRM).unwrap();
    let row = &rows[0];
    assert_eq!(row.firm_name, "Morgan Stanley");
    assert_eq!(row.total_recommendations, 14_182);
    assert_eq!(row.distinct_symbols, 4_146);
    assert_eq!(row.distinct_analysts, 258);
    assert_eq!(row.analyst_action.reiterated, 1_583);
    assert_eq!(row.top_return.to_string(), "17.0036");
    assert_eq!(row.worst_return.to_string(), "-1");
    assert_eq!(serde_json::to_value(rows).unwrap(), source);
}

#[test]
fn every_top_level_and_nested_field_is_required_and_non_null() {
    assert_all_required::<TipRanksSymbolSummary>(source_row(SYMBOL));
    assert_all_required::<TipRanksAnalystSummary>(source_row(ANALYST));
    assert_all_required::<TipRanksFirmSummary>(source_row(FIRM));

    for nested in ["recommendations", "analystAction"] {
        let row = source_row(SYMBOL);
        let object = row[nested].as_object().unwrap();
        for key in object.keys() {
            let mut changed = row.clone();
            changed[nested].as_object_mut().unwrap().remove(key);
            assert!(
                serde_json::from_value::<TipRanksSymbolSummary>(Value::Object(changed)).is_err(),
                "missing {nested}.{key} unexpectedly decoded"
            );

            let mut changed = row.clone();
            changed[nested]
                .as_object_mut()
                .unwrap()
                .insert(key.clone(), Value::Null);
            assert!(
                serde_json::from_value::<TipRanksSymbolSummary>(Value::Object(changed)).is_err(),
                "null {nested}.{key} unexpectedly decoded"
            );
        }
    }
}

#[test]
fn counts_and_returns_keep_strict_documented_json_kinds() {
    for field in [
        "totalRecommendations",
        "distinctSymbols",
        "distinctAnalysts",
        "validPriceTargets",
        "comparedPriceTargets",
        "beats",
        "misses",
    ] {
        for replacement in [json!("1"), json!(-1), json!(1.5)] {
            let mut row = source_row(SYMBOL);
            row.insert(field.into(), replacement);
            assert!(
                serde_json::from_value::<TipRanksSymbolSummary>(Value::Object(row)).is_err(),
                "accepted invalid count for {field}"
            );
        }
    }

    for (nested, key) in [("recommendations", "buy"), ("analystAction", "initiated")] {
        for replacement in [json!("1"), json!(-1), json!(1.5)] {
            let mut row = source_row(SYMBOL);
            row[nested]
                .as_object_mut()
                .unwrap()
                .insert(key.into(), replacement);
            assert!(
                serde_json::from_value::<TipRanksSymbolSummary>(Value::Object(row)).is_err(),
                "accepted invalid count for {nested}.{key}"
            );
        }
    }

    for field in ["averageReturn", "topReturn", "worstReturn"] {
        let mut row = source_row(SYMBOL);
        row.insert(field.into(), json!("-1"));
        assert!(serde_json::from_value::<TipRanksSymbolSummary>(Value::Object(row)).is_err());
    }

    let mut row = source_row(SYMBOL);
    row.insert("averageReturn".into(), json!(-2.25));
    row.insert("topReturn".into(), json!(17.0036));
    row.insert("worstReturn".into(), json!(-1));
    let decoded = serde_json::from_value::<TipRanksSymbolSummary>(Value::Object(row)).unwrap();
    assert_eq!(decoded.average_return.to_string(), "-2.25");
    assert_eq!(decoded.top_return.to_string(), "17.0036");
    assert_eq!(decoded.worst_return.to_string(), "-1");
}

#[test]
fn exact_identity_and_analyst_action_keys_are_preserved_without_aliases() {
    let analyst: Vec<TipRanksAnalystSummary> = serde_json::from_slice(ANALYST).unwrap();
    let encoded = serde_json::to_value(analyst).unwrap();
    assert!(encoded[0].get("expertUID").is_some());
    assert!(encoded[0].get("analystAction").is_some());
    for absent in [
        "expertUid",
        "expert_uid",
        "analystActions",
        "analyst_action",
    ] {
        assert!(encoded[0].get(absent).is_none(), "emitted {absent}");
    }
}

#[test]
fn summary_models_reject_other_identity_shapes() {
    assert!(serde_json::from_slice::<Vec<TipRanksAnalystSummary>>(SYMBOL).is_err());
    assert!(serde_json::from_slice::<Vec<TipRanksFirmSummary>>(SYMBOL).is_err());
    assert!(serde_json::from_slice::<Vec<TipRanksSymbolSummary>>(ANALYST).is_err());
    assert!(serde_json::from_slice::<Vec<TipRanksFirmSummary>>(ANALYST).is_err());
    assert!(serde_json::from_slice::<Vec<TipRanksSymbolSummary>>(FIRM).is_err());
    assert!(serde_json::from_slice::<Vec<TipRanksAnalystSummary>>(FIRM).is_err());
}

#[test]
fn unknown_fields_are_tolerated_and_response_is_a_bare_array() {
    let mut row = source_row(SYMBOL);
    row.insert("futureField".into(), json!({"nested": true}));
    row["recommendations"]
        .as_object_mut()
        .unwrap()
        .insert("futureRating".into(), json!(9));
    row["analystAction"]
        .as_object_mut()
        .unwrap()
        .insert("futureAction".into(), json!(3));
    assert!(serde_json::from_value::<TipRanksSymbolSummary>(Value::Object(row)).is_ok());

    assert!(
        serde_json::from_slice::<Vec<TipRanksSymbolSummary>>(b"[]")
            .unwrap()
            .is_empty()
    );
    assert!(serde_json::from_slice::<Vec<TipRanksSymbolSummary>>(b"{}").is_err());
}

fn assert_all_required<T: DeserializeOwned>(row: Map<String, Value>) {
    for field in row.keys() {
        let mut missing = row.clone();
        missing.remove(field);
        assert!(
            serde_json::from_value::<T>(Value::Object(missing)).is_err(),
            "missing {field} unexpectedly decoded"
        );

        let mut null = row.clone();
        null.insert(field.clone(), Value::Null);
        assert!(
            serde_json::from_value::<T>(Value::Object(null)).is_err(),
            "null {field} unexpectedly decoded"
        );
    }
}

fn source_row(source: &[u8]) -> Map<String, Value> {
    serde_json::from_slice::<Value>(source).unwrap()[0]
        .as_object()
        .unwrap()
        .clone()
}
