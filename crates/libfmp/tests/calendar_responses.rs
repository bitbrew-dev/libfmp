use std::str::FromStr;

use libfmp::{
    codecs::IsoTimestamp,
    responses::calendar::{
        DividendEvent, EarningsEvent, IpoCalendarEvent, IpoDisclosure, IpoProspectus,
        StockSplitEvent,
    },
    types::{Cik, Date, ExchangeCode, Ticker},
};
use serde::de::DeserializeOwned;

const DIVIDENDS: &[u8] = include_bytes!("fixtures/dividends.json");
const DIVIDENDS_CALENDAR: &[u8] = include_bytes!("fixtures/dividends_calendar.json");
const EARNINGS: &[u8] = include_bytes!("fixtures/earnings.json");
const EARNINGS_CALENDAR: &[u8] = include_bytes!("fixtures/earnings_calendar.json");
const IPOS_CALENDAR: &[u8] = include_bytes!("fixtures/ipos_calendar.json");
const IPOS_DISCLOSURE: &[u8] = include_bytes!("fixtures/ipos_disclosure.json");
const IPOS_PROSPECTUS: &[u8] = include_bytes!("fixtures/ipos_prospectus.json");
const STOCK_SPLITS: &[u8] = include_bytes!("fixtures/stock_splits.json");
const STOCK_SPLITS_CALENDAR: &[u8] = include_bytes!("fixtures/stock_splits_calendar.json");

#[test]
fn both_exact_dividend_fixtures_share_the_nine_field_row() {
    assert_field_count(DIVIDENDS, 9);
    assert_field_count(DIVIDENDS_CALENDAR, 9);

    let company: Vec<DividendEvent> = serde_json::from_slice(DIVIDENDS).unwrap();
    assert_eq!(
        company,
        [DividendEvent {
            symbol: Ticker::new("AAPL").unwrap(),
            date: Date::from_str("2026-05-11").unwrap(),
            record_date: Date::from_str("2026-05-11").unwrap(),
            payment_date: Date::from_str("2026-05-14").unwrap(),
            declaration_date: Some(Date::from_str("2026-04-30").unwrap()),
            adj_dividend: 0.27,
            dividend: 0.27,
            r#yield: 0.3587535875358754,
            frequency: "Quarterly".to_owned(),
        }]
    );

    let calendar: Vec<DividendEvent> = serde_json::from_slice(DIVIDENDS_CALENDAR).unwrap();
    assert_eq!(calendar[0].symbol.as_str(), "5871.TW");
    assert_eq!(calendar[0].declaration_date, None);
    assert_eq!(calendar[0].frequency, "Annual");

    let mut null: serde_json::Value = serde_json::from_slice(DIVIDENDS).unwrap();
    null[0]["declarationDate"] = serde_json::Value::Null;
    assert_eq!(
        serde_json::from_value::<Vec<DividendEvent>>(null).unwrap()[0].declaration_date,
        None
    );
}

#[test]
fn both_exact_earnings_fixtures_preserve_only_nullable_actual_values() {
    assert_field_count(EARNINGS, 7);
    assert_field_count(EARNINGS_CALENDAR, 7);

    let company: Vec<EarningsEvent> = serde_json::from_slice(EARNINGS).unwrap();
    assert_eq!(
        company,
        [EarningsEvent {
            symbol: Ticker::new("AAPL").unwrap(),
            date: Date::from_str("2026-07-30").unwrap(),
            eps_actual: None,
            eps_estimated: 1.88,
            revenue_actual: None,
            revenue_estimated: 109_038_900_000,
            last_updated: Date::from_str("2026-07-30").unwrap(),
        }]
    );

    let calendar: Vec<EarningsEvent> = serde_json::from_slice(EARNINGS_CALENDAR).unwrap();
    assert_eq!(calendar[0].symbol.as_str(), "GRG.L");
    assert_eq!(calendar[0].eps_actual, Some(0.549));
    assert_eq!(calendar[0].eps_estimated, 0.501);
    assert_eq!(calendar[0].revenue_actual, Some(1_101_500_000));
    assert_eq!(calendar[0].revenue_estimated, 1_086_300_000);
}

