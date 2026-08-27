use libfmp::responses::directory::{
    AvailableCountry, AvailableExchange, AvailableIndustry, AvailableSector,
};

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
    assert_eq!(exchange.country_code.as_str(), "US");
    assert_eq!(exchange.symbol_suffix, "N/A");
    assert_eq!(exchange.delay, "Real-time");
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
