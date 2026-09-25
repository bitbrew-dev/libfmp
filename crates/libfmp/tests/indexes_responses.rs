use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};

use libfmp::responses::{
    indexes::IndexListing,
    quote::{Quote, QuoteShort},
};

const LIST: &[u8] = include_bytes!("fixtures/indexes_list.json");
const QUOTE: &[u8] = include_bytes!("fixtures/indexes_quote.json");
const QUOTE_SHORT: &[u8] = include_bytes!("fixtures/indexes_quote_short.json");
const QUOTES: &[u8] = include_bytes!("fixtures/indexes_quotes.json");

#[test]
fn every_documented_index_directory_and_quote_fixture_matches_the_typed_wire_contract() {
    assert_exact::<IndexListing>(LIST);
    assert_exact::<Quote>(QUOTE);
    assert_exact::<QuoteShort>(QUOTE_SHORT);
    assert_exact::<QuoteShort>(QUOTES);

    let listing: Vec<IndexListing> = serde_json::from_slice(LIST).unwrap();
    assert_eq!(listing[0].symbol.as_str(), "^TTIN");
    assert_eq!(listing[0].name, "S&P/TSX Capped Industrials Index");
    assert_eq!(listing[0].exchange.as_str(), "TSX");
    assert_eq!(listing[0].currency.as_str(), "CAD");

    let quote: Vec<Quote> = serde_json::from_slice(QUOTE).unwrap();
    assert_eq!(quote[0].symbol.as_str(), "^VIX");
    assert_eq!(quote[0].market_cap, Some(0.0));

    let compact: Vec<QuoteShort> = serde_json::from_slice(QUOTE_SHORT).unwrap();
    assert_eq!(compact[0].symbol.as_str(), "^VIX");
    assert_eq!(compact[0].volume, 0.0);
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
fn index_listing_fields_are_required_non_null_and_open_to_unknown_fields() {
    let source: Value = serde_json::from_slice(LIST).unwrap();
    let row = source[0].clone();

    for field in ["symbol", "name", "exchange", "currency"] {
        let mut missing = row.clone();
        missing.as_object_mut().unwrap().remove(field);
        assert!(serde_json::from_value::<IndexListing>(missing).is_err());

        let mut null = row.clone();
        null[field] = Value::Null;
        assert!(serde_json::from_value::<IndexListing>(null).is_err());
    }

    let mut future = row;
    future["futureField"] = json!({"nested": true});
    let decoded: IndexListing = serde_json::from_value(future).unwrap();
    assert_eq!(decoded.symbol.as_str(), "^TTIN");
}

#[test]
fn index_contracts_preserve_bare_array_roots() {
    assert!(
        serde_json::from_str::<Vec<IndexListing>>("[]")
            .unwrap()
            .is_empty()
    );
    assert!(serde_json::from_str::<Vec<Quote>>("[]").unwrap().is_empty());
    assert!(
        serde_json::from_str::<Vec<QuoteShort>>("[]")
            .unwrap()
            .is_empty()
    );

    let listing: Value = serde_json::from_slice(LIST).unwrap();
    let row = listing[0].clone();
    let multiple = Value::Array(vec![row.clone(), row]);
    assert_eq!(
        serde_json::from_value::<Vec<IndexListing>>(multiple)
            .unwrap()
            .len(),
        2
    );
}
