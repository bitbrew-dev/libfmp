#[macro_use]
#[path = "support/assert_row.rs"]
mod assert_row;

use std::str::FromStr;

use libfmp::{
    responses::institutional_ownership::{
        Form13fFilingDate, InstitutionalHolding, InstitutionalOwnershipFiling,
    },
    types::{ApiDateTime, CalendarQuarter, CalendarYear, Cik, Cusip, Date, Ticker},
};
use serde::de::DeserializeOwned;

const LATEST: &[u8] = include_bytes!("fixtures/latest_institutional_ownership_filings.json");
const EXTRACT: &[u8] = include_bytes!("fixtures/institutional_ownership_extract.json");
const DATES: &[u8] = include_bytes!("fixtures/form_13f_filing_dates.json");

#[test]
fn exact_latest_fixture_decodes_eight_fields_and_distinct_temporal_kinds() {
    assert_field_count(LATEST, 8);
    let rows: Vec<InstitutionalOwnershipFiling> = serde_json::from_slice(LATEST).unwrap();
    assert_rows!(
        rows,
        [InstitutionalOwnershipFiling {
            cik: Cik::new("0001803005").unwrap(),
            name: "WEALTH ADVISORS OF IOWA, LLC".to_owned(),
            date: Date::from_str("2026-06-30").unwrap(),
            filing_date: ApiDateTime::parse("2026-07-30 00:00:00").unwrap(),
            accepted_date: ApiDateTime::parse("2026-07-30 13:14:23").unwrap(),
            form_type: "13F-HR".to_owned(),
            link: "https://www.sec.gov/Archives/edgar/data/1803005/000180300526000003/0001803005-26-000003-index.htm".to_owned(),
            final_link: "https://www.sec.gov/Archives/edgar/data/1803005/000180300526000003/xslForm13F_X02/primary_doc.xml".to_owned(),
        }]
    );
    assert_eq!(rows[0].cik.as_str(), "0001803005");
}

#[test]
fn exact_extract_fixture_decodes_fourteen_fields_and_required_empty_put_call() {
    assert_field_count(EXTRACT, 14);
    let rows: Vec<InstitutionalHolding> = serde_json::from_slice(EXTRACT).unwrap();
    assert_rows!(
        rows,
        [InstitutionalHolding {
            date: Date::from_str("2023-09-30").unwrap(),
            filing_date: Date::from_str("2023-11-13").unwrap(),
            accepted_date: Date::from_str("2023-11-13").unwrap(),
            cik: Cik::new("0001388838").unwrap(),
            security_cusip: Cusip::new("674215207").unwrap(),
            symbol: Ticker::new("CHRD").unwrap(),
            name_of_issuer: "CHORD ENERGY CORPORATION".to_owned(),
            shares: 13_280.0,
            title_of_class: "COM NEW".to_owned(),
            shares_type: "SH".to_owned(),
            put_call_share: String::new(),
            value: 2_152_290.0,
            link: "https://www.sec.gov/Archives/edgar/data/1388838/000117266123003760/0001172661-23-003760-index.htm".to_owned(),
            final_link: "https://www.sec.gov/Archives/edgar/data/1388838/000117266123003760/infotable.xml".to_owned(),
        }]
    );
    assert_eq!(rows[0].cik.as_str(), "0001388838");
    assert_eq!(rows[0].security_cusip.as_str(), "674215207");
    assert!(rows[0].put_call_share.is_empty());

    let mut leading_zero: serde_json::Value = serde_json::from_slice(EXTRACT).unwrap();
    leading_zero[0]["securityCusip"] = serde_json::json!("001234567");
    let rows: Vec<InstitutionalHolding> = serde_json::from_value(leading_zero).unwrap();
    assert_eq!(rows[0].security_cusip.as_str(), "001234567");
}

#[test]
fn exact_dates_fixture_uses_numeric_calendar_year_and_quarter() {
    assert_field_count(DATES, 3);
    let rows: Vec<Form13fFilingDate> = serde_json::from_slice(DATES).unwrap();
    assert_rows!(
        rows,
        [Form13fFilingDate {
            date: Date::from_str("2026-03-31").unwrap(),
            year: CalendarYear(2026),
            quarter: CalendarQuarter::new(1).unwrap(),
        }]
    );

    for replacement in [serde_json::json!("2026"), serde_json::json!(2026.0)] {
        let mut value: serde_json::Value = serde_json::from_slice(DATES).unwrap();
        value[0]["year"] = replacement;
        assert!(serde_json::from_value::<Vec<Form13fFilingDate>>(value).is_err());
    }
    for replacement in [serde_json::json!("1"), serde_json::json!(1.0)] {
        let mut value: serde_json::Value = serde_json::from_slice(DATES).unwrap();
        value[0]["quarter"] = replacement;
        assert!(serde_json::from_value::<Vec<Form13fFilingDate>>(value).is_err());
    }
}

