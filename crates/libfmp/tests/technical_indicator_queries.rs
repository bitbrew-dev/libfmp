use std::str::FromStr;

use libfmp::{
    endpoints::technical_indicators::TechnicalIndicatorQuery,
    query::{ChartTimeframe, PeriodLength},
    types::{Date, Ticker},
};

#[test]
fn public_query_preserves_required_values_and_independent_dates() {
    let symbol = Ticker::new("AAPL").unwrap();
    let period = PeriodLength::new(10).unwrap();
    let query = TechnicalIndicatorQuery::new(symbol.clone(), period, ChartTimeframe::OneDay);

    assert_eq!(query.symbol(), &symbol);
    assert_eq!(query.period_length(), period);
    assert_eq!(query.timeframe(), ChartTimeframe::OneDay);
    assert_eq!(query.from(), None);
    assert_eq!(query.to(), None);

    let from = Date::from_str("2026-06-01").unwrap();
    let to = Date::from_str("2026-03-01").unwrap();
    let reversed = query.with_from(from).with_to(to);
    assert_eq!(reversed.from(), Some(from));
    assert_eq!(reversed.to(), Some(to));
}

#[test]
fn period_length_rejects_zero_and_preserves_the_full_positive_domain() {
    assert!(PeriodLength::new(0).is_none());
    assert_eq!(PeriodLength::new(1).unwrap().get(), 1);
    assert_eq!(PeriodLength::new(u32::MAX).unwrap().get(), u32::MAX);
}

#[test]
fn every_documented_timeframe_has_its_exact_wire_value() {
    let values = [
        (ChartTimeframe::OneMinute, "1min"),
        (ChartTimeframe::FiveMinutes, "5min"),
        (ChartTimeframe::FifteenMinutes, "15min"),
        (ChartTimeframe::ThirtyMinutes, "30min"),
        (ChartTimeframe::OneHour, "1hour"),
        (ChartTimeframe::FourHours, "4hour"),
        (ChartTimeframe::OneDay, "1day"),
    ];
    for (timeframe, wire) in values {
        assert_eq!(timeframe.to_string(), wire);
    }
}
