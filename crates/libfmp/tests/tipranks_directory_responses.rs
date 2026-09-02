use serde::de::DeserializeOwned;
use serde_json::{Map, Value, json};

use libfmp::{responses::tipranks::TipRanksAnalystProfile, types::TipRanksExpertUid};

const ANALYSTS: &[u8] = include_bytes!("fixtures/tipranks_analysts.json");

#[test]
fn exact_outer_md_fixture_decodes_with_nine_required_keys_and_round_trips() {
    let source: Value = serde_json::from_slice(ANALYSTS).unwrap();
    let source_row = source[0].as_object().unwrap();
    assert_eq!(source_row.len(), 9);

    let rows: Vec<TipRanksAnalystProfile> = serde_json::from_slice(ANALYSTS).unwrap();
    let row = &rows[0];
    assert_eq!(
        row.expert_uid,
        TipRanksExpertUid::new("0458d251af4db6d595c17e02da3bc6ae4bb093b0").unwrap()
    );
    assert_eq!(row.analyst_name, "Sujeeva De Silva");
    assert_eq!(row.firm_name, "Roth MKM");
    assert_eq!(row.success_rate.to_string(), "0.617");
    assert_eq!(row.excess_return.to_string(), "0.563");
    assert_eq!(row.total_recommendations, 496);
    assert_eq!(row.good_recommendations, 306);
    assert_eq!(row.analyst_rank, 24);
    assert_eq!(row.num_of_stars, 5);
    assert_eq!(serde_json::to_value(rows).unwrap(), source);
}

#[test]
fn every_documented_field_is_required_and_non_null() {
    assert_all_required::<TipRanksAnalystProfile>(source_row());
}

#[test]
fn performance_numbers_are_strict_bare_json_numbers() {
    for field in ["successRate", "excessReturn"] {
        let mut row = source_row();
        row.insert(field.into(), json!("0.617"));
        assert!(serde_json::from_value::<TipRanksAnalystProfile>(Value::Object(row)).is_err());
    }

    let mut row = source_row();
    row.insert("successRate".into(), json!(1));
    row.insert("excessReturn".into(), json!(-2.25));
    let decoded = serde_json::from_value::<TipRanksAnalystProfile>(Value::Object(row)).unwrap();
    assert_eq!(decoded.success_rate.to_string(), "1");
    assert_eq!(decoded.excess_return.to_string(), "-2.25");
}

#[test]
fn counters_reject_strings_signed_values_and_fractions() {
    for field in [
        "totalRecommendations",
        "goodRecommendations",
        "analystRank",
        "numOfStars",
    ] {
        for replacement in [json!("1"), json!(-1), json!(1.5)] {
            let mut row = source_row();
            row.insert(field.into(), replacement);
            assert!(
                serde_json::from_value::<TipRanksAnalystProfile>(Value::Object(row)).is_err(),
                "accepted invalid count for {field}"
            );
        }
    }
}

#[test]
fn strings_and_exact_expert_uid_wire_key_keep_documented_kinds_and_spelling() {
    for field in ["expertUID", "analystName", "firmName"] {
        let mut row = source_row();
        row.insert(field.into(), json!(1));
        assert!(serde_json::from_value::<TipRanksAnalystProfile>(Value::Object(row)).is_err());
    }

    let rows: Vec<TipRanksAnalystProfile> = serde_json::from_slice(ANALYSTS).unwrap();
    let encoded = serde_json::to_value(rows).unwrap();
    assert!(encoded[0].get("expertUID").is_some());
    for absent in ["expertUid", "expert_uid", "image", "link", "url"] {
        assert!(encoded[0].get(absent).is_none(), "emitted {absent}");
    }
}

#[test]
fn unknown_fields_are_tolerated_and_response_root_is_a_bare_array() {
    let mut row = source_row();
    row.insert("futureField".into(), json!({"nested": true}));
    row.insert(
        "profileImage".into(),
        json!("https://example.invalid/image"),
    );
    assert!(serde_json::from_value::<TipRanksAnalystProfile>(Value::Object(row)).is_ok());

    assert!(
        serde_json::from_slice::<Vec<TipRanksAnalystProfile>>(b"[]")
            .unwrap()
            .is_empty()
    );
    assert!(serde_json::from_slice::<Vec<TipRanksAnalystProfile>>(b"{}").is_err());
    assert!(serde_json::from_slice::<Vec<TipRanksAnalystProfile>>(b"[").is_err());
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

fn source_row() -> Map<String, Value> {
    serde_json::from_slice::<Value>(ANALYSTS).unwrap()[0]
        .as_object()
        .unwrap()
        .clone()
}
