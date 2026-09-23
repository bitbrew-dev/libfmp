#[path = "support/exact_json.rs"]
mod exact_json;

use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};

use libfmp::{
    responses::{
        commodities::{
            CommodityListing, Quote as CommodityQuote, QuoteShort as CommodityQuoteShort,
        },
        crypto::{
            CryptocurrencyListing, Quote as CryptocurrencyQuote,
            QuoteShort as CryptocurrencyQuoteShort,
        },
        forex::{ForexPair, Quote as ForexQuote, QuoteShort as ForexQuoteShort},
    },
    types::Date,
};

use exact_json::{assert_json_wire_equivalent, assert_json_wire_equivalent_with_f64_fields};

const QUOTE_F64_FIELDS: &[&str] = &[
    "price",
    "changePercentage",
    "change",
    "dayLow",
    "dayHigh",
    "yearHigh",
    "yearLow",
    "priceAvg50",
    "priceAvg200",
    "open",
    "previousClose",
];

const COMMODITIES_LIST: &[u8] = include_bytes!("fixtures/commodities_list.json");
const COMMODITY_QUOTE: &[u8] = include_bytes!("fixtures/commodities_quote.json");
const COMMODITY_QUOTE_SHORT: &[u8] = include_bytes!("fixtures/commodities_quote_short.json");
const COMMODITY_QUOTES: &[u8] = include_bytes!("fixtures/commodities_quotes.json");
const FOREX_LIST: &[u8] = include_bytes!("fixtures/forex_list.json");
const FOREX_QUOTE: &[u8] = include_bytes!("fixtures/forex_quote.json");
const FOREX_QUOTE_SHORT: &[u8] = include_bytes!("fixtures/forex_quote_short.json");
const FOREX_QUOTES: &[u8] = include_bytes!("fixtures/forex_quotes.json");
const CRYPTOCURRENCY_LIST: &[u8] = include_bytes!("fixtures/cryptocurrency_list.json");
const CRYPTOCURRENCY_QUOTE: &[u8] = include_bytes!("fixtures/cryptocurrency_quote.json");
const CRYPTOCURRENCY_QUOTE_SHORT: &[u8] =
    include_bytes!("fixtures/cryptocurrency_quote_short.json");
const CRYPTOCURRENCY_QUOTES: &[u8] = include_bytes!("fixtures/cryptocurrency_quotes.json");

#[test]
fn every_documented_catalog_and_quote_fixture_matches_the_typed_wire_contract_exactly() {
    assert_exact::<CommodityListing>(COMMODITIES_LIST);
    assert_exact::<CommodityQuote>(COMMODITY_QUOTE);
    assert_exact::<CommodityQuoteShort>(COMMODITY_QUOTE_SHORT);
    assert_exact::<CommodityQuoteShort>(COMMODITY_QUOTES);

    assert_exact::<ForexPair>(FOREX_LIST);
    assert_exact::<ForexQuote>(FOREX_QUOTE);
    assert_exact::<ForexQuoteShort>(FOREX_QUOTE_SHORT);
    assert_exact::<ForexQuoteShort>(FOREX_QUOTES);

    assert_exact::<CryptocurrencyListing>(CRYPTOCURRENCY_LIST);
    assert_exact::<CryptocurrencyQuote>(CRYPTOCURRENCY_QUOTE);
    assert_exact::<CryptocurrencyQuoteShort>(CRYPTOCURRENCY_QUOTE_SHORT);
    assert_exact::<CryptocurrencyQuoteShort>(CRYPTOCURRENCY_QUOTES);
}

fn assert_exact<T>(fixture: &[u8])
where
    T: DeserializeOwned + Serialize,
{
    let source: serde_json::Value = serde_json::from_slice(fixture).unwrap();
    let rows: Vec<T> = serde_json::from_slice(fixture).unwrap();
    assert_json_wire_equivalent_with_f64_fields(
        &serde_json::to_value(rows).unwrap(),
        &source,
        QUOTE_F64_FIELDS,
    );
}

