#[macro_use]
#[path = "support/assert_row.rs"]
mod assert_row;

use std::str::FromStr;

use libfmp::{
    endpoints::{metadata::GeographicAvailability, transcripts::earnings_transcript_list},
    query::FiscalPeriod,
    responses::transcripts::{
        EarningsTranscript, EarningsTranscriptAvailability, EarningsTranscriptDate,
        LatestEarningsTranscript,
    },
    types::{CalendarQuarter, CalendarYear, Date, Ticker},
};
use serde::de::DeserializeOwned;

const LATEST: &[u8] = include_bytes!("fixtures/latest_earnings_transcripts.json");
const TRANSCRIPT: &[u8] = include_bytes!("fixtures/earnings_transcript.json");
const DATES: &[u8] = include_bytes!("fixtures/earnings_transcript_dates.json");
const AVAILABILITY: &[u8] = include_bytes!("fixtures/directory_earnings_transcript_list.json");

const EXACT_CONTENT: &str = "Operator: Good day, everyone. Welcome to the Apple Incorporated Third Quarter Fiscal Year 2020 Earnings Conference Call. Today's call is being recorded. At this time, for opening remarks and introductions, I would like to turn things over to Mr. Tejas Gala, Senior Manager, Corporate Finance and Investor Relations. Please go ahead, sir.\nTejas Gala: Thank you. Good afternoon and thank you for joining us. Speaking first today is Apple's CEO, Tim Cook; and he'll be followed by CFO, Luca Maestri. Aft...";

#[test]
fn exact_latest_fixture_decodes_four_required_metadata_fields() {
    assert_field_count(LATEST, 4);
    let rows: Vec<LatestEarningsTranscript> = serde_json::from_slice(LATEST).unwrap();
    assert_rows!(
        rows,
        [LatestEarningsTranscript {
            symbol: Ticker::new("VLO").unwrap(),
            period: FiscalPeriod::Q2,
            fiscal_year: CalendarYear(2026),
            date: Date::from_str("2026-07-30").unwrap(),
        }]
    );
}

#[test]
fn exact_transcript_fixture_retains_the_complete_documented_content() {
    assert_field_count(TRANSCRIPT, 5);
    let rows: Vec<EarningsTranscript> = serde_json::from_slice(TRANSCRIPT).unwrap();
    assert_rows!(
        rows,
        [EarningsTranscript {
            symbol: Ticker::new("AAPL").unwrap(),
            period: FiscalPeriod::Q3,
            year: CalendarYear(2020),
            date: Date::from_str("2020-07-30").unwrap(),
            content: EXACT_CONTENT.to_owned(),
        }]
    );
    assert!(rows[0].content.contains("\nTejas Gala:"));
    assert!(rows[0].content.ends_with("Aft..."));
}

#[test]
fn exact_dates_fixture_uses_numeric_response_quarter_and_year() {
    assert_field_count(DATES, 3);
    let rows: Vec<EarningsTranscriptDate> = serde_json::from_slice(DATES).unwrap();
    assert_rows!(
        rows,
        [EarningsTranscriptDate {
            quarter: CalendarQuarter::new(2).unwrap(),
            fiscal_year: CalendarYear(2026),
            date: Date::from_str("2026-04-30").unwrap(),
        }]
    );

    for invalid in [serde_json::json!("2"), serde_json::json!(2.0)] {
        let mut value: serde_json::Value = serde_json::from_slice(DATES).unwrap();
        value[0]["quarter"] = invalid;
        assert!(serde_json::from_value::<Vec<EarningsTranscriptDate>>(value).is_err());
    }

    for invalid in [serde_json::json!("2026"), serde_json::json!(2026.0)] {
        for field in ["fiscalYear", "year"] {
            let fixture = if field == "year" { TRANSCRIPT } else { LATEST };
            let mut value: serde_json::Value = serde_json::from_slice(fixture).unwrap();
            value[0][field] = invalid.clone();
            if field == "year" {
                assert!(serde_json::from_value::<Vec<EarningsTranscript>>(value).is_err());
            } else {
                assert!(serde_json::from_value::<Vec<LatestEarningsTranscript>>(value).is_err());
            }
        }
    }
}

#[test]
fn transcript_content_is_unbounded_and_preserved_exactly() {
    let chunk = "Operator: café 中文 📈.\nQuoted: \"guidance\"; path=C:\\reports\\Q4; tab=\tend.\n";
    let content = chunk.repeat(40_000);
    assert!(content.len() > 2 * 1024 * 1024);
    let value = serde_json::json!([{
        "symbol": "BIG",
        "period": "Q4",
        "year": 4294967295_u32,
        "date": "2026-07-30",
        "content": content,
    }]);
    let wire = serde_json::to_vec(&value).unwrap();
    assert!(wire.windows(2).any(|window| window == b"\\n"));
    assert!(wire.windows(2).any(|window| window == b"\\\""));
    assert!(wire.windows(2).any(|window| window == b"\\\\"));

    let rows: Vec<EarningsTranscript> = serde_json::from_slice(&wire).unwrap();
    assert_eq!(rows[0].year, CalendarYear(u32::MAX));
    assert_eq!(rows[0].content.len(), content.len());
    assert_eq!(rows[0].content, content);
}

#[test]
fn every_documented_field_is_required_non_null_and_unknowns_are_accepted() {
    assert_contract::<LatestEarningsTranscript>(LATEST);
    assert_contract::<EarningsTranscript>(TRANSCRIPT);
    assert_contract::<EarningsTranscriptDate>(DATES);
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
    let duplicate = forward[0].clone();
    forward.as_array_mut().unwrap().push(duplicate);
    assert_eq!(serde_json::from_value::<Vec<T>>(forward).unwrap().len(), 2);
}

fn assert_field_count(fixture: &[u8], expected: usize) {
    let value: serde_json::Value = serde_json::from_slice(fixture).unwrap();
    assert_eq!(value[0].as_object().unwrap().len(), expected);
}

#[test]
fn all_transcript_rows_are_bare_arrays_preserving_empty_shapes() {
    assert!(
        serde_json::from_slice::<Vec<LatestEarningsTranscript>>(b"[]")
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_slice::<Vec<EarningsTranscript>>(b"[]")
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_slice::<Vec<EarningsTranscriptDate>>(b"[]")
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_value::<Vec<EarningsTranscript>>(serde_json::json!({
            "transcripts": []
        }))
        .is_err()
    );
}

#[test]
fn transcript_module_reexports_the_existing_us_only_availability_contract() {
    let endpoint = earnings_transcript_list();
    assert_eq!(endpoint.id(), "earnings-transcript-list");
    assert_eq!(
        endpoint.metadata().geography(),
        GeographicAvailability::UsOnly
    );

    let rows: Vec<EarningsTranscriptAvailability> = serde_json::from_slice(AVAILABILITY).unwrap();
    assert_eq!(rows[0].symbol.as_str(), "INBS");
    assert_eq!(rows[0].no_of_transcripts.as_str(), "6");
}
