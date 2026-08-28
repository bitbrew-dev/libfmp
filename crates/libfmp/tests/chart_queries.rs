use std::str::FromStr;

use libfmp::{
    endpoints::chart::{StockChartEodQuery, StockChartIntradayQuery},
    types::{Date, Ticker},
};

#[test]
fn public_query_builders_and_conversions_preserve_independent_options() {
    let symbol = Ticker::new("AAPL").unwrap();
    let from = Date::from_str("2024-01-01").unwrap();
    let to = Date::from_str("2024-03-01").unwrap();

    let eod: StockChartEodQuery = (&symbol).into();
    assert_eq!(eod.symbol(), &symbol);
    assert_eq!(eod.from(), None);
    assert_eq!(eod.to(), None);
    assert_eq!(
        StockChartEodQuery::new(symbol.clone()).with_from(from).to(),
        None
    );
    assert_eq!(
        StockChartEodQuery::new(symbol.clone()).with_to(to).from(),
        None
    );

    let intraday: StockChartIntradayQuery = symbol.into();
    assert_eq!(intraday.from(), None);
    assert_eq!(intraday.to(), None);
    assert_eq!(intraday.nonadjusted(), None);
    assert_eq!(intraday.extended(), None);

    let explicit = intraday
        .with_from(from)
        .with_to(to)
        .with_nonadjusted(false)
        .with_extended(false);
    assert_eq!(explicit.from(), Some(from));
    assert_eq!(explicit.to(), Some(to));
    assert_eq!(explicit.nonadjusted(), Some(false));
    assert_eq!(explicit.extended(), Some(false));
}
