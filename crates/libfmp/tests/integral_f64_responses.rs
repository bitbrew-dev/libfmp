//! Integral-`f64` response fields decode fractional and exponent-form numbers.
//!
//! The provider documents these fields as JSON integers; the synthetic
//! fixtures carry `1.5` and `1.0`-style values that an integer decoder
//! rejects. The repository's JSON formatter rewrites exponent spellings in
//! fixture files, so the `1e9`-style cases are inline. Re-encoding writes an
//! integral value back as a JSON integer.

use libfmp::responses::{
    calendar::StockSplitEvent, crypto::CryptocurrencyListing, screener::CompanyScreenerEntry,
};
use serde_json::json;

const SCREENER: &[u8] = include_bytes!("fixtures/company_screener_fractional_synthetic.json");
const CRYPTO: &[u8] = include_bytes!("fixtures/cryptocurrency_list_fractional_synthetic.json");
const SPLITS: &[u8] = include_bytes!("fixtures/stock_splits_fractional_synthetic.json");

#[test]
fn market_cap_decodes_integral_float_and_fractional_forms() {
    let rows: Vec<CompanyScreenerEntry> = serde_json::from_slice(SCREENER).unwrap();

    assert_eq!(rows[0].market_cap, 4_885_602_246_714.0);
    assert_eq!(rows[1].market_cap, 1_234_567.5);
    assert_eq!(rows[1].volume, 1.0);
    let wire = serde_json::to_value(&rows).unwrap();
    assert_eq!(wire[0]["marketCap"], json!(4_885_602_246_714_u64));
    assert_eq!(wire[1]["marketCap"], json!(1_234_567.5));
    assert_eq!(wire[1]["volume"], json!(1_u64));
}

#[test]
fn crypto_supply_decodes_integral_float_and_fractional_forms() {
    let rows: Vec<CryptocurrencyListing> = serde_json::from_slice(CRYPTO).unwrap();

    assert_eq!(rows[0].circulating_supply, 4_232_705_124.5);
    assert_eq!(rows[0].total_supply, 4_788_606_639.0);
    let wire = serde_json::to_value(&rows).unwrap();
    assert_eq!(wire[0]["circulatingSupply"], json!(4_232_705_124.5));
    assert_eq!(wire[0]["totalSupply"], json!(4_788_606_639_u64));
}

#[test]
fn split_terms_decode_fractional_and_integral_float_forms() {
    let rows: Vec<StockSplitEvent> = serde_json::from_slice(SPLITS).unwrap();

    assert_eq!(rows[0].numerator, 1.5);
    assert_eq!(rows[0].denominator, 1.0);
    let wire = serde_json::to_value(&rows).unwrap();
    assert_eq!(wire[0]["numerator"], json!(1.5));
    assert_eq!(wire[0]["denominator"], json!(1_u64));
}

#[test]
fn exponent_form_numbers_decode_into_every_integral_f64_alias() {
    let screener: CompanyScreenerEntry = serde_json::from_value(json!({
        "symbol": "AAPL", "companyName": "Apple Inc.", "marketCap": 4.885602246714e12,
        "sector": "Technology", "industry": "Consumer Electronics", "beta": 1.097,
        "price": 332.64001, "lastAnnualDividend": 1.05, "volume": 2.9909012e7,
        "exchange": "NASDAQ Global Select", "exchangeShortName": "NASDAQ", "country": "US",
        "isEtf": false, "isFund": false, "isActivelyTrading": true
    }))
    .unwrap();
    assert_eq!(screener.market_cap, 4_885_602_246_714.0);
    assert_eq!(screener.volume, 29_909_012.0);

    let crypto: CryptocurrencyListing = serde_json::from_str(
        r#"{"symbol":"MIOTAUSD","name":"IOTA USD","exchange":"CCC","icoDate":"2017-11-09",
            "circulatingSupply":4.2327051245e9,"totalSupply":4.788606639e9}"#,
    )
    .unwrap();
    assert_eq!(crypto.circulating_supply, 4_232_705_124.5);
    assert_eq!(crypto.total_supply, 4_788_606_639.0);
    assert_eq!(
        serde_json::to_value(&crypto).unwrap()["totalSupply"],
        json!(4_788_606_639_u64)
    );

    let split: StockSplitEvent = serde_json::from_str(
        r#"{"symbol":"AAPL","date":"2020-08-31","numerator":4e0,"denominator":1E0,"splitType":"stock-split"}"#,
    )
    .unwrap();
    assert_eq!((split.numerator, split.denominator), (4.0, 1.0));
}
