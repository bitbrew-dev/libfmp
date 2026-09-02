use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Map, Value, json};

use libfmp::responses::fundraising::{
    CrowdfundingOfferingSearchResult, RegulationDOfferingSearchResult,
};

const CROWDFUNDING: &[u8] = include_bytes!("fixtures/crowdfunding_offerings_search.json");
const REGULATION_D: &[u8] = include_bytes!("fixtures/fundraising_search.json");

#[test]
fn exact_outer_md_fixtures_round_trip_as_distinct_three_key_rows() {
    assert_exact::<CrowdfundingOfferingSearchResult>(CROWDFUNDING);
    assert_exact::<RegulationDOfferingSearchResult>(REGULATION_D);

    let crowdfunding = rows::<CrowdfundingOfferingSearchResult>(CROWDFUNDING).remove(0);
    assert_eq!(crowdfunding.cik.as_str(), "0001912939");
    assert_eq!(crowdfunding.name, "Enotap LLC");
    assert_eq!(crowdfunding.date, None);

    let regulation_d = rows::<RegulationDOfferingSearchResult>(REGULATION_D).remove(0);
    assert_eq!(regulation_d.cik.as_str(), "0001547416");
    assert_eq!(regulation_d.name, "NJOY INC");
    assert_eq!(regulation_d.date.to_string(), "2014-02-28 16:00:25");

    for fixture in [CROWDFUNDING, REGULATION_D] {
        let source: Value = serde_json::from_slice(fixture).unwrap();
        let keys = source[0].as_object().unwrap().keys().collect::<Vec<_>>();
        assert_eq!(keys.len(), 3);
        assert!(keys.iter().any(|key| *key == "cik"));
        assert!(keys.iter().any(|key| *key == "name"));
        assert!(keys.iter().any(|key| *key == "date"));
    }
}

#[test]
fn crowdfunding_date_key_is_required_present_nullable_and_raw_when_non_null() {
    let source: Value = serde_json::from_slice(CROWDFUNDING).unwrap();
    let row = source[0].as_object().unwrap();

    let mut missing = row.clone();
    missing.remove("date");
    assert!(
        serde_json::from_value::<CrowdfundingOfferingSearchResult>(Value::Object(missing)).is_err()
    );

    let null =
        serde_json::from_value::<CrowdfundingOfferingSearchResult>(Value::Object(row.clone()))
            .unwrap();
    assert_eq!(null.date, None);

    let mut future_non_null = row.clone();
    future_non_null.insert("date".into(), json!({"provider": ["shape", 1]}));
    let decoded = serde_json::from_value::<CrowdfundingOfferingSearchResult>(Value::Object(
        future_non_null.clone(),
    ))
    .unwrap();
    assert_eq!(decoded.date, Some(json!({"provider": ["shape", 1]})));
    assert_eq!(
        serde_json::to_value(decoded).unwrap(),
        Value::Object(future_non_null)
    );
}

#[test]
fn regulation_d_date_requires_the_exact_documented_datetime_shape() {
    let source: Value = serde_json::from_slice(REGULATION_D).unwrap();
    let row = source[0].as_object().unwrap();

    for invalid in [
        Value::Null,
        json!("2014-02-28"),
        json!("2014-02-28T16:00:25"),
        json!("2014-02-28 16:00:25Z"),
        json!("2014-02-30 16:00:25"),
    ] {
        let mut candidate = row.clone();
        candidate.insert("date".into(), invalid);
        assert!(
            serde_json::from_value::<RegulationDOfferingSearchResult>(Value::Object(candidate))
                .is_err()
        );
    }
}

#[test]
fn required_fields_reject_missing_values_and_unknown_fields_are_tolerated() {
    assert_required::<CrowdfundingOfferingSearchResult>(CROWDFUNDING, true);
    assert_required::<RegulationDOfferingSearchResult>(REGULATION_D, false);
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

fn assert_required<T>(fixture: &[u8], nullable_date: bool)
where
    T: DeserializeOwned,
{
    let source: Value = serde_json::from_slice(fixture).unwrap();
    let row = source[0].as_object().unwrap();
    assert_eq!(row.len(), 3);

    for field in row.keys() {
        let mut missing = row.clone();
        missing.remove(field);
        assert!(serde_json::from_value::<T>(Value::Object(missing)).is_err());

        if field != "date" || !nullable_date {
            let mut null = row.clone();
            null.insert(field.clone(), Value::Null);
            assert!(serde_json::from_value::<T>(Value::Object(null)).is_err());
        }
    }

    let mut future: Map<String, Value> = row.clone();
    future.insert("futureField".into(), json!({"nested": true}));
    assert!(serde_json::from_value::<T>(Value::Object(future)).is_ok());
}