#[test]
fn exact_wire_comparison_preserves_f64_semantics_without_masking_precision_loss() {
    let integer_kind = json!({"volume": 1});
    let float_kind = json!({"volume": 1.0});
    assert!(
        std::panic::catch_unwind(|| {
            assert_json_wire_equivalent(&float_kind, &integer_kind);
        })
        .is_err()
    );

    assert_json_wire_equivalent_with_f64_fields(
        &json!({"price": 1.0}),
        &json!({"price": 1}),
        QUOTE_F64_FIELDS,
    );

    let rounded_integer = json!({"price": 9_007_199_254_740_992_u64});
    let adjacent_integer = json!({"price": 9_007_199_254_740_993_u64});
    assert!(
        std::panic::catch_unwind(|| {
            assert_json_wire_equivalent_with_f64_fields(
                &rounded_integer,
                &adjacent_integer,
                QUOTE_F64_FIELDS,
            );
        })
        .is_err()
    );

    let rounded_float: serde_json::Value =
        serde_json::from_str(r#"{"price":9007199254740992.0}"#).unwrap();
    assert!(
        std::panic::catch_unwind(|| {
            assert_json_wire_equivalent_with_f64_fields(
                &rounded_float,
                &adjacent_integer,
                QUOTE_F64_FIELDS,
            );
        })
        .is_err()
    );

    let out_of_range_float: serde_json::Value = serde_json::from_str(r#"{"price":1e400}"#).unwrap();
    let different_out_of_range_float: serde_json::Value =
        serde_json::from_str(r#"{"price":2e400}"#).unwrap();
    assert_eq!(out_of_range_float["price"].as_f64(), None);
    assert_eq!(different_out_of_range_float["price"].as_f64(), None);
    assert!(
        std::panic::catch_unwind(|| {
            assert_json_wire_equivalent_with_f64_fields(
                &out_of_range_float,
                &different_out_of_range_float,
                QUOTE_F64_FIELDS,
            );
        })
        .is_err()
    );
}

#[test]
fn catalog_rows_preserve_exact_identifiers_dates_and_large_unsigned_supplies() {
    let commodities: Vec<CommodityListing> = serde_json::from_slice(COMMODITIES_LIST).unwrap();
    assert_eq!(commodities[0].symbol.as_str(), "ZMUSD");
    assert_eq!(commodities[0].exchange, None);
    assert_eq!(commodities[0].trade_month, "Dec");
    assert_eq!(commodities[0].currency.as_str(), "USD");

    let forex: Vec<ForexPair> = serde_json::from_slice(FOREX_LIST).unwrap();
    assert_eq!(forex[0].symbol.as_str(), "ARSMXN");
    assert_eq!(forex[0].from_currency.as_str(), "ARS");
    assert_eq!(forex[0].to_currency.as_str(), "MXN");

    let cryptocurrencies: Vec<CryptocurrencyListing> =
        serde_json::from_slice(CRYPTOCURRENCY_LIST).unwrap();
    assert_eq!(
        cryptocurrencies[0].ico_date,
        "2017-11-09".parse::<Date>().unwrap()
    );
    assert_eq!(cryptocurrencies[0].circulating_supply, 4_232_705_124);
    assert_eq!(cryptocurrencies[0].total_supply, 4_788_606_639);
}

#[test]
fn commodity_exchange_is_required_nullable_and_future_non_null_values_remain_open() {
    let source: Value = serde_json::from_slice(COMMODITIES_LIST).unwrap();
    let row = source[0].clone();

    let decoded: CommodityListing = serde_json::from_value(row.clone()).unwrap();
    assert_eq!(decoded.exchange, None);

    let mut missing = row.clone();
    missing.as_object_mut().unwrap().remove("exchange");
    assert!(serde_json::from_value::<CommodityListing>(missing).is_err());

    let mut future_non_null = row;
    future_non_null["exchange"] = json!("FUTURES");
    future_non_null["futureField"] = json!({"nested": true});
    let decoded: CommodityListing = serde_json::from_value(future_non_null).unwrap();
    assert_eq!(decoded.exchange.unwrap().as_str(), "FUTURES");
}

#[test]
fn catalog_non_nullable_fields_are_required_and_unknown_fields_are_accepted() {
    assert_required_non_null::<CommodityListing>(
        COMMODITIES_LIST,
        &["symbol", "name", "tradeMonth", "currency"],
    );
    assert_required_non_null::<ForexPair>(
        FOREX_LIST,
        &["symbol", "fromCurrency", "toCurrency", "fromName", "toName"],
    );
    assert_required_non_null::<CryptocurrencyListing>(
        CRYPTOCURRENCY_LIST,
        &[
            "symbol",
            "name",
            "exchange",
            "icoDate",
            "circulatingSupply",
            "totalSupply",
        ],
    );
}

fn assert_required_non_null<T>(fixture: &[u8], fields: &[&str])
where
    T: DeserializeOwned,
{
    let source: Value = serde_json::from_slice(fixture).unwrap();
    let row = source[0].clone();
    for field in fields {
        let mut missing = row.clone();
        missing.as_object_mut().unwrap().remove(*field);
        assert!(
            serde_json::from_value::<T>(missing).is_err(),
            "missing {field}"
        );

        let mut null = row.clone();
        null[*field] = Value::Null;
        assert!(serde_json::from_value::<T>(null).is_err(), "null {field}");
    }

    let mut future = row;
    future["futureField"] = json!([1, {"open": true}]);
    assert!(serde_json::from_value::<T>(future).is_ok());
}

#[test]
fn crypto_quotes_preserve_large_volume_market_cap_and_nullable_market_cap() {
    let crypto: Vec<CryptocurrencyQuote> = serde_json::from_slice(CRYPTOCURRENCY_QUOTE).unwrap();
    assert_eq!(crypto[0].volume, 32_030_003_200.0);
    assert_eq!(crypto[0].market_cap, Some(1_293_361_815_015));

    let commodity: Vec<CommodityQuote> = serde_json::from_slice(COMMODITY_QUOTE).unwrap();
    let forex: Vec<ForexQuote> = serde_json::from_slice(FOREX_QUOTE).unwrap();
    assert_eq!(commodity[0].market_cap, None);
    assert_eq!(forex[0].market_cap, None);

    let compact: Vec<CryptocurrencyQuoteShort> =
        serde_json::from_slice(CRYPTOCURRENCY_QUOTE_SHORT).unwrap();
    assert_eq!(compact[0].volume, 32_030_003_200.0);
}

#[test]
fn every_catalog_and_quote_contract_preserves_bare_array_roots() {
    assert!(
        serde_json::from_str::<Vec<CommodityListing>>("[]")
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_str::<Vec<ForexPair>>("[]")
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_str::<Vec<CryptocurrencyListing>>("[]")
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_str::<Vec<CommodityQuote>>("[]")
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_str::<Vec<ForexQuoteShort>>("[]")
            .unwrap()
            .is_empty()
    );
}