#[test]
fn date_and_datetime_fields_reject_each_others_wire_kinds() {
    let mut latest: serde_json::Value = serde_json::from_slice(LATEST).unwrap();
    latest[0]["date"] = serde_json::json!("2026-06-30 00:00:00");
    assert!(serde_json::from_value::<Vec<InstitutionalOwnershipFiling>>(latest).is_err());

    for field in ["filingDate", "acceptedDate"] {
        let mut latest: serde_json::Value = serde_json::from_slice(LATEST).unwrap();
        latest[0][field] = serde_json::json!("2026-07-30");
        assert!(serde_json::from_value::<Vec<InstitutionalOwnershipFiling>>(latest).is_err());
    }

    for field in ["date", "filingDate", "acceptedDate"] {
        let mut extract: serde_json::Value = serde_json::from_slice(EXTRACT).unwrap();
        extract[0][field] = serde_json::json!("2023-11-13 00:00:00");
        assert!(serde_json::from_value::<Vec<InstitutionalHolding>>(extract).is_err());
    }
}

#[test]
fn shares_and_value_decode_large_fractional_and_negative_numbers_but_reject_text() {
    let mut value: serde_json::Value = serde_json::from_slice(EXTRACT).unwrap();
    value[0]["shares"] = serde_json::json!(u64::MAX);
    value[0]["value"] = serde_json::json!(u64::MAX - 1);
    let rows: Vec<InstitutionalHolding> = serde_json::from_value(value).unwrap();
    assert_eq!(rows[0].shares, u64::MAX as f64);
    assert_eq!(rows[0].value, (u64::MAX - 1) as f64);

    for field in ["shares", "value"] {
        for number in [serde_json::json!(-1), serde_json::json!(1.5)] {
            let mut value: serde_json::Value = serde_json::from_slice(EXTRACT).unwrap();
            value[0][field] = number.clone();
            let rows: Vec<InstitutionalHolding> = serde_json::from_value(value).unwrap();
            assert_eq!(serde_json::to_value(&rows[0]).unwrap()[field], number);
        }
        let mut value: serde_json::Value = serde_json::from_slice(EXTRACT).unwrap();
        value[0][field] = serde_json::json!("13280");
        assert!(serde_json::from_value::<Vec<InstitutionalHolding>>(value).is_err());
    }
}

#[test]
fn all_fields_are_required_non_null_and_unknowns_are_accepted() {
    assert_contract::<InstitutionalOwnershipFiling>(LATEST);
    assert_contract::<InstitutionalHolding>(EXTRACT);
    assert_contract::<Form13fFilingDate>(DATES);
}

#[test]
fn all_three_contracts_are_bare_arrays_preserving_empty_and_multiple_rows() {
    assert!(
        serde_json::from_slice::<Vec<InstitutionalOwnershipFiling>>(b"[]")
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_slice::<Vec<InstitutionalHolding>>(b"[]")
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_slice::<Vec<Form13fFilingDate>>(b"[]")
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_value::<Vec<InstitutionalHolding>>(serde_json::json!({
            "holdings": []
        }))
        .is_err()
    );

    assert_multiple::<InstitutionalOwnershipFiling>(LATEST);
    assert_multiple::<InstitutionalHolding>(EXTRACT);
    assert_multiple::<Form13fFilingDate>(DATES);
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

fn assert_multiple<T: DeserializeOwned>(fixture: &[u8]) {
    let mut value: serde_json::Value = serde_json::from_slice(fixture).unwrap();
    let duplicate = value[0].clone();
    value.as_array_mut().unwrap().push(duplicate);
    assert_eq!(serde_json::from_value::<Vec<T>>(value).unwrap().len(), 2);
}

fn assert_field_count(fixture: &[u8], expected: usize) {
    let value: serde_json::Value = serde_json::from_slice(fixture).unwrap();
    assert_eq!(value[0].as_object().unwrap().len(), expected);
}
