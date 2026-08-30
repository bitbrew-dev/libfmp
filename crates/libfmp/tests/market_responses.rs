use libfmp::{
    responses::market::{
        IndustryPe, IndustryPerformance, MarketMover, SectorPe, SectorPerformance,
    },
    types::{Date, ExchangeCode, Industry, Sector, Ticker},
};
use serde::{Serialize, de::DeserializeOwned};

const SECTOR_SNAPSHOT: &[u8] = include_bytes!("fixtures/sector_performance_snapshot.json");
const INDUSTRY_SNAPSHOT: &[u8] = include_bytes!("fixtures/industry_performance_snapshot.json");
const HISTORICAL_SECTOR: &[u8] = include_bytes!("fixtures/historical_sector_performance.json");
const HISTORICAL_INDUSTRY: &[u8] = include_bytes!("fixtures/historical_industry_performance.json");
const SECTOR_PE_SNAPSHOT: &[u8] = include_bytes!("fixtures/sector_pe_snapshot.json");
const INDUSTRY_PE_SNAPSHOT: &[u8] = include_bytes!("fixtures/industry_pe_snapshot.json");
const HISTORICAL_SECTOR_PE: &[u8] = include_bytes!("fixtures/historical_sector_pe.json");
const HISTORICAL_INDUSTRY_PE: &[u8] = include_bytes!("fixtures/historical_industry_pe.json");
const GAINERS: &[u8] = include_bytes!("fixtures/biggest_gainers.json");
const LOSERS: &[u8] = include_bytes!("fixtures/biggest_losers.json");
const ACTIVES: &[u8] = include_bytes!("fixtures/most_actives.json");

#[test]
fn all_eleven_outer_fixtures_decode_and_preserve_exact_documented_values() {
    assert_exact::<SectorPerformance>(SECTOR_SNAPSHOT, 4);
    assert_exact::<IndustryPerformance>(INDUSTRY_SNAPSHOT, 4);
    assert_exact::<SectorPerformance>(HISTORICAL_SECTOR, 4);
    assert_exact::<IndustryPerformance>(HISTORICAL_INDUSTRY, 4);
    assert_exact::<SectorPe>(SECTOR_PE_SNAPSHOT, 4);
    assert_exact::<IndustryPe>(INDUSTRY_PE_SNAPSHOT, 4);
    assert_exact::<SectorPe>(HISTORICAL_SECTOR_PE, 4);
    assert_exact::<IndustryPe>(HISTORICAL_INDUSTRY_PE, 4);
    assert_exact::<MarketMover>(GAINERS, 6);
    assert_exact::<MarketMover>(LOSERS, 6);
    assert_exact::<MarketMover>(ACTIVES, 6);
}

#[test]
fn performance_and_pe_rows_are_shared_by_snapshot_and_history_endpoints() {
    assert_shared::<SectorPerformance>(SECTOR_SNAPSHOT, HISTORICAL_SECTOR);
    assert_shared::<IndustryPerformance>(INDUSTRY_SNAPSHOT, HISTORICAL_INDUSTRY);
    assert_shared::<SectorPe>(SECTOR_PE_SNAPSHOT, HISTORICAL_SECTOR_PE);
    assert_shared::<IndustryPe>(INDUSTRY_PE_SNAPSHOT, HISTORICAL_INDUSTRY_PE);
    assert_shared::<MarketMover>(GAINERS, LOSERS);
    assert_shared::<MarketMover>(LOSERS, ACTIVES);
}

#[test]
fn exact_fields_preserve_signed_fractional_zero_and_integer_json_numerics() {
    for (fixture, expected) in [
        (SECTOR_SNAPSHOT, -0.31481377464310634),
        (HISTORICAL_SECTOR, 1.3989969286740689),
    ] {
        let rows: Vec<SectorPerformance> = serde_json::from_slice(fixture).unwrap();
        assert_eq!(rows[0].average_change, expected);
    }
    let gainers: Vec<MarketMover> = serde_json::from_slice(GAINERS).unwrap();
    assert_eq!(gainers[0].changes_percentage, 100.0);
    assert_eq!(gainers[0].change, 0.0001);
    let losers: Vec<MarketMover> = serde_json::from_slice(LOSERS).unwrap();
    assert_eq!(losers[0].change, -0.002);
    assert_eq!(losers[0].changes_percentage, -90.90909);

    let numeric_forms = serde_json::json!([
        {
            "symbol": "ZERO",
            "price": 0,
            "name": "Integer price and zero changes",
            "change": 0,
            "changesPercentage": 0.0,
            "exchange": "FUTURE / EXCHANGE"
        }
    ]);
    let rows: Vec<MarketMover> = serde_json::from_value(numeric_forms).unwrap();
    assert_eq!(rows[0].price, 0.0);
    assert_eq!(rows[0].change, 0.0);
    assert_eq!(rows[0].changes_percentage, 0.0);

    let integer_pe = serde_json::json!([
        {
            "date": "2024-02-29",
            "sector": "Future / Sector",
            "exchange": "NEW-EXCHANGE",
            "pe": -3
        }
    ]);
    let rows: Vec<SectorPe> = serde_json::from_value(integer_pe).unwrap();
    assert_eq!(rows[0].pe, -3.0);
}

