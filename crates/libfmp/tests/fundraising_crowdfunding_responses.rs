use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Map, Value, json};

use libfmp::{codecs::YnFlag, responses::fundraising::CrowdfundingOffering};

const LATEST: &[u8] = include_bytes!("fixtures/crowdfunding_offerings_latest.json");
const BY_CIK: &[u8] = include_bytes!("fixtures/crowdfunding_offerings_by_cik.json");

#[test]
fn both_exact_outer_md_fixtures_round_trip_as_the_shared_48_key_row() {
    assert_exact::<CrowdfundingOffering>(LATEST);
    assert_exact::<CrowdfundingOffering>(BY_CIK);

    for fixture in [LATEST, BY_CIK] {
        let source: Value = serde_json::from_slice(fixture).unwrap();
        let row = source[0].as_object().unwrap();
        assert_eq!(row.len(), 48);
        assert_eq!(
            row.keys().collect::<std::collections::HashSet<_>>().len(),
            48
        );
        assert!(row.contains_key("cashAndCashEquiValentMostRecentFiscalYear"));
        assert!(row.contains_key("cashAndCashEquiValentPriorFiscalYear"));
        assert!(!row.contains_key("cashAndCashEquivalentMostRecentFiscalYear"));
    }
}

#[test]
fn typed_fields_preserve_dates_ciks_flags_prices_counts_and_signed_financials() {
    let latest = rows::<CrowdfundingOffering>(LATEST).remove(0);
    assert_eq!(latest.cik.as_str(), "0001621902");
    assert_eq!(latest.intermediary_commission_cik.as_str(), "0001669191");
    assert_eq!(latest.date.to_string(), "11-22-2011");
    assert_eq!(latest.filing_date.to_string(), "2026-07-30 00:00:00");
    assert_eq!(latest.accepted_date.to_string(), "2026-07-30 12:54:38");
    assert_eq!(latest.offering_deadline_date.to_string(), "10-31-2026");
    assert_eq!(latest.form_type.as_str(), "C/A");
    assert_eq!(latest.over_subscription_accepted, YnFlag::True);
    assert_eq!(latest.number_of_security_offered, 100_000.0);
    assert_eq!(latest.current_number_of_employees, 5);
    assert_eq!(latest.offering_price.to_string(), "0.1");
    assert_eq!(latest.net_income_most_recent_fiscal_year, -152_577.0);
    assert_eq!(latest.net_income_prior_fiscal_year, -105_631.0);
    assert_eq!(latest.security_offered_other_description, None);

    let by_cik = rows::<CrowdfundingOffering>(BY_CIK).remove(0);
    assert_eq!(by_cik.cik.as_str(), "0001916078");
    assert_eq!(by_cik.intermediary_commission_cik.as_str(), "0001665160");
    assert_eq!(by_cik.offering_price.to_string(), "2");
    assert_eq!(by_cik.net_income_most_recent_fiscal_year, -964_551.0);
    assert_eq!(by_cik.net_income_prior_fiscal_year, -10_860.0);
    assert_eq!(
        by_cik.security_offered_other_description.as_deref(),
        Some("Non-Voting Common Stock")
    );
}

#[test]
fn security_description_is_required_present_nullable_and_all_other_keys_are_non_null() {
    for fixture in [LATEST, BY_CIK] {
        let source: Value = serde_json::from_slice(fixture).unwrap();
        let row = source[0].as_object().unwrap();

        for field in row.keys() {
            let mut missing = row.clone();
            missing.remove(field);
            assert!(
                serde_json::from_value::<CrowdfundingOffering>(Value::Object(missing)).is_err(),
                "field {field} must be present"
            );

            let mut null = row.clone();
            null.insert(field.clone(), Value::Null);
            if field == "securityOfferedOtherDescription" {
                assert!(
                    serde_json::from_value::<CrowdfundingOffering>(Value::Object(null)).is_ok()
                );
            } else {
                assert!(
                    serde_json::from_value::<CrowdfundingOffering>(Value::Object(null)).is_err(),
                    "field {field} must reject null"
                );
            }
        }
    }
}

#[test]
fn exact_temporal_flag_and_numeric_wire_shapes_are_enforced() {
    let source: Value = serde_json::from_slice(LATEST).unwrap();
    let row = source[0].as_object().unwrap();

    for (field, invalid_values) in [
        ("date", vec![json!("2011-11-22"), json!("11/22/2011")]),
        (
            "offeringDeadlineDate",
            vec![json!("2026-10-31"), json!("10-31-2026 00:00:00")],
        ),
        (
            "filingDate",
            vec![json!("2026-07-30"), json!("2026-07-30T00:00:00")],
        ),
        (
            "acceptedDate",
            vec![json!("2026-07-30"), json!("2026-07-30T12:54:38")],
        ),
    ] {
        for invalid in invalid_values {
            let mut candidate = row.clone();
            candidate.insert(field.into(), invalid);
            assert!(
                serde_json::from_value::<CrowdfundingOffering>(Value::Object(candidate)).is_err(),
                "field {field} accepted the wrong date shape"
            );
        }
    }

    for invalid in [
        json!(true),
        json!(false),
        json!("Yes"),
        json!("N/A"),
        json!("y"),
    ] {
        let mut candidate = row.clone();
        candidate.insert("overSubscriptionAccepted".into(), invalid);
        assert!(serde_json::from_value::<CrowdfundingOffering>(Value::Object(candidate)).is_err());
    }
    for valid in ["Y", "N"] {
        let mut candidate = row.clone();
        candidate.insert("overSubscriptionAccepted".into(), json!(valid));
        assert!(serde_json::from_value::<CrowdfundingOffering>(Value::Object(candidate)).is_ok());
    }

    let mut negative = row.clone();
    negative.insert("currentNumberOfEmployees".into(), json!(-1));
    assert!(serde_json::from_value::<CrowdfundingOffering>(Value::Object(negative)).is_err());

    let mut fractional = row.clone();
    fractional.insert("numberOfSecurityOffered".into(), json!(2_500.5));
    let offering: CrowdfundingOffering = serde_json::from_value(Value::Object(fractional)).unwrap();
    assert_eq!(offering.number_of_security_offered, 2_500.5);
}

#[test]
fn unknown_fields_are_tolerated_and_bare_arrays_are_required() {
    let source: Value = serde_json::from_slice(LATEST).unwrap();
    let mut row: Map<String, Value> = source[0].as_object().unwrap().clone();
    row.insert("futureField".into(), json!({"nested": [1, true]}));
    assert!(serde_json::from_value::<CrowdfundingOffering>(Value::Object(row)).is_ok());

    assert!(
        serde_json::from_str::<Vec<CrowdfundingOffering>>("[]")
            .unwrap()
            .is_empty()
    );
    assert!(serde_json::from_str::<Vec<CrowdfundingOffering>>("{}").is_err());
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
