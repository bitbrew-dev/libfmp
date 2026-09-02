use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};

use libfmp::responses::dcf::DcfValuation;

const STANDARD: &[u8] = include_bytes!("fixtures/discounted_cash_flow.json");
const LEVERED: &[u8] = include_bytes!("fixtures/levered_discounted_cash_flow.json");

#[test]
fn both_outer_fixtures_round_trip_exactly() {
    assert_exact::<DcfValuation>(STANDARD);
    assert_exact::<DcfValuation>(LEVERED);

    let standard: Vec<DcfValuation> = serde_json::from_slice(STANDARD).unwrap();
    let levered: Vec<DcfValuation> = serde_json::from_slice(LEVERED).unwrap();

    assert_eq!(standard[0].symbol.as_str(), "AAPL");
    assert_eq!(standard[0].date.to_string(), "2026-07-30");
    assert_eq!(standard[0].dcf, 147.10881272667325);
    assert_eq!(standard[0].stock_price, 338.19);
    assert_eq!(levered[0].dcf, 140.6429495133426);
    assert_eq!(levered[0].stock_price, 338.19);
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
fn every_documented_field_is_required_non_null_and_unknown_fields_are_accepted() {
    let source: Value = serde_json::from_slice(STANDARD).unwrap();
    let row = source[0].clone();

    for field in ["symbol", "date", "dcf", "Stock Price"] {
        let mut missing = row.clone();
        missing.as_object_mut().unwrap().remove(field);
        assert!(serde_json::from_value::<DcfValuation>(missing).is_err());

        let mut null = row.clone();
        null[field] = Value::Null;
        assert!(serde_json::from_value::<DcfValuation>(null).is_err());
    }

    let mut future = row;
    future["futureField"] = json!({"nested": true});
    let decoded: DcfValuation = serde_json::from_value(future).unwrap();
    assert_eq!(decoded.symbol.as_str(), "AAPL");
}

#[test]
fn dcf_contract_preserves_bare_array_roots_and_exact_stock_price_key() {
    assert!(
        serde_json::from_str::<Vec<DcfValuation>>("[]")
            .unwrap()
            .is_empty()
    );
    assert!(serde_json::from_str::<Vec<DcfValuation>>("{}").is_err());

    let row: DcfValuation = serde_json::from_slice::<Value>(STANDARD)
        .and_then(|value| serde_json::from_value(value[0].clone()))
        .unwrap();
    let encoded = serde_json::to_value(row).unwrap();
    assert_eq!(encoded["Stock Price"], json!(338.19));
    assert!(encoded.get("stockPrice").is_none());
    assert!(encoded.get("stock_price").is_none());
}
