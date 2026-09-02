use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Map, Value, json};

use libfmp::responses::commitment_of_traders::{CotAnalysis, CotReport, CotReportListing};

const REPORT: &[u8] = include_bytes!("fixtures/cot_report.json");
const ANALYSIS: &[u8] = include_bytes!("fixtures/cot_analysis.json");
const REPORT_LIST: &[u8] = include_bytes!("fixtures/cot_report_list.json");

#[test]
fn all_documented_cot_fixtures_decode_and_round_trip_exactly() {
    assert_exact::<CotReport>(REPORT);
    assert_exact::<CotAnalysis>(ANALYSIS);
    assert_exact::<CotReportListing>(REPORT_LIST);

    let report = rows::<CotReport>(REPORT).remove(0);
    assert_eq!(report.symbol.as_str(), "VX");
    assert_eq!(report.date.to_string(), "2024-02-27 00:00:00");
    assert_eq!(report.sector.as_str(), "INDICES");
    assert_eq!(report.change_in_noncomm_spread_all, 9_257);
    assert_eq!(report.traders_noncomm_spread_old, 101);
    assert_eq!(report.contract_units, "($1000 X INDEX)");

    let analysis = rows::<CotAnalysis>(ANALYSIS).remove(0);
    assert_eq!(
        analysis.exchange,
        "PALLADIUM - NEW YORK MERCANTILE EXCHANGE"
    );
    assert_eq!(analysis.net_position, -12_315);
    assert!(analysis.reversal_trend);
}

#[test]
fn report_has_exactly_the_documented_128_unique_keys_and_wire_hazards() {
    let source: Value = serde_json::from_slice(REPORT).unwrap();
    let source_row = source[0].as_object().unwrap();
    assert_eq!(source_row.len(), 128);

    let report = rows::<CotReport>(REPORT).remove(0);
    let encoded = serde_json::to_value(report).unwrap();
    let encoded_row = encoded.as_object().unwrap();
    assert_eq!(encoded_row.len(), 128);
    assert_eq!(
        encoded_row.keys().collect::<Vec<_>>(),
        source_row.keys().collect::<Vec<_>>()
    );
    for key in [
        "changeInNoncommSpeadAll",
        "tradersNoncommSpeadOl",
        "pctOfOpenInterestOl",
        "concGrossLe4TdrLongOl",
    ] {
        assert!(
            encoded_row.contains_key(key),
            "missing exact wire key {key}"
        );
    }
    for corrected_but_wrong_wire_key in [
        "changeInNoncommSpreadAll",
        "tradersNoncommSpreadOld",
        "pctOfOpenInterestOld",
        "concGrossLe4TdrLongOld",
    ] {
        assert!(!encoded_row.contains_key(corrected_but_wrong_wire_key));
    }
}

#[test]
fn report_preserves_integer_and_decimal_number_spellings() {
    let report = rows::<CotReport>(REPORT).remove(0);
    let encoded = serde_json::to_string(&report).unwrap();
    assert!(encoded.contains(r#""pctOfOpenInterestAll":100"#));
    assert!(encoded.contains(r#""pctOfOiNoncommLongAll":20.6"#));
    assert!(encoded.contains(r#""pctOfOiNoncommSpreadAll":27"#));
    assert!(encoded.contains(r#""concGrossLe8TdrShortAll":29"#));
    assert!(!encoded.contains(r#""pctOfOpenInterestAll":100.0"#));
    assert!(!encoded.contains(r#""pctOfOiNoncommSpreadAll":27.0"#));
}

#[test]
fn every_cot_field_is_required_non_null_and_unknown_fields_are_tolerated() {
    assert_required_contract::<CotReport>(REPORT, 128);
    assert_required_contract::<CotAnalysis>(ANALYSIS, 16);
    assert_required_contract::<CotReportListing>(REPORT_LIST, 2);
}

#[test]
fn every_cot_contract_requires_a_bare_array_root() {
    assert!(
        serde_json::from_str::<Vec<CotReport>>("[]")
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_str::<Vec<CotAnalysis>>("[]")
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_str::<Vec<CotReportListing>>("[]")
            .unwrap()
            .is_empty()
    );
    assert!(serde_json::from_str::<Vec<CotReport>>("{}").is_err());
    assert!(serde_json::from_str::<Vec<CotAnalysis>>("{}").is_err());
    assert!(serde_json::from_str::<Vec<CotReportListing>>("{}").is_err());
}

fn rows<T: DeserializeOwned>(fixture: &[u8]) -> Vec<T> {
    serde_json::from_slice(fixture).unwrap()
}

fn assert_exact<T>(fixture: &[u8])
where
    T: DeserializeOwned + Serialize,
{
    let source: Value = serde_json::from_slice(fixture).unwrap();
    assert_eq!(serde_json::to_value(rows::<T>(fixture)).unwrap(), source);
}

fn assert_required_contract<T>(fixture: &[u8], field_count: usize)
where
    T: DeserializeOwned,
{
    let source: Value = serde_json::from_slice(fixture).unwrap();
    let row = source[0].as_object().unwrap();
    assert_eq!(row.len(), field_count);

    for field in row.keys() {
        let mut missing = row.clone();
        missing.remove(field);
        assert!(
            serde_json::from_value::<T>(Value::Object(missing)).is_err(),
            "field {field} must be present"
        );

        let mut null = row.clone();
        null.insert(field.clone(), Value::Null);
        assert!(
            serde_json::from_value::<T>(Value::Object(null)).is_err(),
            "field {field} must not accept null"
        );
    }

    let mut future: Map<String, Value> = row.clone();
    future.insert("futureField".into(), json!({"nested": true}));
    assert!(serde_json::from_value::<T>(Value::Object(future)).is_ok());
}