#[test]
fn exact_ipo_calendar_fixture_preserves_literal_daa_and_source_only_nulls() {
    assert_field_count(IPOS_CALENDAR, 9);
    let rows: Vec<IpoCalendarEvent> = serde_json::from_slice(IPOS_CALENDAR).unwrap();
    assert_eq!(
        rows,
        [IpoCalendarEvent {
            symbol: Ticker::new("IMC").unwrap(),
            date: Date::from_str("2026-07-29").unwrap(),
            daa: IsoTimestamp::from_str("2026-07-29T04:00:00.000Z").unwrap(),
            company: "IMC Rare Earths Ltd".to_owned(),
            exchange: ExchangeCode::new("NYSE").unwrap(),
            actions: "Priced".to_owned(),
            shares: None,
            price_range: None,
            market_cap: None,
        }]
    );

    let encoded = serde_json::to_value(&rows[0]).unwrap();
    assert_eq!(encoded["daa"], "2026-07-29T04:00:00.000Z");
    assert!(encoded.get("dateAdded").is_none());

    let mut dynamic: serde_json::Value = serde_json::from_slice(IPOS_CALENDAR).unwrap();
    dynamic[0]["shares"] = serde_json::json!(18_446_744_073_709_551_615_u64);
    dynamic[0]["priceRange"] = serde_json::json!({ "low": 12, "high": "open" });
    dynamic[0]["marketCap"] = serde_json::json!([1, true, null]);
    let rows: Vec<IpoCalendarEvent> = serde_json::from_value(dynamic).unwrap();
    assert_eq!(
        rows[0].shares.as_ref().unwrap(),
        &serde_json::json!(18_446_744_073_709_551_615_u64)
    );
    assert!(rows[0].price_range.as_ref().unwrap().is_object());
    assert!(rows[0].market_cap.as_ref().unwrap().is_array());
}

#[test]
fn exact_ipo_disclosure_fixture_preserves_dates_leading_zero_cik_and_url() {
    assert_field_count(IPOS_DISCLOSURE, 7);
    let rows: Vec<IpoDisclosure> = serde_json::from_slice(IPOS_DISCLOSURE).unwrap();
    assert_eq!(
        rows,
        [IpoDisclosure {
            symbol: Ticker::new("QTJA").unwrap(),
            filing_date: Date::from_str("2026-07-30").unwrap(),
            accepted_date: Date::from_str("2026-07-30").unwrap(),
            effectiveness_date: Date::from_str("2026-07-30").unwrap(),
            cik: Cik::new("0001415726").unwrap(),
            form: "CERT".to_owned(),
            url: "https://www.sec.gov/Archives/edgar/data/1415726/000141783526000235/8A_Cert_DDTG_DDFG.pdf".to_owned(),
        }]
    );
    assert_eq!(rows[0].cik.as_str(), "0001415726");
}

#[test]
fn exact_ipo_prospectus_fixture_decodes_all_thirteen_fields() {
    assert_field_count(IPOS_PROSPECTUS, 13);
    let rows: Vec<IpoProspectus> = serde_json::from_slice(IPOS_PROSPECTUS).unwrap();
    assert_eq!(
        rows,
        [IpoProspectus {
            symbol: Ticker::new("FTW-WT").unwrap(),
            accepted_date: Date::from_str("2026-07-29").unwrap(),
            filing_date: Date::from_str("2026-07-30").unwrap(),
            ipo_date: Date::from_str("2026-07-28").unwrap(),
            cik: Cik::new("0002083125").unwrap(),
            price_public_per_share: 1.0,
            price_public_total: 434.0,
            discounts_and_commissions_per_share: 0.0,
            discounts_and_commissions_total: 82_251.0,
            proceeds_before_expenses_per_share: 1.0,
            proceeds_before_expenses_total: 82_251.0,
            form: "S-1".to_owned(),
            url: "https://www.sec.gov/Archives/edgar/data/2083125/000121390026082963/ea0298363-s1_presidio.htm".to_owned(),
        }]
    );
}

