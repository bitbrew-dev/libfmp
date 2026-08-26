use libfmp::{
    responses::quote::{Quote, QuoteShort},
    types::{ExchangeCode, Ticker, UnixSeconds},
};

#[test]
fn documented_full_quote_decodes_exact_wire_values() {
    let quotes: Vec<Quote> = serde_json::from_str(include_str!("fixtures/quote.json")).unwrap();

    assert_eq!(quotes.len(), 1);
    let quote = &quotes[0];
    assert_eq!(quote.symbol, Ticker::new("AAPL").unwrap());
    assert_eq!(quote.name, "Apple Inc.");
    assert_eq!(quote.price, 331.85501);
    assert_eq!(quote.change_percentage, -1.8732);
    assert_eq!(quote.change, -6.33498);
    assert_eq!(quote.volume, 28_718_014);
    assert_eq!(quote.day_low, 329.59);
    assert_eq!(quote.day_high, 334.48);
    assert_eq!(quote.year_high, 344.57);
    assert_eq!(quote.year_low, 201.5);
    assert_eq!(quote.market_cap, Some(4_874_072_686_740));
    assert_eq!(quote.price_avg_50, 308.5888);
    assert_eq!(quote.price_avg_200, 277.21344);
    assert_eq!(quote.exchange, ExchangeCode::new("NASDAQ").unwrap());
    assert_eq!(quote.open, 333.13);
    assert_eq!(quote.previous_close, 338.18999);
    assert_eq!(quote.timestamp, UnixSeconds(1_785_430_812));

    let encoded = serde_json::to_value(quote).unwrap();
    assert_eq!(encoded["changePercentage"], -1.8732);
    assert_eq!(encoded["marketCap"], 4_874_072_686_740_u64);
    assert_eq!(encoded["priceAvg50"], 308.5888);
    assert_eq!(encoded["priceAvg200"], 277.21344);
    assert_eq!(encoded["previousClose"], 338.18999);
    assert!(encoded.get("change_percentage").is_none());
}

#[test]
fn documented_commodity_quote_accepts_null_market_cap() {
    let quotes: Vec<Quote> =
        serde_json::from_str(include_str!("fixtures/quote_commodity.json")).unwrap();

    assert_eq!(quotes.len(), 1);
    assert_eq!(quotes[0].symbol.as_str(), "GCUSD");
    assert_eq!(quotes[0].exchange.as_str(), "COMMODITY");
    assert_eq!(quotes[0].market_cap, None);
}

#[test]
fn documented_short_quote_preserves_negative_change() {
    let quotes: Vec<QuoteShort> =
        serde_json::from_str(include_str!("fixtures/quote_short.json")).unwrap();

    assert_eq!(quotes.len(), 1);
    assert_eq!(quotes[0].symbol.as_str(), "AAPL");
    assert_eq!(quotes[0].price, 331.85501);
    assert_eq!(quotes[0].change, -6.33498);
    assert_eq!(quotes[0].volume, 28_718_014);
}

#[test]
fn bare_quote_arrays_retain_empty_and_multiple_shapes() {
    let empty: Vec<QuoteShort> =
        serde_json::from_str(include_str!("fixtures/quote_short_empty.json")).unwrap();
    let multiple: Vec<QuoteShort> =
        serde_json::from_str(include_str!("fixtures/quote_short_multiple.json")).unwrap();

    assert!(empty.is_empty());
    assert_eq!(multiple.len(), 2);
    assert_eq!(multiple[0].symbol.as_str(), "000001.SZ");
    assert_eq!(multiple[0].volume, 4_294_967_296);
    assert_eq!(multiple[1].symbol.as_str(), "^VIX");
}
