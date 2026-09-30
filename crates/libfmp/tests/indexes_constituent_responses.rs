use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};

use libfmp::responses::indexes::{HistoricalIndexConstituent, IndexConstituent};

const SP500: &[u8] = include_bytes!("fixtures/indexes_sp500_constituents.json");
const NASDAQ: &[u8] = include_bytes!("fixtures/indexes_nasdaq_constituents.json");
const DOW_JONES: &[u8] = include_bytes!("fixtures/indexes_dow_jones_constituents.json");
const HISTORICAL_SP500: &[u8] =
    include_bytes!("fixtures/indexes_historical_sp500_constituents.json");
const HISTORICAL_NASDAQ: &[u8] =
    include_bytes!("fixtures/indexes_historical_nasdaq_constituents.json");
const HISTORICAL_DOW_JONES: &[u8] =
    include_bytes!("fixtures/indexes_historical_dow_jones_constituents.json");

#[test]
fn all_six_documented_constituent_fixtures_match_the_typed_wire_contract_exactly() {
    for fixture in [SP500, NASDAQ, DOW_JONES] {
        assert_exact::<IndexConstituent>(fixture);
    }
    for fixture in [HISTORICAL_SP500, HISTORICAL_NASDAQ, HISTORICAL_DOW_JONES] {
        assert_exact::<HistoricalIndexConstituent>(fixture);
    }

    let current: Vec<IndexConstituent> = serde_json::from_slice(SP500).unwrap();
    assert_eq!(current[0].cik.as_str(), "0002089271");
    assert_eq!(current[0].sub_sector.as_str(), "Aerospace & Defense");
    assert_eq!(
        current[0].date_first_added.unwrap().to_string(),
        "2026-06-29"
    );

    let nullable: Vec<IndexConstituent> = serde_json::from_slice(NASDAQ).unwrap();
    assert_eq!(nullable[0].cik.as_str(), "0000796343");
    assert_eq!(nullable[0].date_first_added, None);

    let history: Vec<HistoricalIndexConstituent> =
        serde_json::from_slice(HISTORICAL_NASDAQ).unwrap();
    assert_eq!(history[0].date_added.0, "July 7, 2026");
    assert_eq!(history[0].removed_ticker, None);
    assert_eq!(history[0].removed_security, None);

    let dow_history: Vec<HistoricalIndexConstituent> =
        serde_json::from_slice(HISTORICAL_DOW_JONES).unwrap();
    assert_eq!(dow_history[0].reason, None);
    assert_eq!(
        dow_history[0].added_security.as_deref(),
        Some("Alphabet Inc.")
    );
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
fn current_constituent_fields_are_required_and_nullable_date_stays_required() {
    let source: Value = serde_json::from_slice(NASDAQ).unwrap();
    let row = source[0].clone();

    for field in [
        "symbol",
        "name",
        "sector",
        "subSector",
        "headQuarter",
        "dateFirstAdded",
        "cik",
        "founded",
    ] {
        let mut missing = row.clone();
        missing.as_object_mut().unwrap().remove(field);
        assert!(serde_json::from_value::<IndexConstituent>(missing).is_err());
    }

    for field in [
        "symbol",
        "name",
        "sector",
        "subSector",
        "headQuarter",
        "cik",
        "founded",
    ] {
        let mut null = row.clone();
        null[field] = Value::Null;
        assert!(serde_json::from_value::<IndexConstituent>(null).is_err());
    }

    let mut future = row;
    future["futureField"] = json!({"nested": true});
    let decoded: IndexConstituent = serde_json::from_value(future).unwrap();
    assert_eq!(decoded.symbol.as_str(), "ADBE");
}

#[test]
fn historical_fields_are_required_while_omittable_members_accept_null_or_empty() {
    let source: Value = serde_json::from_slice(HISTORICAL_NASDAQ).unwrap();
    let row = source[0].clone();

    for field in [
        "dateAdded",
        "addedSecurity",
        "removedTicker",
        "removedSecurity",
        "date",
        "symbol",
        "reason",
    ] {
        let mut missing = row.clone();
        missing.as_object_mut().unwrap().remove(field);
        assert!(serde_json::from_value::<HistoricalIndexConstituent>(missing).is_err());
    }

    for field in ["dateAdded", "date", "symbol"] {
        let mut null = row.clone();
        null[field] = Value::Null;
        assert!(serde_json::from_value::<HistoricalIndexConstituent>(null).is_err());
    }

    let mut omitted = row.clone();
    omitted["addedSecurity"] = Value::Null;
    omitted["reason"] = Value::Null;
    omitted["removedTicker"] = json!("");
    let decoded: HistoricalIndexConstituent = serde_json::from_value(omitted).unwrap();
    assert_eq!(decoded.added_security, None);
    assert_eq!(decoded.reason, None);
    assert_eq!(decoded.removed_ticker, None);
    let wire = serde_json::to_value(&decoded).unwrap();
    assert!(wire["addedSecurity"].is_null());
    assert!(wire["reason"].is_null());
    assert!(wire["removedTicker"].is_null());

    let mut future = row;
    future["futureField"] = json!([1, 2, 3]);
    let decoded: HistoricalIndexConstituent = serde_json::from_value(future).unwrap();
    assert_eq!(decoded.symbol.as_str(), "SPCX");
}

#[test]
fn constituent_contracts_preserve_bare_array_roots() {
    assert!(
        serde_json::from_str::<Vec<IndexConstituent>>("[]")
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_str::<Vec<HistoricalIndexConstituent>>("[]")
            .unwrap()
            .is_empty()
    );

    let current: Value = serde_json::from_slice(SP500).unwrap();
    assert!(serde_json::from_value::<IndexConstituent>(current).is_err());
    let historical: Value = serde_json::from_slice(HISTORICAL_SP500).unwrap();
    assert!(serde_json::from_value::<HistoricalIndexConstituent>(historical).is_err());
}
