use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Map, Value, json};

use libfmp::responses::fundraising::RegulationDOffering;

const LATEST: &[u8] = include_bytes!("fixtures/fundraising_latest.json");
const BY_CIK: &[u8] = include_bytes!("fixtures/fundraising_by_cik.json");
const NULLABLE: [&str; 3] = [
    "incorporatedWithinFiveYears",
    "revenueRange",
    "securitiesOfferedAreOfEquityType",
];

#[test]
fn both_exact_outer_md_fixtures_round_trip_as_the_shared_43_key_row() {
    assert_exact::<RegulationDOffering>(LATEST);
    assert_exact::<RegulationDOffering>(BY_CIK);

    for fixture in [LATEST, BY_CIK] {
        let source: Value = serde_json::from_slice(fixture).unwrap();
        let row = source[0].as_object().unwrap();
        assert_eq!(row.len(), 43);
        assert_eq!(
            row.keys().collect::<std::collections::HashSet<_>>().len(),
            43
        );
    }
}

#[test]
fn typed_fields_preserve_ciks_temporals_empty_sentinels_and_amounts() {
    let latest = rows::<RegulationDOffering>(LATEST).remove(0);
    assert_eq!(latest.cik.as_str(), "0002127786");
    assert_eq!(latest.date.to_string(), "2026-07-30");
    assert_eq!(latest.filing_date.to_string(), "2026-07-30 00:00:00");
    assert_eq!(latest.accepted_date.to_string(), "2026-07-30 13:05:23");
    assert_eq!(latest.form_type.as_str(), "D");
    assert_eq!(latest.incorporated_within_five_years, Some(true));
    assert_eq!(latest.year_of_incorporation, "2026");
    assert_eq!(latest.date_of_first_sale, None);
    assert_eq!(latest.total_number_already_invested, 0);

    let by_cik = rows::<RegulationDOffering>(BY_CIK).remove(0);
    assert_eq!(by_cik.cik.as_str(), "0001547416");
    assert_eq!(by_cik.incorporated_within_five_years, None);
    assert_eq!(by_cik.year_of_incorporation, "");
    assert_eq!(by_cik.date_of_first_sale.unwrap().to_string(), "2014-02-14");
    assert_eq!(by_cik.total_offering_amount, 71_999_990.0);
    assert_eq!(by_cik.total_number_already_invested, 24);
}

#[test]
fn nullable_members_are_required_present_and_every_other_key_is_non_null() {
    for fixture in [LATEST, BY_CIK] {
        let source: Value = serde_json::from_slice(fixture).unwrap();
        let row = source[0].as_object().unwrap();

        for field in row.keys() {
            let mut missing = row.clone();
            missing.remove(field);
            assert!(
                serde_json::from_value::<RegulationDOffering>(Value::Object(missing)).is_err(),
                "field {field} must be present"
            );

            let mut null = row.clone();
            null.insert(field.clone(), Value::Null);
            if NULLABLE.contains(&field.as_str()) {
                let decoded =
                    serde_json::from_value::<RegulationDOffering>(Value::Object(null)).unwrap();
                assert!(serde_json::to_value(decoded).unwrap()[field].is_null());
            } else {
                assert!(
                    serde_json::from_value::<RegulationDOffering>(Value::Object(null)).is_err(),
                    "field {field} must reject null"
                );
            }
        }
    }
}

#[test]
fn exact_temporal_boolean_count_and_amount_wire_shapes_are_enforced() {
    let source: Value = serde_json::from_slice(LATEST).unwrap();
    let row = source[0].as_object().unwrap();

    for (field, invalid_values) in [
        (
            "date",
            vec![json!("07-30-2026"), json!("2026-07-30 00:00:00")],
        ),
        (
            "filingDate",
            vec![json!("2026-07-30"), json!("2026-07-30T00:00:00")],
        ),
        (
            "acceptedDate",
            vec![json!("2026-07-30"), json!("2026-07-30T13:05:23")],
        ),
        (
            "dateOfFirstSale",
            vec![json!("02-14-2014"), json!("2014-02-30")],
        ),
    ] {
        for invalid in invalid_values {
            let mut candidate = row.clone();
            candidate.insert(field.into(), invalid);
            assert!(
                serde_json::from_value::<RegulationDOffering>(Value::Object(candidate)).is_err(),
                "field {field} accepted the wrong temporal shape"
            );
        }
    }

    for field in [
        "incorporatedWithinFiveYears",
        "isAmendment",
        "durationOfOfferingIsMoreThanYear",
        "securitiesOfferedAreOfEquityType",
        "isBusinessCombinationTransaction",
        "hasNonAccreditedInvestors",
    ] {
        let mut candidate = row.clone();
        candidate.insert(field.into(), json!("false"));
        assert!(
            serde_json::from_value::<RegulationDOffering>(Value::Object(candidate)).is_err(),
            "field {field} accepted a string-backed boolean"
        );
    }

    let mut candidate = row.clone();
    candidate.insert("totalNumberAlreadyInvested".into(), json!(-1));
    assert!(serde_json::from_value::<RegulationDOffering>(Value::Object(candidate)).is_err());

    for field in [
        "minimumInvestmentAccepted",
        "totalOfferingAmount",
        "totalAmountSold",
        "totalAmountRemaining",
        "salesCommissions",
        "findersFees",
        "grossProceedsUsed",
    ] {
        let mut candidate = row.clone();
        candidate.insert(field.into(), json!(1_250.75));
        let offering: RegulationDOffering =
            serde_json::from_value(Value::Object(candidate)).unwrap();
        assert_eq!(
            serde_json::to_value(offering).unwrap()[field],
            1_250.75,
            "field {field} lost a fractional amount"
        );
    }
}

#[test]
fn unknown_fields_are_tolerated_and_bare_arrays_are_required() {
    let source: Value = serde_json::from_slice(LATEST).unwrap();
    let mut row: Map<String, Value> = source[0].as_object().unwrap().clone();
    row.insert("futureField".into(), json!({"nested": [1, true]}));
    assert!(serde_json::from_value::<RegulationDOffering>(Value::Object(row)).is_ok());

    assert!(
        serde_json::from_str::<Vec<RegulationDOffering>>("[]")
            .unwrap()
            .is_empty()
    );
    assert!(serde_json::from_str::<Vec<RegulationDOffering>>("{}").is_err());
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
