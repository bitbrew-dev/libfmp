use libfmp::{
    responses::search::{
        CikSearchResult, CusipSearchResult, ExchangeVariant, IsinSearchResult, NameSearchResult,
        SymbolSearchResult,
    },
    types::Date,
};

#[test]
fn documented_symbol_and_name_results_decode_their_exact_fields() {
    let symbols: Vec<SymbolSearchResult> =
        serde_json::from_str(include_str!("fixtures/search_symbol.json")).unwrap();
    let names: Vec<NameSearchResult> =
        serde_json::from_str(include_str!("fixtures/search_name.json")).unwrap();

    assert_eq!(symbols.len(), 1);
    assert_eq!(symbols[0].symbol.as_str(), "AAPL");
    assert_eq!(symbols[0].name, "Apple Inc.");
    assert_eq!(symbols[0].currency.as_str(), "USD");
    assert_eq!(symbols[0].exchange_full_name, "NASDAQ Global Select");
    assert_eq!(symbols[0].exchange.as_str(), "NASDAQ");

    assert_eq!(names.len(), 1);
    assert_eq!(names[0].symbol.as_str(), "AAGUSD");
    assert_eq!(names[0].name, "AAG USD");
    assert_eq!(names[0].currency.as_str(), "USD");
    assert_eq!(names[0].exchange_full_name, "CCC");
    assert_eq!(names[0].exchange.as_str(), "CRYPTO");

    let encoded = serde_json::to_value(&symbols[0]).unwrap();
    assert_eq!(encoded["exchangeFullName"], "NASDAQ Global Select");
    assert!(encoded.get("companyName").is_none());
}

#[test]
fn documented_identifier_results_preserve_wire_names_and_large_caps() {
    let cik: Vec<CikSearchResult> =
        serde_json::from_str(include_str!("fixtures/search_cik.json")).unwrap();
    let cusip: Vec<CusipSearchResult> =
        serde_json::from_str(include_str!("fixtures/search_cusip.json")).unwrap();
    let isin: Vec<IsinSearchResult> =
        serde_json::from_str(include_str!("fixtures/search_isin.json")).unwrap();

    assert_eq!(cik[0].company_name, "Apple Inc.");
    assert_eq!(cik[0].cik.as_str(), "0000320193");
    assert_eq!(cik[0].exchange_full_name, "NASDAQ Global Select");
    assert_eq!(cik[0].exchange.as_str(), "NASDAQ");
    assert_eq!(cik[0].currency.as_str(), "USD");

    assert_eq!(cusip[0].symbol.as_str(), "APC.F");
    assert_eq!(cusip[0].company_name, "Apple Inc.");
    assert_eq!(cusip[0].cusip.as_str(), "037833100");
    assert_eq!(cusip[0].market_cap, 4_227_021_056_800);
    assert!(cusip[0].market_cap > u64::from(u32::MAX));

    assert_eq!(isin[0].symbol.as_str(), "AAPL");
    assert_eq!(isin[0].name, "Apple Inc.");
    assert_eq!(isin[0].isin.as_str(), "US0378331005");
    assert_eq!(isin[0].market_cap, 4_874_072_686_740);
    assert!(isin[0].market_cap > u64::from(u32::MAX));

    let cik_wire = serde_json::to_value(&cik[0]).unwrap();
    let cusip_wire = serde_json::to_value(&cusip[0]).unwrap();
    let isin_wire = serde_json::to_value(&isin[0]).unwrap();
    assert_eq!(cik_wire["companyName"], "Apple Inc.");
    assert!(cik_wire.get("name").is_none());
    assert_eq!(cusip_wire["marketCap"], 4_227_021_056_800_u64);
    assert_eq!(isin_wire["marketCap"], 4_874_072_686_740_u64);
}

