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
    assert_eq!(row.beta, Some(1.097));
    assert_eq!(row.price, Some(332.64001));
    assert_eq!(row.last_annual_dividend, Some(1.05));
    assert_eq!(row.volume, 29_909_012.0);
    assert_eq!(row.exchange, "NASDAQ Global Select");
    assert_eq!(row.exchange_short_name.as_str(), "NASDAQ");
    assert_eq!(row.country.as_ref().unwrap().as_str(), "US");
    assert!(!row.is_etf);
    assert_eq!(row.is_fund, Some(false));
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
    assert_eq!(multiple[1].is_fund, Some(true));
    assert_eq!(unknown.len(), 1);
    assert_eq!(unknown[0].symbol.as_str(), "AAPL");
}

#[test]
fn screener_page_with_a_null_beta_row_decodes_every_row() {
    let rows: Vec<CompanyScreenerResult> = serde_json::from_str(include_str!(
        "fixtures/company_screener_null_beta_synthetic.json"
    ))
    .unwrap();

    assert_eq!(rows.len(), 40);
    assert_eq!(rows[37].symbol.as_str(), "R37");
    assert_eq!(rows[37].beta, None);
    assert_eq!(rows[36].beta, Some(1.097));
}

#[test]
fn nullable_screener_members_decode_null_to_none_and_encode_null() {
    let mut wire: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/company_screener.json")).unwrap();
    for member in ["beta", "price", "lastAnnualDividend", "country", "isFund"] {
        wire[0][member] = serde_json::Value::Null;
    }
    let rows: Vec<CompanyScreenerResult> = serde_json::from_value(wire).unwrap();
    let row = &rows[0];

    assert_eq!(row.beta, None);
    assert_eq!(row.price, None);
    assert_eq!(row.last_annual_dividend, None);
    assert_eq!(row.country, None);
    assert_eq!(row.is_fund, None);

    let encoded = serde_json::to_value(row).unwrap();
    for member in ["beta", "price", "lastAnnualDividend", "country", "isFund"] {
        assert!(encoded[member].is_null(), "{member} must re-encode as null");
    }
}
