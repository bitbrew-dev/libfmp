//! Integral-`f64` response fields decode fractional and exponent-form numbers.
//!
//! The provider documents these fields as JSON integers; the synthetic
//! fixtures carry `1.5` and `1.0`-style values that an integer decoder
//! rejects. The repository's JSON formatter rewrites exponent spellings in
//! fixture files, so the `1e9`-style cases are inline. Re-encoding writes an
//! integral value back as a JSON integer.

use libfmp::responses::{
    calendar::{EarningsEvent, StockSplitEvent},
    company::CompanyShareFloat,
    crypto::CryptocurrencyListing,
    fundraising::RegulationDOffering,
    funds::FundDisclosure,
    insider_trading::InsiderTrade,
    screener::CompanyScreenerEntry,
};
use serde_json::json;

const SCREENER: &[u8] = include_bytes!("fixtures/company_screener_fractional_synthetic.json");
const CRYPTO: &[u8] = include_bytes!("fixtures/cryptocurrency_list_fractional_synthetic.json");
const SPLITS: &[u8] = include_bytes!("fixtures/stock_splits_fractional_synthetic.json");
const INSIDER: &[u8] = include_bytes!("fixtures/latest_insider_trades_fractional_synthetic.json");
const DISCLOSURES: &[u8] = include_bytes!("fixtures/fund_disclosures_fractional_synthetic.json");
const FLOAT: &[u8] = include_bytes!("fixtures/company_shares_float_fractional_synthetic.json");
const REG_D: &[u8] = include_bytes!("fixtures/fundraising_by_cik_fractional_synthetic.json");
const EARNINGS: &[u8] = include_bytes!("fixtures/earnings_calendar_fractional_synthetic.json");

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
fn insider_share_quantities_decode_fractional_and_integral_float_forms() {
    let rows: Vec<InsiderTrade> = serde_json::from_slice(INSIDER).unwrap();

    assert_eq!(rows[0].securities_owned, 62_959.5);
    assert_eq!(rows[0].securities_transacted, 1.0);
    let wire = serde_json::to_value(&rows).unwrap();
    assert_eq!(wire[0]["securitiesOwned"], json!(62_959.5));
    assert_eq!(wire[0]["securitiesTransacted"], json!(1_u64));
}

#[test]
fn fund_disclosure_balance_decodes_a_negative_fractional_short() {
    let rows: Vec<FundDisclosure> = serde_json::from_slice(DISCLOSURES).unwrap();

    assert_eq!(rows[0].balance, -2_438_784.5);
    let wire = serde_json::to_value(&rows).unwrap();
    assert_eq!(wire[0]["balance"], json!(-2_438_784.5));
}

#[test]
fn company_share_float_decodes_fractional_and_integral_float_forms() {
    let rows: Vec<CompanyShareFloat> = serde_json::from_slice(FLOAT).unwrap();

    assert_eq!(rows[0].float_shares, 14_662_387_495.5);
    assert_eq!(rows[0].outstanding_shares, 14_687_356_000.0);
    let wire = serde_json::to_value(&rows).unwrap();
    assert_eq!(wire[0]["floatShares"], json!(14_662_387_495.5));
    assert_eq!(wire[0]["outstandingShares"], json!(14_687_356_000_u64));
}

#[test]
fn regulation_d_amounts_decode_fractional_and_integral_float_forms() {
    let rows: Vec<RegulationDOffering> = serde_json::from_slice(REG_D).unwrap();

    assert_eq!(rows[0].total_offering_amount, 71_999_990.5);
    assert_eq!(rows[0].total_amount_sold, 1.0);
    let wire = serde_json::to_value(&rows).unwrap();
    assert_eq!(wire[0]["totalOfferingAmount"], json!(71_999_990.5));
    assert_eq!(wire[0]["totalAmountSold"], json!(1_u64));
}

#[test]
fn earnings_revenue_decodes_fractional_and_integral_float_forms() {
    let rows: Vec<EarningsEvent> = serde_json::from_slice(EARNINGS).unwrap();

    assert_eq!(rows[0].revenue_estimated, 1_086_300_000.5);
    assert_eq!(rows[0].revenue_actual, Some(1_101_500_000.0));
    let wire = serde_json::to_value(&rows).unwrap();
    assert_eq!(wire[0]["revenueEstimated"], json!(1_086_300_000.5));
    assert_eq!(wire[0]["revenueActual"], json!(1_101_500_000_u64));
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
