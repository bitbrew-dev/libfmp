use libfmp::responses::company::{CompanyNote, CompanyProfile, StockPeer};

#[test]
fn documented_company_profile_decodes_every_exact_field_and_wire_type() {
    let rows: Vec<CompanyProfile> =
        serde_json::from_str(include_str!("fixtures/company_profile.json")).unwrap();

    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row.symbol.as_str(), "AAPL");
    assert_eq!(row.price, 331.85501);
    assert_eq!(row.market_cap, 4_874_072_686_740.0);
    assert_eq!(row.beta, 1.097);
    assert_eq!(row.last_dividend, 1.05);
    assert_eq!(row.range, "201.5-344.57");
    assert_eq!(row.change, -6.33498);
    assert_eq!(row.change_percentage, -1.8732);
    assert_eq!(row.volume, 28_718_014.0);
    assert_eq!(row.average_volume, 55_309_000.0);
    assert_eq!(row.company_name, "Apple Inc.");
    assert_eq!(row.currency.as_str(), "USD");
    assert_eq!(row.cik.as_str(), "0000320193");
    assert_eq!(row.isin.as_str(), "US0378331005");
    assert_eq!(row.cusip.as_str(), "037833100");
    assert_eq!(row.exchange_full_name, "NASDAQ Global Select");
    assert_eq!(row.exchange.as_str(), "NASDAQ");
    assert_eq!(row.industry.as_str(), "Consumer Electronics");
    assert_eq!(row.website, "https://www.apple.com");
    assert!(
        row.description
            .starts_with("Apple Inc. is a global technology corporation")
    );
    assert_eq!(row.ceo, "Timothy D. Cook");
    assert_eq!(row.sector.as_str(), "Technology");
    assert_eq!(row.country.as_str(), "US");
    assert_eq!(row.full_time_employees.as_str(), "166000");
    assert_eq!(row.phone, "(408) 996-1010");
    assert_eq!(row.address, "One Apple Park Way");
    assert_eq!(row.city, "Cupertino");
    assert_eq!(row.state, "CA");
    assert_eq!(row.zip, "95014");
    assert_eq!(
        row.image,
        "https://images.financialmodelingprep.com/symbol/AAPL.png"
    );
    assert_eq!(row.ipo_date.to_string(), "1980-12-12");
    assert!(!row.default_image);
    assert!(!row.is_etf);
    assert!(row.is_actively_trading);
    assert!(!row.is_adr);
    assert!(!row.is_fund);

    let wire = serde_json::to_value(row).unwrap();
    assert_eq!(wire["marketCap"], 4_874_072_686_740_u64);
    assert_eq!(wire["fullTimeEmployees"], "166000");
    assert!(wire["fullTimeEmployees"].is_string());
    assert_eq!(wire["cik"], "0000320193");
    assert_eq!(wire["changePercentage"], -1.8732);
    assert_eq!(wire["exchangeFullName"], "NASDAQ Global Select");
    assert_eq!(wire["ipoDate"], "1980-12-12");
    assert_eq!(wire["defaultImage"], false);
    assert_eq!(wire["isEtf"], false);
    assert_eq!(wire["isActivelyTrading"], true);
    assert_eq!(wire["isAdr"], false);
    assert_eq!(wire["isFund"], false);
    assert!(wire.get("market_cap").is_none());
}

#[test]
fn documented_company_note_and_stock_peer_preserve_exact_names() {
    let notes: Vec<CompanyNote> =
        serde_json::from_str(include_str!("fixtures/company_note.json")).unwrap();
    let peers: Vec<StockPeer> =
        serde_json::from_str(include_str!("fixtures/stock_peer.json")).unwrap();

    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].cik.as_str(), "0000320193");
    assert_eq!(notes[0].symbol.as_str(), "AAPL");
    assert_eq!(notes[0].title, "0.000% Notes due 2025");
    assert_eq!(notes[0].exchange.as_str(), "NASDAQ");

    assert_eq!(peers.len(), 1);
    assert_eq!(peers[0].symbol.as_str(), "GOOGL");
    assert_eq!(peers[0].company_name, "Alphabet Inc.");
    assert_eq!(peers[0].price, 333.84);
    assert_eq!(peers[0].market_cap, 4_040_168_831_718.0);

    let peer_wire = serde_json::to_value(&peers[0]).unwrap();
    assert_eq!(peer_wire["mktCap"], 4_040_168_831_718_u64);
    assert!(peer_wire.get("marketCap").is_none());
    assert!(peer_wire.get("mkt_cap").is_none());
}

#[test]
fn company_arrays_preserve_empty_multiple_unknown_and_large_values() {
    let empty = include_str!("fixtures/company_empty.json");
    let profiles: Vec<CompanyProfile> =
        serde_json::from_str(include_str!("fixtures/company_profile_multiple.json")).unwrap();
    let unknown: Vec<CompanyProfile> =
        serde_json::from_str(include_str!("fixtures/company_profile_unknown.json")).unwrap();

    assert!(
        serde_json::from_str::<Vec<CompanyProfile>>(empty)
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_str::<Vec<CompanyNote>>(empty)
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_str::<Vec<StockPeer>>(empty)
            .unwrap()
            .is_empty()
    );
    assert_eq!(profiles.len(), 2);
    assert_eq!(profiles[0].market_cap, 9_007_199_254_740_992.0);
    assert!(profiles[0].market_cap >= 2_f64.powi(53));
    assert_eq!(profiles[0].volume, u64::MAX as f64);
    assert_eq!(profiles[0].average_volume, 4_294_967_296.0);
    assert_eq!(profiles[0].full_time_employees.as_str(), "42");
    let large_wire = serde_json::to_value(&profiles[0]).unwrap();
    assert_eq!(large_wire["fullTimeEmployees"], "42");
    assert!(large_wire["fullTimeEmployees"].is_string());
    assert_eq!(profiles[0].sector.as_str(), "Future Sector");
    assert_eq!(profiles[0].industry.as_str(), "Future Industry");
    assert_eq!(unknown.len(), 1);
    assert_eq!(unknown[0].symbol.as_str(), "AAPL");
}
