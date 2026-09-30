use libfmp::responses::directory::{
    AvailableCountry, AvailableExchange, AvailableIndustry, AvailableSector,
};
use serde_json::{Value, json};

#[test]
fn documented_taxonomy_rows_decode_exact_field_names_and_open_values() {
    let exchanges: Vec<AvailableExchange> =
        serde_json::from_str(include_str!("fixtures/directory_available_exchanges.json")).unwrap();
    let sectors: Vec<AvailableSector> =
        serde_json::from_str(include_str!("fixtures/directory_available_sectors.json")).unwrap();
    let industries: Vec<AvailableIndustry> =
        serde_json::from_str(include_str!("fixtures/directory_available_industries.json")).unwrap();
    let countries: Vec<AvailableCountry> =
        serde_json::from_str(include_str!("fixtures/directory_available_countries.json")).unwrap();

    let exchange = &exchanges[0];
    assert_eq!(exchange.exchange.as_str(), "AMEX");
    assert_eq!(exchange.name, "New York Stock Exchange Arca");
    assert_eq!(exchange.country_name, "United States of America");
    assert_eq!(exchange.country_code.as_ref().unwrap().as_str(), "US");
    assert_eq!(exchange.symbol_suffix, "N/A");
    assert_eq!(exchange.delay.as_deref(), Some("Real-time"));
    assert_eq!(sectors[0].sector.as_str(), "Basic Materials");
    assert_eq!(industries[0].industry.as_str(), "Steel");
    assert_eq!(countries[0].country.as_str(), "FK");

    let wire = serde_json::to_value(exchange).unwrap();
    assert_eq!(wire["countryName"], "United States of America");
    assert_eq!(wire["countryCode"], "US");
    assert_eq!(wire["symbolSuffix"], "N/A");
    assert_eq!(wire["delay"], "Real-time");
}

#[test]
fn available_exchange_blank_country_code_and_null_delay_decode_as_none() {
    let row = json!({
        "exchange": "CRYPTO",
        "name": "Cryptocurrency",
        "countryName": "",
        "countryCode": "",
        "symbolSuffix": "N/A",
        "delay": null,
    });
    let decoded: AvailableExchange = serde_json::from_value(row.clone()).unwrap();
    assert_eq!(decoded.country_name, "");
    assert_eq!(decoded.country_code, None);
    assert_eq!(decoded.delay, None);
    let wire = serde_json::to_value(&decoded).unwrap();
    assert_eq!(wire["countryName"], "");
    assert!(wire["countryCode"].is_null());
    assert!(wire["delay"].is_null());

    let mut null_code = row.clone();
    null_code["countryCode"] = Value::Null;
    assert_eq!(
        serde_json::from_value::<AvailableExchange>(null_code)
            .unwrap()
            .country_code,
        None
    );

    for field in ["countryCode", "delay"] {
        let mut missing = row.clone();
        missing.as_object_mut().unwrap().remove(field);
        assert!(serde_json::from_value::<AvailableExchange>(missing).is_err());
    }
}

#[test]
fn every_taxonomy_response_contract_preserves_empty_arrays() {
    let empty = include_str!("fixtures/directory_empty.json");

    assert!(
        serde_json::from_str::<Vec<AvailableExchange>>(empty)
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_str::<Vec<AvailableSector>>(empty)
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_str::<Vec<AvailableIndustry>>(empty)
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_str::<Vec<AvailableCountry>>(empty)
            .unwrap()
            .is_empty()
    );
}