#[test]
fn both_exact_stock_split_fixtures_share_the_five_field_integer_row() {
    assert_field_count(STOCK_SPLITS, 5);
    assert_field_count(STOCK_SPLITS_CALENDAR, 5);

    let company: Vec<StockSplitEvent> = serde_json::from_slice(STOCK_SPLITS).unwrap();
    assert_eq!(
        company,
        [StockSplitEvent {
            symbol: Ticker::new("AAPL").unwrap(),
            date: Date::from_str("2020-08-31").unwrap(),
            numerator: 4.0,
            denominator: 1.0,
            split_type: "stock-split".to_owned(),
        }]
    );

    let calendar: Vec<StockSplitEvent> = serde_json::from_slice(STOCK_SPLITS_CALENDAR).unwrap();
    assert_eq!(calendar[0].symbol.as_str(), "WHLR");
    assert_eq!(calendar[0].numerator, 1.0);
    assert_eq!(calendar[0].denominator, 5.0);
}

#[test]
fn documented_fields_are_required_nullable_fields_stay_nullable_and_unknowns_are_accepted() {
    assert_contract::<DividendEvent>(DIVIDENDS, &["declarationDate"]);
    assert_contract::<DividendEvent>(DIVIDENDS_CALENDAR, &["declarationDate"]);
    assert_contract::<EarningsEvent>(EARNINGS, &["epsActual", "revenueActual"]);
    assert_contract::<EarningsEvent>(EARNINGS_CALENDAR, &["epsActual", "revenueActual"]);
    assert_contract::<IpoCalendarEvent>(IPOS_CALENDAR, &["shares", "priceRange", "marketCap"]);
    assert_contract::<IpoDisclosure>(IPOS_DISCLOSURE, &[]);
    assert_contract::<IpoProspectus>(IPOS_PROSPECTUS, &[]);
    assert_contract::<StockSplitEvent>(STOCK_SPLITS, &[]);
    assert_contract::<StockSplitEvent>(STOCK_SPLITS_CALENDAR, &[]);
}

fn assert_contract<T: DeserializeOwned>(fixture: &[u8], nullable: &[&str]) {
    let source: serde_json::Value = serde_json::from_slice(fixture).unwrap();
    let keys = source[0]
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect::<Vec<_>>();

    for key in keys {
        let mut missing = source.clone();
        missing[0].as_object_mut().unwrap().remove(&key);
        assert!(
            serde_json::from_value::<Vec<T>>(missing).is_err(),
            "accepted missing {key}"
        );

        let mut null = source.clone();
        null[0][&key] = serde_json::Value::Null;
        let decoded = serde_json::from_value::<Vec<T>>(null);
        assert_eq!(
            decoded.is_ok(),
            nullable.contains(&key.as_str()),
            "unexpected null behavior for {key}"
        );
    }

    let mut forward = source;
    forward[0]["futureProviderField"] = serde_json::json!({ "nested": [1, true, null] });
    let duplicate = forward[0].clone();
    forward.as_array_mut().unwrap().push(duplicate);
    assert_eq!(serde_json::from_value::<Vec<T>>(forward).unwrap().len(), 2);
}

fn assert_field_count(fixture: &[u8], expected: usize) {
    let value: serde_json::Value = serde_json::from_slice(fixture).unwrap();
    assert_eq!(value[0].as_object().unwrap().len(), expected);
}

#[test]
fn all_six_calendar_rows_are_bare_arrays_preserving_empty_shapes() {
    assert!(
        serde_json::from_slice::<Vec<DividendEvent>>(b"[]")
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_slice::<Vec<EarningsEvent>>(b"[]")
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_slice::<Vec<IpoCalendarEvent>>(b"[]")
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_slice::<Vec<IpoDisclosure>>(b"[]")
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_slice::<Vec<IpoProspectus>>(b"[]")
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_slice::<Vec<StockSplitEvent>>(b"[]")
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_value::<Vec<DividendEvent>>(serde_json::json!({ "events": [] })).is_err()
    );
}