#[test]
fn documented_exchange_variant_decodes_every_field_and_inverted_exchange_names() {
    let rows: Vec<ExchangeVariant> =
        serde_json::from_str(include_str!("fixtures/search_exchange_variants.json")).unwrap();

    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row.symbol.as_str(), "AAPL");
    assert_eq!(row.price, 331.85501);
    assert_eq!(row.beta, 1.097);
    assert_eq!(row.vol_avg, 55_309_000);
    assert_eq!(row.market_cap, 4_874_072_686_740);
    assert!(row.market_cap > u64::from(u32::MAX));
    assert_eq!(row.last_div, 1.05);
    assert_eq!(row.range, "201.5-344.57");
    assert_eq!(row.changes, -6.33498);
    assert_eq!(row.company_name, "Apple Inc.");
    assert_eq!(row.currency.as_str(), "USD");
    assert_eq!(row.cik.as_str(), "0000320193");
    assert_eq!(row.isin.as_str(), "US0378331005");
    assert_eq!(row.cusip.as_str(), "037833100");
    assert_eq!(row.exchange, "NASDAQ Global Select");
    assert_eq!(row.exchange_short_name.as_str(), "NASDAQ");
    assert_eq!(row.industry, "Consumer Electronics");
    assert_eq!(row.website, "https://www.apple.com");
    assert!(row.description.starts_with("Apple Inc. is a global"));
    assert_eq!(row.ceo, "Timothy D. Cook");
    assert_eq!(row.sector, "Technology");
    assert_eq!(row.country.as_str(), "US");
    assert_eq!(row.full_time_employees, "166000");
    assert_eq!(row.phone, "(408) 996-1010");
    assert_eq!(row.address, "One Apple Park Way");
    assert_eq!(row.city, "Cupertino");
    assert_eq!(row.state, "CA");
    assert_eq!(row.zip, "95014");
    assert_eq!(row.dcf_diff, 191.60731);
    assert_eq!(row.dcf, 140.70269296445176);
    assert_eq!(
        row.image,
        "https://images.financialmodelingprep.com/symbol/AAPL.png"
    );
    assert_eq!(row.ipo_date, "1980-12-12".parse::<Date>().unwrap());
    assert!(!row.default_image);
    assert!(!row.is_etf);
    assert!(row.is_actively_trading);
    assert!(!row.is_adr);
    assert!(!row.is_fund);

    let encoded = serde_json::to_value(row).unwrap();
    assert_eq!(encoded["volAvg"], 55_309_000);
    assert_eq!(encoded["mktCap"], 4_874_072_686_740_u64);
    assert_eq!(encoded["changes"], -6.33498);
    assert_eq!(encoded["exchange"], "NASDAQ Global Select");
    assert_eq!(encoded["exchangeShortName"], "NASDAQ");
    assert_eq!(encoded["fullTimeEmployees"], "166000");
    assert_eq!(encoded["range"], "201.5-344.57");
    assert_eq!(encoded["zip"], "95014");
    assert!(encoded.get("marketCap").is_none());
    assert!(encoded.get("change").is_none());
}

#[test]
fn search_arrays_preserve_empty_multiple_and_unknown_field_shapes() {
    let empty: Vec<SymbolSearchResult> =
        serde_json::from_str(include_str!("fixtures/search_empty.json")).unwrap();
    let multiple: Vec<SymbolSearchResult> =
        serde_json::from_str(include_str!("fixtures/search_symbol_multiple.json")).unwrap();
    let unknown: Vec<SymbolSearchResult> =
        serde_json::from_str(include_str!("fixtures/search_symbol_unknown.json")).unwrap();

    assert!(empty.is_empty());
    assert_eq!(multiple.len(), 2);
    assert_eq!(multiple[0].symbol.as_str(), "000001.SZ");
    assert_eq!(multiple[1].symbol.as_str(), "^VIX");
    assert_eq!(unknown.len(), 1);
    assert_eq!(unknown[0].symbol.as_str(), "AAPL");
}
