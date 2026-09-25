use libfmp::responses::company::{
    AllSharesFloatRecord, CompanyShareFloat, MarketCapitalizationRecord,
};

#[test]
fn documented_market_capitalization_decodes_every_exact_field_and_wire_type() {
    let rows: Vec<MarketCapitalizationRecord> =
        serde_json::from_str(include_str!("fixtures/company_market_capitalization.json")).unwrap();

    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row.symbol.as_str(), "AAPL");
    assert_eq!(row.date.to_string(), "2026-07-30");
    assert_eq!(row.market_cap, 4_874_072_686_740.0);

    let wire = serde_json::to_value(row).unwrap();
    assert_eq!(wire["date"], "2026-07-30");
    assert_eq!(wire["marketCap"], 4_874_072_686_740_u64);
    assert!(wire["marketCap"].is_u64());
    assert!(wire.get("market_cap").is_none());

    let historical: Vec<MarketCapitalizationRecord> = serde_json::from_str(include_str!(
        "fixtures/company_historical_market_capitalization.json"
    ))
    .unwrap();
    assert_eq!(historical.len(), 1);
    assert_eq!(historical[0].symbol.as_str(), "AAPL");
    assert_eq!(historical[0].date.to_string(), "2026-07-30");
    assert_eq!(historical[0].market_cap, 4_879_177_245_542.0);
}

#[test]
fn documented_company_share_float_decodes_source_and_exact_wire_types() {
    let rows: Vec<CompanyShareFloat> =
        serde_json::from_str(include_str!("fixtures/company_shares_float.json")).unwrap();

    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row.symbol.as_str(), "AAPL");
    assert_eq!(row.date.to_string(), "2026-07-30 15:48:00");
    assert!((row.free_float - 99.83000000136171).abs() < 1e-12);
    assert_eq!(row.float_shares, 14_662_387_495);
    assert_eq!(row.outstanding_shares, 14_687_356_000);
    assert_eq!(
        row.source,
        "https://www.sec.gov/Archives/edgar/data/320193/000032019326000013/aapl-20260328.htm"
    );

    let wire = serde_json::to_value(row).unwrap();
    assert_eq!(wire["date"], "2026-07-30 15:48:00");
    assert_eq!(wire["floatShares"], 14_662_387_495_u64);
    assert_eq!(wire["outstandingShares"], 14_687_356_000_u64);
    assert!(wire["floatShares"].is_u64());
    assert!(wire["outstandingShares"].is_u64());
    assert!(wire.get("free_float").is_none());
    assert!(wire.get("float_shares").is_none());
}

#[test]
fn documented_all_share_float_is_a_separate_source_free_shape() {
    let rows: Vec<AllSharesFloatRecord> =
        serde_json::from_str(include_str!("fixtures/company_shares_float_all.json")).unwrap();

    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row.symbol.as_str(), "000001.SZ");
    assert_eq!(row.date.to_string(), "2026-07-29 14:23:30");
    assert_eq!(row.free_float, 41.40900000201062);
    assert_eq!(row.float_shares, 8_035_796_667);
    assert_eq!(row.outstanding_shares, 19_405_918_198);

    let wire = serde_json::to_value(row).unwrap();
    assert!(wire.get("source").is_none());
    assert!(
        serde_json::from_str::<Vec<CompanyShareFloat>>(include_str!(
            "fixtures/company_shares_float_all.json"
        ))
        .is_err()
    );
}

#[test]
fn market_data_arrays_preserve_empty_multiple_unknown_fields_and_u64_values() {
    assert!(
        serde_json::from_str::<Vec<MarketCapitalizationRecord>>("[]")
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_str::<Vec<CompanyShareFloat>>("[]")
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_str::<Vec<AllSharesFloatRecord>>("[]")
            .unwrap()
            .is_empty()
    );

    let market_caps: Vec<MarketCapitalizationRecord> = serde_json::from_str(
        r#"[
          {"symbol":"AAPL","date":"2026-07-30","marketCap":18446744073709551615,"future":true},
          {"symbol":"MSFT","date":"2026-07-30","marketCap":9007199254740993}
        ]"#,
    )
    .unwrap();
    assert_eq!(market_caps.len(), 2);
    assert_eq!(market_caps[0].market_cap, u64::MAX as f64);
    assert_eq!(market_caps[1].market_cap, 9_007_199_254_740_992.0);

    let company_floats: Vec<CompanyShareFloat> = serde_json::from_str(
        r#"[
          {"symbol":"AAPL","date":"2026-07-30 15:48:00","freeFloat":99.83,"floatShares":18446744073709551615,"outstandingShares":9007199254740993,"source":"https://www.sec.gov/example","future":{"nested":true}},
          {"symbol":"MSFT","date":"2026-07-30 15:48:00","freeFloat":98.1,"floatShares":1,"outstandingShares":2,"source":"https://www.sec.gov/example"}
        ]"#,
    )
    .unwrap();
    assert_eq!(company_floats.len(), 2);
    assert_eq!(company_floats[0].float_shares, u64::MAX);
    assert_eq!(company_floats[0].outstanding_shares, 9_007_199_254_740_993);

    let all_floats: Vec<AllSharesFloatRecord> = serde_json::from_str(
        r#"[
          {"symbol":"000001.SZ","date":"2026-07-29 14:23:30","freeFloat":41.409,"floatShares":18446744073709551615,"outstandingShares":9007199254740993,"future":null},
          {"symbol":"000002.SZ","date":"2026-07-29 14:23:30","freeFloat":42.0,"floatShares":3,"outstandingShares":4}
        ]"#,
    )
    .unwrap();
    assert_eq!(all_floats.len(), 2);
    assert_eq!(all_floats[0].float_shares, u64::MAX);
    assert_eq!(all_floats[0].outstanding_shares, 9_007_199_254_740_993);
}