#[test]
fn open_fundamentals_preserve_provider_values_without_closed_vocabularies() {
    let value = serde_json::json!([
        {
            "date": "2024-02-29",
            "industry": "Future & Quantum / Services",
            "exchange": "NEW-EXCHANGE / DARK",
            "averageChange": 0
        }
    ]);
    let rows: Vec<IndustryPerformance> = serde_json::from_value(value).unwrap();
    assert_eq!(rows[0].industry.as_str(), "Future & Quantum / Services");
    assert_eq!(rows[0].exchange.as_str(), "NEW-EXCHANGE / DARK");

    let mover = MarketMover {
        symbol: Ticker::new("BRK.B / Class A").unwrap(),
        price: 1.0,
        name: "Provider Name".to_owned(),
        change: 0.0,
        changes_percentage: 0.0,
        exchange: ExchangeCode::new("FUTURE").unwrap(),
    };
    assert_eq!(mover.symbol.as_str(), "BRK.B / Class A");
}

#[test]
fn dates_are_strict_yyyy_mm_dd_calendar_dates() {
    for invalid in [
        "2024-2-01",
        "2024-02-1",
        "2024/02/01",
        "2024-02-30",
        "2024-02-01T00:00:00",
    ] {
        let mut value: serde_json::Value = serde_json::from_slice(SECTOR_SNAPSHOT).unwrap();
        value[0]["date"] = serde_json::json!(invalid);
        assert!(serde_json::from_value::<Vec<SectorPerformance>>(value).is_err());
    }
    assert!(serde_json::from_slice::<Vec<SectorPerformance>>(SECTOR_SNAPSHOT).is_ok());
}

#[test]
fn every_documented_field_is_required_non_null_and_unknown_fields_are_accepted() {
    assert_contract::<SectorPerformance>(SECTOR_SNAPSHOT);
    assert_contract::<IndustryPerformance>(INDUSTRY_SNAPSHOT);
    assert_contract::<SectorPerformance>(HISTORICAL_SECTOR);
    assert_contract::<IndustryPerformance>(HISTORICAL_INDUSTRY);
    assert_contract::<SectorPe>(SECTOR_PE_SNAPSHOT);
    assert_contract::<IndustryPe>(INDUSTRY_PE_SNAPSHOT);
    assert_contract::<SectorPe>(HISTORICAL_SECTOR_PE);
    assert_contract::<IndustryPe>(HISTORICAL_INDUSTRY_PE);
    assert_contract::<MarketMover>(GAINERS);
    assert_contract::<MarketMover>(LOSERS);
    assert_contract::<MarketMover>(ACTIVES);
}

#[test]
fn all_five_contracts_are_bare_arrays_preserving_empty_and_multiple_rows() {
    assert_bare_array::<SectorPerformance>(SECTOR_SNAPSHOT);
    assert_bare_array::<IndustryPerformance>(INDUSTRY_SNAPSHOT);
    assert_bare_array::<SectorPe>(SECTOR_PE_SNAPSHOT);
    assert_bare_array::<IndustryPe>(INDUSTRY_PE_SNAPSHOT);
    assert_bare_array::<MarketMover>(GAINERS);
    assert!(
        serde_json::from_value::<Vec<MarketMover>>(serde_json::json!({ "movers": [] })).is_err()
    );
}

fn assert_exact<T: DeserializeOwned + Serialize>(fixture: &[u8], fields: usize) {
    let source: serde_json::Value = serde_json::from_slice(fixture).unwrap();
    assert_eq!(source[0].as_object().unwrap().len(), fields);
    let rows: Vec<T> = serde_json::from_value(source.clone()).unwrap();
    assert_json_values(&serde_json::to_value(rows).unwrap(), &source);
}

fn assert_json_values(actual: &serde_json::Value, expected: &serde_json::Value) {
    match (actual, expected) {
        (serde_json::Value::Array(actual), serde_json::Value::Array(expected)) => {
            assert_eq!(actual.len(), expected.len());
            for (actual, expected) in actual.iter().zip(expected) {
                assert_json_values(actual, expected);
            }
        }
        (serde_json::Value::Object(actual), serde_json::Value::Object(expected)) => {
            assert_eq!(actual.len(), expected.len());
            for (key, expected) in expected {
                assert_json_values(&actual[key], expected);
            }
        }
        (serde_json::Value::Number(actual), serde_json::Value::Number(expected)) => {
            assert_eq!(actual.as_f64(), expected.as_f64());
        }
        _ => assert_eq!(actual, expected),
    }
}

fn assert_shared<T: DeserializeOwned>(first: &[u8], second: &[u8]) {
    assert_eq!(serde_json::from_slice::<Vec<T>>(first).unwrap().len(), 1);
    assert_eq!(serde_json::from_slice::<Vec<T>>(second).unwrap().len(), 1);
}

fn assert_contract<T: DeserializeOwned>(fixture: &[u8]) {
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
        assert!(
            serde_json::from_value::<Vec<T>>(null).is_err(),
            "accepted null {key}"
        );
    }
    let mut forward = source;
    forward[0]["futureProviderField"] = serde_json::json!({ "nested": [1, true, null] });
    assert_eq!(serde_json::from_value::<Vec<T>>(forward).unwrap().len(), 1);
}

fn assert_bare_array<T: DeserializeOwned>(fixture: &[u8]) {
    assert!(serde_json::from_slice::<Vec<T>>(b"[]").unwrap().is_empty());
    let mut source: serde_json::Value = serde_json::from_slice(fixture).unwrap();
    let duplicate = source[0].clone();
    source.as_array_mut().unwrap().push(duplicate);
    assert_eq!(serde_json::from_value::<Vec<T>>(source).unwrap().len(), 2);
}

#[test]
fn response_fundamental_types_are_the_reserved_shared_types() {
    let sector = Sector::new("Energy").unwrap();
    let industry = Industry::new("Biotechnology").unwrap();
    let exchange = ExchangeCode::new("NASDAQ").unwrap();
    let date: Date = "2024-03-01".parse().unwrap();
    let _: Sector = sector;
    let _: Industry = industry;
    let _: ExchangeCode = exchange;
    let _: Date = date;
}
