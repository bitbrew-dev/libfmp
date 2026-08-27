use std::str::FromStr;

use libfmp::types::{
    ApiDateTime, Cik, CountryCode, CurrencyCode, Cusip, Date, DateRange, ExchangeCode, Isin, Limit,
    Page, StringValueError, Ticker, TickerList, UnixMilliseconds, UnixSeconds,
};

#[test]
fn string_values_preserve_representation_and_leading_zeroes() {
    let ticker = Ticker::new("000001.SZ").unwrap();
    let cik = Cik::new("0000320193").unwrap();
    let cusip = Cusip::new(" 037833100 ").unwrap();
    let isin = Isin::new("US0378331005").unwrap();

    assert_eq!(ticker.as_str(), "000001.SZ");
    assert_eq!(cik.as_str(), "0000320193");
    assert_eq!(cusip.as_str(), " 037833100 ");
    assert_eq!(isin.as_str(), "US0378331005");
    assert_eq!(serde_json::from_str::<Cik>(r#""0000320193""#).unwrap(), cik);
}

#[test]
fn documented_index_and_international_tickers_are_accepted() {
    assert_eq!(Ticker::new("^VIX").unwrap().as_str(), "^VIX");
    assert_eq!(Ticker::new("000001.SZ").unwrap().as_str(), "000001.SZ");
}

#[test]
fn string_values_reject_empty_whitespace_and_controls() {
    assert_eq!(Ticker::new("").unwrap_err(), StringValueError::Empty);
    assert_eq!(
        Cik::new("  \u{2003} ").unwrap_err(),
        StringValueError::Empty
    );
    assert_eq!(
        Cusip::new("0378\n33100").unwrap_err(),
        StringValueError::ControlCharacter
    );
    assert_eq!(
        Isin::new("US037833\t1005").unwrap_err(),
        StringValueError::ControlCharacter
    );

    assert!(ExchangeCode::new("").is_err());
    assert!(CurrencyCode::new("\r").is_err());
    assert!(CountryCode::new("   ").is_err());
    assert!(serde_json::from_str::<Ticker>(r#""line\nbreak""#).is_err());
}

#[test]
fn ticker_rejects_comma_but_other_string_values_remain_open() {
    assert_eq!(
        Ticker::new("AAPL,MSFT").unwrap_err(),
        StringValueError::Comma
    );
    assert!(serde_json::from_str::<Ticker>(r#""AAPL,MSFT""#).is_err());

    assert_eq!(
        ExchangeCode::new("NEW-EXCHANGE").unwrap().as_str(),
        "NEW-EXCHANGE"
    );
    assert_eq!(CurrencyCode::new("XTS").unwrap().as_str(), "XTS");
    assert_eq!(CountryCode::new("N/A").unwrap().as_str(), "N/A");
}

#[test]
fn ticker_list_is_non_empty_and_has_deterministic_query_encoding() {
    assert!(TickerList::new(Vec::new()).is_err());

    let tickers = TickerList::new(vec![
        Ticker::new("AAPL").unwrap(),
        Ticker::new("^VIX").unwrap(),
        Ticker::new("000001.SZ").unwrap(),
    ])
    .unwrap();

    assert_eq!(tickers.len(), 3);
    assert!(!tickers.is_empty());
    assert_eq!(tickers.to_string(), "AAPL,^VIX,000001.SZ");
    assert_eq!(
        tickers.iter().map(Ticker::as_str).collect::<Vec<_>>(),
        ["AAPL", "^VIX", "000001.SZ"]
    );
}

#[test]
fn date_requires_the_exact_wire_format_and_valid_calendar_values() {
    let date = Date::from_str("2024-02-29").unwrap();
    assert_eq!(date.to_string(), "2024-02-29");
    assert_eq!(serde_json::to_string(&date).unwrap(), r#""2024-02-29""#);
    assert_eq!(
        serde_json::from_str::<Date>(r#""2024-02-29""#).unwrap(),
        date
    );

    for invalid in [
        "2024-2-29",
        "24-02-29",
        "2024/02/29",
        "2023-02-29",
        "2024-02-29 ",
    ] {
        assert!(Date::from_str(invalid).is_err(), "accepted {invalid:?}");
    }
}

#[test]
fn api_datetime_requires_the_exact_timezone_less_wire_format() {
    let datetime = ApiDateTime::from_str("2024-02-29 23:59:58").unwrap();
    assert_eq!(datetime.to_string(), "2024-02-29 23:59:58");
    assert_eq!(
        serde_json::to_string(&datetime).unwrap(),
        r#""2024-02-29 23:59:58""#
    );
    assert_eq!(
        serde_json::from_str::<ApiDateTime>(r#""2024-02-29 23:59:58""#).unwrap(),
        datetime
    );

    for invalid in [
        "2024-02-29T23:59:58",
        "2024-02-29 23:59",
        "2024-02-29 23:59:58Z",
        "2024-02-29 24:00:00",
        "2023-02-29 12:00:00",
    ] {
        assert!(
            ApiDateTime::from_str(invalid).is_err(),
            "accepted {invalid:?}"
        );
    }
}

#[test]
fn inclusive_date_range_validates_ordering() {
    let start = Date::from_str("2024-01-01").unwrap();
    let end = Date::from_str("2024-12-31").unwrap();

    let range = DateRange::new(start, end).unwrap();
    assert_eq!(range.from(), start);
    assert_eq!(range.to(), end);
    assert!(DateRange::new(start, start).is_ok());
    assert!(DateRange::new(end, start).is_err());
}

#[test]
fn timestamps_keep_seconds_and_milliseconds_as_distinct_numeric_types() {
    let seconds: UnixSeconds = serde_json::from_str("1700000000").unwrap();
    let milliseconds: UnixMilliseconds = serde_json::from_str("1700000000000").unwrap();

    assert_eq!(seconds, UnixSeconds(1_700_000_000));
    assert_eq!(milliseconds, UnixMilliseconds(1_700_000_000_000));
    assert_eq!(seconds.to_string(), "1700000000");
    assert_eq!(milliseconds.to_string(), "1700000000000");
    assert_eq!(serde_json::to_string(&seconds).unwrap(), "1700000000");
    assert_eq!(
        serde_json::to_string(&milliseconds).unwrap(),
        "1700000000000"
    );
}

#[test]
fn pagination_units_do_not_invent_global_bounds() {
    assert_eq!(Page(0).0, 0);
    assert_eq!(Page(u32::MAX).0, u32::MAX);
    assert_eq!(Limit(0).0, 0);
    assert_eq!(Limit(u32::MAX).0, u32::MAX);
    assert_eq!(Page(0).to_string(), "0");
    assert_eq!(Limit(1000).to_string(), "1000");
}
