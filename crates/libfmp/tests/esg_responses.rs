use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Map, Value, json};

use libfmp::responses::esg::{EsgBenchmark, EsgDisclosure, EsgRating};

const DISCLOSURES: &[u8] = include_bytes!("fixtures/esg_disclosures.json");
const RATINGS: &[u8] = include_bytes!("fixtures/esg_ratings.json");
const BENCHMARK: &[u8] = include_bytes!("fixtures/esg_benchmark.json");

#[test]
fn all_documented_esg_fixtures_round_trip_exactly_with_exact_acronym_keys() {
    assert_exact::<EsgDisclosure>(DISCLOSURES);
    assert_exact::<EsgRating>(RATINGS);
    assert_exact::<EsgBenchmark>(BENCHMARK);

    let disclosure = rows::<EsgDisclosure>(DISCLOSURES).remove(0);
    assert_eq!(disclosure.date.to_string(), "2026-03-28");
    assert_eq!(disclosure.accepted_date.to_string(), "2026-04-30");
    assert_eq!(disclosure.symbol.as_str(), "AAPL");
    assert_eq!(disclosure.cik.as_str(), "0000320193");
    assert_eq!(
        disclosure.form_type.as_ref().map(|form| form.as_str()),
        Some("8-K")
    );
    assert_eq!(disclosure.esg_score, 56.79);

    let rating = rows::<EsgRating>(RATINGS).remove(0);
    assert_eq!(rating.industry.as_str(), "CONSUMER ELECTRONICS");
    assert_eq!(rating.fiscal_year.0, 2025);
    assert_eq!(rating.esg_risk_rating, "B");

    let benchmark = rows::<EsgBenchmark>(BENCHMARK).remove(0);
    assert_eq!(benchmark.sector.as_str(), "APPAREL RETAIL");
    assert_eq!(benchmark.fiscal_year.0, 2023);

    let disclosure_json = serde_json::to_value(disclosure).unwrap();
    let rating_json = serde_json::to_value(rating).unwrap();
    let benchmark_json = serde_json::to_value(benchmark).unwrap();
    assert_eq!(disclosure_json["ESGScore"], json!(56.79));
    assert_eq!(rating_json["ESGRiskRating"], json!("B"));
    assert_eq!(benchmark_json["ESGScore"], json!(65.63));
    assert!(disclosure_json.get("esgScore").is_none());
    assert!(rating_json.get("esgRiskRating").is_none());
}

#[test]
fn documented_field_counts_required_non_null_contracts_and_unknown_tolerance_hold() {
    assert_required_contract::<EsgDisclosure>(DISCLOSURES, 11, &["companyName", "formType"]);
    assert_required_contract::<EsgRating>(RATINGS, 7, &["companyName"]);
    assert_required_contract::<EsgBenchmark>(BENCHMARK, 6, &[]);
}

#[test]
fn every_esg_contract_requires_a_bare_array_root() {
    assert!(
        serde_json::from_str::<Vec<EsgDisclosure>>("[]")
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_str::<Vec<EsgRating>>("[]")
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_str::<Vec<EsgBenchmark>>("[]")
            .unwrap()
            .is_empty()
    );
    assert!(serde_json::from_str::<Vec<EsgDisclosure>>("{}").is_err());
    assert!(serde_json::from_str::<Vec<EsgRating>>("{}").is_err());
    assert!(serde_json::from_str::<Vec<EsgBenchmark>>("{}").is_err());
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

fn assert_required_contract<T>(fixture: &[u8], field_count: usize, nullable: &[&str])
where
    T: DeserializeOwned + Serialize,
{
    let source: Value = serde_json::from_slice(fixture).unwrap();
    let row = source[0].as_object().unwrap();
    assert_eq!(row.len(), field_count);

    for field in row.keys() {
        let mut missing = row.clone();
        missing.remove(field);
        assert!(serde_json::from_value::<T>(Value::Object(missing)).is_err());

        let mut null = row.clone();
        null.insert(field.clone(), Value::Null);
        let decoded = serde_json::from_value::<T>(Value::Object(null));
        if nullable.contains(&field.as_str()) {
            assert!(serde_json::to_value(decoded.unwrap()).unwrap()[field].is_null());
        } else {
            assert!(decoded.is_err(), "field {field} must reject null");
        }
    }

    let mut future: Map<String, Value> = row.clone();
    future.insert("futureField".into(), json!({"nested": true}));
    assert!(serde_json::from_value::<T>(Value::Object(future)).is_ok());
}
