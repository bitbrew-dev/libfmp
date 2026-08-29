use std::str::FromStr;

use libfmp::{
    endpoints::calendar::{
        DividendsCalendarQuery, DividendsQuery, EarningsCalendarQuery, EarningsQuery,
        IposCalendarQuery, IposDisclosureQuery, IposProspectusQuery, StockSplitsCalendarQuery,
        StockSplitsQuery,
    },
    types::{Date, Limit, Page, Ticker},
};

#[test]
fn ticker_queries_preserve_conversions_limits_and_report_time_states() {
    let symbol = Ticker::new("AAPL").unwrap();

    let dividends: DividendsQuery = (&symbol).into();
    assert_eq!(dividends.symbol(), &symbol);
    assert_eq!(dividends.limit(), None);
    assert_eq!(
        DividendsQuery::new(symbol.clone())
            .with_limit(Limit(0))
            .limit(),
        Some(Limit(0))
    );

    let earnings: EarningsQuery = symbol.clone().into();
    assert_eq!(earnings.symbol(), &symbol);
    assert_eq!(earnings.limit(), None);
    assert_eq!(earnings.include_report_times(), None);
    assert_eq!(
        earnings
            .with_limit(Limit(100))
            .with_include_report_times(false)
            .include_report_times(),
        Some(false)
    );

    let splits: StockSplitsQuery = (&symbol).into();
    assert_eq!(splits.symbol(), &symbol);
    assert_eq!(splits.limit(), None);
    assert_eq!(splits.with_limit(Limit(1_000)).limit(), Some(Limit(1_000)));
}

#[test]
fn every_calendar_date_query_preserves_independent_options_and_page_zero() {
    let from = Date::from_str("2026-03-06").unwrap();
    let to = Date::from_str("2026-06-06").unwrap();

    let dividends = DividendsCalendarQuery::new()
        .with_from(from)
        .with_page(Page(0));
    assert_eq!(dividends.from(), Some(from));
    assert_eq!(dividends.to(), None);
    assert_eq!(dividends.page(), Some(Page(0)));

    let earnings = EarningsCalendarQuery::new()
        .with_to(to)
        .with_page(Page(0))
        .with_include_report_times(true);
    assert_eq!(earnings.from(), None);
    assert_eq!(earnings.to(), Some(to));
    assert_eq!(earnings.page(), Some(Page(0)));
    assert_eq!(earnings.include_report_times(), Some(true));

    let ipo = IposCalendarQuery::new().with_from(from);
    assert_eq!(ipo.from(), Some(from));
    assert_eq!(ipo.to(), None);

    let disclosure = IposDisclosureQuery::new().with_to(to);
    assert_eq!(disclosure.from(), None);
    assert_eq!(disclosure.to(), Some(to));

    let prospectus = IposProspectusQuery::new().with_from(from).with_to(to);
    assert_eq!(prospectus.from(), Some(from));
    assert_eq!(prospectus.to(), Some(to));

    let splits = StockSplitsCalendarQuery::new()
        .with_to(to)
        .with_page(Page(0));
    assert_eq!(splits.from(), None);
    assert_eq!(splits.to(), Some(to));
    assert_eq!(splits.page(), Some(Page(0)));
}
