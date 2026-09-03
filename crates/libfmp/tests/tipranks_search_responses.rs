use libfmp::{
    codecs::IsoTimestamp,
    responses::tipranks::TipRanksRatingSearchResult,
    types::{CurrencyCode, Date, Ticker, TipRanksExpertUid},
};
use serde_json::{Map, Value, json};

const SEARCH: &[u8] = include_bytes!("fixtures/tipranks_ratings_search.json");

#[test]
fn exact_outer_md_fixture_decodes_with_all_thirteen_fields_and_narrow_types() {
    let source: Value = serde_json::from_slice(SEARCH).unwrap();
    assert_eq!(source[0].as_object().unwrap().len(), 13);

    let rows: Vec<TipRanksRatingSearchResult> = serde_json::from_slice(SEARCH).unwrap();
    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row.symbol, Ticker::new("RR.L").unwrap());
    assert_eq!(
        row.date,
        IsoTimestamp::parse("2026-07-30T16:40:58.403Z").unwrap()
    );
    assert_eq!(row.recommendation_date, Date::parse("2026-07-30").unwrap());
    assert_eq!(
        row.expert_uid,
        TipRanksExpertUid::new("9d6962cbd29862b8d70de0a2ddb3eb0bdfedc2b7").unwrap()
    );
    assert_eq!(row.recommendation, "buy");
    assert_eq!(row.analyst_action, "maintained");
    assert_eq!(row.article_site, "TipRanks Contributor");
    assert_eq!(row.price_target.to_string(), "1500");
    assert_eq!(row.price_target_currency, CurrencyCode::new("GBX").unwrap());
    assert_eq!(serde_json::to_value(rows).unwrap(), source);
}

#[test]
fn expert_uid_uses_the_exact_acronym_key_and_serialization_is_faithful() {
    let rows: Vec<TipRanksRatingSearchResult> = serde_json::from_slice(SEARCH).unwrap();
    let encoded = serde_json::to_value(rows).unwrap();
    assert_eq!(
        encoded[0]["expertUID"],
        "9d6962cbd29862b8d70de0a2ddb3eb0bdfedc2b7"
    );
    for wrong in ["expertUid", "expert_uid", "expertId"] {
        assert!(encoded[0].get(wrong).is_none(), "emitted {wrong}");
    }
}

#[test]
fn every_documented_field_is_required_non_null_and_unknown_fields_are_tolerated() {
    let row = source_row();
    for field in row.keys() {
        let mut missing = row.clone();
        missing.remove(field);
        assert!(
            serde_json::from_value::<TipRanksRatingSearchResult>(Value::Object(missing)).is_err(),
            "missing {field} unexpectedly decoded"
        );

        let mut null = row.clone();
        null.insert(field.clone(), Value::Null);
        assert!(
            serde_json::from_value::<TipRanksRatingSearchResult>(Value::Object(null)).is_err(),
            "null {field} unexpectedly decoded"
        );
    }

    let mut future = row;
    future.insert("futureField".into(), json!({"nested": true}));
    assert!(serde_json::from_value::<TipRanksRatingSearchResult>(Value::Object(future)).is_ok());
}

#[test]
fn temporal_numeric_and_identifier_wire_kinds_are_strict() {
    for (field, replacement) in [
        ("date", json!("2026-07-30 16:40:58")),
        ("recommendationDate", json!("2026-07-30T00:00:00Z")),
        ("expertUID", json!(123)),
        ("symbol", json!("")),
        ("priceTarget", json!("1500")),
        ("priceTargetCurrency", json!("")),
    ] {
        let mut row = source_row();
        row.insert(field.into(), replacement);
        assert!(
            serde_json::from_value::<TipRanksRatingSearchResult>(Value::Object(row)).is_err(),
            "accepted invalid {field}"
        );
    }
}

#[test]
fn recommendation_action_and_attribution_text_remain_open_strings() {
    let mut row = source_row();
    for field in [
        "analystName",
        "firmName",
        "recommendation",
        "analystAction",
        "articleTitle",
        "articleSite",
        "url",
    ] {
        row.insert(field.into(), json!(""));
    }
    let decoded = serde_json::from_value::<TipRanksRatingSearchResult>(Value::Object(row)).unwrap();
    assert!(decoded.recommendation.is_empty());
    assert!(decoded.analyst_action.is_empty());
    assert!(decoded.article_site.is_empty());
}

#[test]
fn response_contract_is_a_bare_array_and_accepts_an_empty_array() {
    assert!(
        serde_json::from_slice::<Vec<TipRanksRatingSearchResult>>(b"[]")
            .unwrap()
            .is_empty()
    );
    assert!(serde_json::from_slice::<Vec<TipRanksRatingSearchResult>>(b"{}").is_err());
}

fn source_row() -> Map<String, Value> {
    serde_json::from_slice::<Value>(SEARCH).unwrap()[0]
        .as_object()
        .unwrap()
        .clone()
}
