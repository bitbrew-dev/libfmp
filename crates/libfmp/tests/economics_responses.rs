#[macro_use]
#[path = "support/assert_row.rs"]
mod assert_row;

use std::str::FromStr;

use libfmp::{
    query::EconomicIndicator,
    responses::economics::{
        EconomicCalendarEvent, EconomicIndicatorObservation, MarketRiskPremium, TreasuryRate,
    },
    types::{ApiDateTime, CountryCode, CurrencyCode, Date},
};
use serde::de::DeserializeOwned;

const TREASURY: &[u8] = include_bytes!("fixtures/treasury_rates.json");
const INDICATORS: &[u8] = include_bytes!("fixtures/economic_indicators.json");
const CALENDAR: &[u8] = include_bytes!("fixtures/economic_calendar.json");
const RISK_PREMIUM: &[u8] = include_bytes!("fixtures/market_risk_premium.json");

#[test]
fn treasury_fixture_decodes_all_13_exact_fields_without_scaling() {
    let value: serde_json::Value = serde_json::from_slice(TREASURY).unwrap();
    assert_eq!(value[0].as_object().unwrap().len(), 13);

    let rows: Vec<TreasuryRate> = serde_json::from_value(value).unwrap();
    assert_rows!(
        rows,
        [TreasuryRate {
            date: Date::from_str("2026-07-29").unwrap(),
            month_1: 3.73,
            month_2: 3.83,
            month_3: 3.83,
            month_6: 3.97,
            year_1: 4.04,
            year_2: 4.22,
            year_3: 4.29,
            year_5: 4.37,
            year_7: 4.51,
            year_10: 4.67,
            year_20: 5.21,
            year_30: 5.2,
        }]
    );
}

#[test]
fn indicator_fixture_decodes_exact_three_field_contract() {
    let value: serde_json::Value = serde_json::from_slice(INDICATORS).unwrap();
    assert_eq!(value[0].as_object().unwrap().len(), 3);

    let rows: Vec<EconomicIndicatorObservation> = serde_json::from_value(value).unwrap();
    assert_rows!(
        rows,
        [EconomicIndicatorObservation {
            name: EconomicIndicator::Gdp,
            date: Date::from_str("2025-10-01").unwrap(),
            value: 31_422.526,
        }]
    );

    let mut open: serde_json::Value = serde_json::from_slice(INDICATORS).unwrap();
    open[0]["name"] = serde_json::json!("futureProviderIndicator");
    let rows: Vec<EconomicIndicatorObservation> = serde_json::from_value(open).unwrap();
    assert_eq!(rows[0].name.as_str(), "futureProviderIndicator");
}

#[test]
fn calendar_fixture_decodes_all_11_fields_with_required_numeric_values() {
    let value: serde_json::Value = serde_json::from_slice(CALENDAR).unwrap();
    assert_eq!(value[0].as_object().unwrap().len(), 11);

    let rows: Vec<EconomicCalendarEvent> = serde_json::from_value(value).unwrap();
    assert_rows!(
        rows,
        [EconomicCalendarEvent {
            date: ApiDateTime::from_str("2026-07-29 03:30:00").unwrap(),
            country: CountryCode::new("SG").unwrap(),
            event: "Import Prices YoY (Jun)".to_owned(),
            currency: CurrencyCode::new("SGD").unwrap(),
            previous: 18.5,
            estimate: 21.0,
            actual: 13.6,
            change: -4.9,
            impact: "Low".to_owned(),
            change_percentage: -26.486,
            unit: "%".to_owned(),
        }]
    );
}

#[test]
fn market_risk_fixture_uses_full_country_name_and_raw_percentage_values() {
    let value: serde_json::Value = serde_json::from_slice(RISK_PREMIUM).unwrap();
    assert_eq!(value[0].as_object().unwrap().len(), 4);

    let rows: Vec<MarketRiskPremium> = serde_json::from_value(value).unwrap();
    assert_rows!(
        rows,
        [MarketRiskPremium {
            country: "Zimbabwe".to_owned(),
            continent: "Africa".to_owned(),
            country_risk_premium: 11.66,
            total_equity_risk_premium: 15.89,
        }]
    );
    assert_eq!(rows[0].country, "Zimbabwe");
    assert_eq!(rows[0].country_risk_premium, 11.66);
    assert_eq!(rows[0].total_equity_risk_premium, 15.89);
    assert!(
        serde_json::to_value(&rows[0])
            .unwrap()
            .get("date")
            .is_none()
    );
}

#[test]
fn every_economics_row_requires_documented_fields_and_accepts_unknown_fields() {
    assert_contract::<TreasuryRate>(TREASURY);
    assert_contract::<EconomicIndicatorObservation>(INDICATORS);
    assert_contract::<EconomicCalendarEvent>(CALENDAR);
    assert_contract::<MarketRiskPremium>(RISK_PREMIUM);
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
    let duplicate = forward[0].clone();
    forward.as_array_mut().unwrap().push(duplicate);
    assert_eq!(serde_json::from_value::<Vec<T>>(forward).unwrap().len(), 2);
}

#[test]
fn all_economics_contracts_are_bare_arrays_preserving_empty_rows() {
    assert!(
        serde_json::from_slice::<Vec<TreasuryRate>>(b"[]")
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_slice::<Vec<EconomicIndicatorObservation>>(b"[]")
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_slice::<Vec<EconomicCalendarEvent>>(b"[]")
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_slice::<Vec<MarketRiskPremium>>(b"[]")
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_value::<Vec<TreasuryRate>>(serde_json::json!({ "rates": [] })).is_err()
    );
}

#[test]
fn calendar_datetime_remains_strictly_timezone_less() {
    for invalid in ["2026-07-29T03:30:00Z", "2026-07-29 03:30:00+08:00"] {
        let mut value: serde_json::Value = serde_json::from_slice(CALENDAR).unwrap();
        value[0]["date"] = serde_json::json!(invalid);
        assert!(serde_json::from_value::<Vec<EconomicCalendarEvent>>(value).is_err());
    }
}
