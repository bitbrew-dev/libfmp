use libfmp::{
    responses::quote::{AftermarketQuote, AftermarketTrade, Quote, QuoteShort, StockPriceChange},
    types::{ExchangeCode, Ticker, UnixMilliseconds, UnixSeconds},
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

#[test]
fn documented_aftermarket_trade_decodes_exact_wire_values_in_milliseconds() {
    let trades: Vec<AftermarketTrade> =
        serde_json::from_str(include_str!("fixtures/aftermarket_trade.json")).unwrap();

    assert_eq!(trades.len(), 1);
    let trade = &trades[0];
    assert_eq!(trade.symbol, Ticker::new("AAPL").unwrap());
    assert_eq!(trade.price, 331.85999);
    assert_eq!(trade.trade_size, 16);
    assert_eq!(trade.timestamp, UnixMilliseconds(1_785_430_813_000));

    let wire = serde_json::to_value(trade).unwrap();
    assert_eq!(wire["tradeSize"], 16);
    assert_eq!(wire["timestamp"], 1_785_430_813_000_i64);
    assert!(wire.get("trade_size").is_none());
}

#[test]
fn documented_aftermarket_quote_decodes_exact_wire_values_in_milliseconds() {
    let quotes: Vec<AftermarketQuote> =
        serde_json::from_str(include_str!("fixtures/aftermarket_quote.json")).unwrap();

    assert_eq!(quotes.len(), 1);
    let quote = &quotes[0];
    assert_eq!(quote.symbol, Ticker::new("AAPL").unwrap());
    assert_eq!(quote.bid_size, 16);
    assert_eq!(quote.bid_price, 331.85);
    assert_eq!(quote.ask_size, 40);
    assert_eq!(quote.ask_price, 331.88);
    assert_eq!(quote.volume, 28_718_455);
    assert_eq!(quote.timestamp, UnixMilliseconds(1_785_430_813_000));

    let wire = serde_json::to_value(quote).unwrap();
    assert_eq!(wire["bidSize"], 16);
    assert_eq!(wire["bidPrice"], 331.85);
    assert_eq!(wire["askSize"], 40);
    assert_eq!(wire["askPrice"], 331.88);
    assert!(wire.get("bid_size").is_none());
}

#[test]
fn documented_stock_price_change_maps_numeric_period_keys_exactly() {
    let changes: Vec<StockPriceChange> =
        serde_json::from_str(include_str!("fixtures/stock_price_change.json")).unwrap();

    assert_eq!(changes.len(), 1);
    let change = &changes[0];
    assert_eq!(change.symbol, Ticker::new("AAPL").unwrap());
    assert_eq!(change.one_day, -1.8732);
    assert_eq!(change.five_days, 3.12782);
    assert_eq!(change.one_month, 14.68586);
    assert_eq!(change.three_months, 22.29777);
    assert_eq!(change.six_months, 27.89233);
    assert_eq!(change.year_to_date, 22.06835);
    assert_eq!(change.one_year, 58.74432);
    assert_eq!(change.three_years, 68.92594);
    assert_eq!(change.five_years, 127.51612);
    assert_eq!(change.ten_years, 1151.81068);
    assert_eq!(change.max, 258454.74094);

    let wire = serde_json::to_value(change).unwrap();
    for key in [
        "1D", "5D", "1M", "3M", "6M", "ytd", "1Y", "3Y", "5Y", "10Y", "max",
    ] {
        assert!(wire.get(key).is_some(), "missing period key {key}");
    }
    assert!(wire.get("one_day").is_none());
    assert!(wire.get("year_to_date").is_none());
    assert!(wire.get("ten_years").is_none());
}

#[test]
fn new_quote_response_arrays_preserve_empty_multiple_unknown_and_large_values() {
    assert!(
        serde_json::from_str::<Vec<AftermarketTrade>>(include_str!("fixtures/quote_empty.json"))
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_str::<Vec<AftermarketQuote>>(include_str!("fixtures/quote_empty.json"))
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_str::<Vec<StockPriceChange>>(include_str!("fixtures/quote_empty.json"))
            .unwrap()
            .is_empty()
    );

    let trades: Vec<AftermarketTrade> =
        serde_json::from_str(include_str!("fixtures/aftermarket_trade_synthetic.json")).unwrap();
    assert_eq!(trades.len(), 2);
    assert_eq!(trades[0].trade_size, u64::MAX);
    assert_eq!(trades[0].timestamp, UnixMilliseconds(i64::MAX));
    assert_eq!(trades[1].trade_size, 9_007_199_254_740_993);

    let quotes: Vec<AftermarketQuote> =
        serde_json::from_str(include_str!("fixtures/aftermarket_quote_synthetic.json")).unwrap();
    assert_eq!(quotes.len(), 2);
    assert_eq!(quotes[0].bid_size, u64::MAX);
    assert_eq!(quotes[0].ask_size, 9_007_199_254_740_993);
    assert_eq!(quotes[0].volume, u64::MAX - 1);
    assert_eq!(quotes[0].timestamp, UnixMilliseconds(i64::MAX));
    assert_eq!(quotes[1].volume, 9_007_199_254_740_993);

    let changes: Vec<StockPriceChange> =
        serde_json::from_str(include_str!("fixtures/stock_price_change_synthetic.json")).unwrap();
    assert_eq!(changes.len(), 2);
    assert_eq!(changes[0].symbol.as_str(), "BRK.B");
    assert_eq!(changes[1].symbol.as_str(), "000001.SZ");
}
