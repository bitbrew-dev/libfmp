use std::str::FromStr;

use libfmp::{
    query::Year,
    types::{
        ApiDateTime, BenchmarkYear, CalendarQuarter, CalendarYear, Cik, CongressionalMemberId,
        CountryCode, CurrencyCode, Cusip, Date, DateRange, ExchangeCode, FiniteDecimal, FormType,
        Industry, Isin, Lei, Limit, MarketHoursTimestamp, Page, SearchTerm, Sector,
        StatementAmount, StringValueError, Ticker, TickerList, TransactionTypeCode,
        UnixMilliseconds, UnixSeconds,
    },
};

#[test]
fn lei_preserves_its_exact_string_representation() {
    let lei = Lei::new("3003009W045RIKRBZI44").unwrap();
    assert_eq!(lei.as_str(), "3003009W045RIKRBZI44");
    assert_eq!(
        serde_json::to_string(&lei).unwrap(),
        r#""3003009W045RIKRBZI44""#
    );
    assert_eq!(
        serde_json::from_str::<Lei>(r#""3003009W045RIKRBZI44""#).unwrap(),
        lei
    );
    assert_eq!(Lei::new("").unwrap_err(), StringValueError::Empty);
    assert_eq!(
        Lei::new("3003009W045R\nKRBZI44").unwrap_err(),
        StringValueError::ControlCharacter
    );
}

#[test]
fn calendar_quarter_is_a_validated_numeric_response_fundamental() {
    for raw in 1..=4 {
        let quarter = CalendarQuarter::new(raw).unwrap();
        assert_eq!(quarter.get(), raw);
        assert_eq!(u8::from(quarter), raw);
        assert_eq!(quarter.to_string(), raw.to_string());
        assert_eq!(serde_json::to_string(&quarter).unwrap(), raw.to_string());
        assert_eq!(
            serde_json::from_str::<CalendarQuarter>(&raw.to_string()).unwrap(),
            quarter
        );
    }

    assert!(CalendarQuarter::new(0).is_err());
    assert!(CalendarQuarter::new(5).is_err());
    for invalid in ["0", "5", "2.0", r#""2""#] {
        assert!(serde_json::from_str::<CalendarQuarter>(invalid).is_err());
    }
}

#[test]
fn statement_amount_is_a_signed_f64() {
    let debit: StatementAmount = serde_json::from_str("-416161000000").unwrap();
    let fractional: StatementAmount = serde_json::from_str("1234.56").unwrap();

    assert_eq!(debit, -416_161_000_000.0);
    assert_eq!(fractional, 1_234.56);
}

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
fn sec_form_type_is_open_and_representation_preserving() {
    let form_type = FormType::new("10-K/A amendment").unwrap();
    assert_eq!(form_type.as_str(), "10-K/A amendment");
    assert_eq!(form_type.to_string(), "10-K/A amendment");
    assert_eq!(
        serde_json::from_str::<FormType>(r#""8-K""#)
            .unwrap()
            .as_str(),
        "8-K"
    );
}

#[test]
fn insider_transaction_type_code_is_open_and_representation_preserving() {
    let code = TransactionTypeCode::new("S-Sale / future").unwrap();
    assert_eq!(code.as_str(), "S-Sale / future");
    assert_eq!(code.to_string(), "S-Sale / future");
    assert_eq!(
        serde_json::from_str::<TransactionTypeCode>(r#""A-Award""#)
            .unwrap()
            .as_str(),
        "A-Award"
    );
    assert!(TransactionTypeCode::new("").is_err());
    assert!(TransactionTypeCode::new("bad\ncode").is_err());
}

#[test]
fn congressional_member_id_is_chamber_neutral_and_representation_preserving() {
    for raw in ["M001242", "C001120", "  P000197  "] {
        let member_id = CongressionalMemberId::new(raw).unwrap();
        assert_eq!(member_id.as_str(), raw);
        assert_eq!(member_id.to_string(), raw);
        assert_eq!(
            serde_json::from_str::<CongressionalMemberId>(&serde_json::to_string(raw).unwrap())
                .unwrap(),
            member_id
        );
    }
    assert!(CongressionalMemberId::new("").is_err());
    assert!(CongressionalMemberId::new("M001\n242").is_err());
}

#[test]
fn market_hours_timestamp_is_opaque_and_representation_preserving() {
    let timestamp = MarketHoursTimestamp::new("001769527402").unwrap();
    assert_eq!(timestamp.as_str(), "001769527402");
    assert_eq!(timestamp.to_string(), "001769527402");
    assert_eq!(
        serde_json::from_str::<MarketHoursTimestamp>(r#""001769527402""#).unwrap(),
        timestamp
    );
    assert!(MarketHoursTimestamp::new("").is_err());
    assert!(MarketHoursTimestamp::new("1769\n527402").is_err());
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
    assert_eq!(SearchTerm::new(" ").unwrap_err(), StringValueError::Empty);
    assert_eq!(
        SearchTerm::new("Apple\nInc.").unwrap_err(),
        StringValueError::ControlCharacter
    );
}

#[test]
fn search_terms_preserve_representation_without_identifier_normalization() {
    let term = SearchTerm::new("  Apple, Inc. / Class A  ").unwrap();

    assert_eq!(term.as_str(), "  Apple, Inc. / Class A  ");
    assert_eq!(term.to_string(), "  Apple, Inc. / Class A  ");
    assert_eq!(
        serde_json::from_str::<SearchTerm>(r#""  Apple, Inc. / Class A  ""#).unwrap(),
        term
    );
}

#[test]
fn benchmark_years_preserve_open_non_numeric_representations_without_query_year_collision() {
    let year = BenchmarkYear::new("FY 2024/25").unwrap();

    assert_eq!(year.as_str(), "FY 2024/25");
    assert_eq!(year.to_string(), "FY 2024/25");
    assert_eq!(
        serde_json::from_str::<BenchmarkYear>(r#""FY 2024/25""#).unwrap(),
        year
    );
    assert_eq!(BenchmarkYear::new(" 2024 ").unwrap().as_str(), " 2024 ");
    assert_eq!(
        BenchmarkYear::new(" ").unwrap_err(),
        StringValueError::Empty
    );
    assert_eq!(
        BenchmarkYear::new("2024\n").unwrap_err(),
        StringValueError::ControlCharacter
    );
    assert_eq!(Year(2024).to_string(), "2024");
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
    assert_eq!(
        Sector::new("Future Sector").unwrap().as_str(),
        "Future Sector"
    );
    assert_eq!(
        Industry::new("Future Industry").unwrap().as_str(),
        "Future Industry"
    );
    assert!(Sector::new("").is_err());
    assert!(Industry::new("bad\nindustry").is_err());
}

#[test]
fn finite_decimal_rejects_values_that_cannot_safely_enter_a_url() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(FiniteDecimal::new(value).is_err());
    }

    for value in [-1.25, 0.0, 10.5, f64::MAX] {
        let finite = FiniteDecimal::new(value).unwrap();
        assert_eq!(finite.get(), value);
        assert_eq!(f64::from(finite), value);
        let json = serde_json::to_string(&finite).unwrap();
        assert_eq!(
            serde_json::from_str::<FiniteDecimal>(&json).unwrap(),
            finite
        );
    }
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
fn calendar_year_preserves_the_numeric_response_contract() {
    let year = CalendarYear::from(2026);

    assert_eq!(year.get(), 2026);
    assert_eq!(u32::from(year), 2026);
    assert_eq!(year.to_string(), "2026");
    assert_eq!(serde_json::to_string(&year).unwrap(), "2026");
    assert_eq!(serde_json::from_str::<CalendarYear>("2026").unwrap(), year);
    assert!(serde_json::from_str::<CalendarYear>(r#""2026""#).is_err());
    assert!(serde_json::from_str::<CalendarYear>("-1").is_err());
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
