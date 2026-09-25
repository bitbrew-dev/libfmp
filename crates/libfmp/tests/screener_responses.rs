use libfmp::responses::screener::CompanyScreenerResult;

#[test]
fn documented_company_screener_entry_decodes_every_exact_field() {
    let rows: Vec<CompanyScreenerResult> =
        serde_json::from_str(include_str!("fixtures/company_screener.json")).unwrap();

    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row.symbol.as_str(), "AAPL");
    assert_eq!(row.company_name, "Apple Inc.");
    assert_eq!(row.market_cap, 4_885_602_246_714.0);
    assert_eq!(row.sector.as_str(), "Technology");
    assert_eq!(row.industry.as_str(), "Consumer Electronics");
    assert_eq!(row.beta, 1.097);
    assert_eq!(row.price, 332.64001);
    assert_eq!(row.last_annual_dividend, 1.05);
    assert_eq!(row.volume, 29_909_012.0);
    assert_eq!(row.exchange, "NASDAQ Global Select");
    assert_eq!(row.exchange_short_name.as_str(), "NASDAQ");
    assert_eq!(row.country.as_str(), "US");
    assert!(!row.is_etf);
    assert!(!row.is_fund);
    assert!(row.is_actively_trading);

    let encoded = serde_json::to_value(row).unwrap();
    assert_eq!(encoded["companyName"], "Apple Inc.");
    assert_eq!(encoded["marketCap"], 4_885_602_246_714_u64);
    assert_eq!(encoded["lastAnnualDividend"], 1.05);
    assert_eq!(encoded["exchangeShortName"], "NASDAQ");
    assert_eq!(encoded["isEtf"], false);
    assert_eq!(encoded["isFund"], false);
    assert_eq!(encoded["isActivelyTrading"], true);
    assert!(encoded.get("company_name").is_none());
}

#[test]
fn screener_arrays_preserve_empty_multiple_unknown_and_large_integer_values() {
    let empty: Vec<CompanyScreenerResult> =
        serde_json::from_str(include_str!("fixtures/company_screener_empty.json")).unwrap();
    let multiple: Vec<CompanyScreenerResult> =
        serde_json::from_str(include_str!("fixtures/company_screener_multiple.json")).unwrap();
    let unknown: Vec<CompanyScreenerResult> =
        serde_json::from_str(include_str!("fixtures/company_screener_unknown.json")).unwrap();

    assert!(empty.is_empty());
    assert_eq!(multiple.len(), 2);
    assert_eq!(multiple[0].market_cap, 9_007_199_254_740_992.0);
    assert!(multiple[0].market_cap >= 2_f64.powi(53));
    assert_eq!(multiple[0].volume, u64::MAX as f64);
    assert_eq!(multiple[0].sector.as_str(), "Future Sector");
    assert_eq!(multiple[1].market_cap, 4_294_967_296.0);
    assert_eq!(multiple[1].volume, 4_294_967_296.0);
    assert!(multiple[1].is_fund);
    assert_eq!(unknown.len(), 1);
    assert_eq!(unknown[0].symbol.as_str(), "AAPL");
}
