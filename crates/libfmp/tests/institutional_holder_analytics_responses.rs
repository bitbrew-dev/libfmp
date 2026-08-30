use std::str::FromStr;

use libfmp::{
    responses::institutional_ownership::InstitutionalHolderAnalytics,
    types::{Cik, Cusip, Date, Ticker},
};

const ANALYTICS: &[u8] = include_bytes!("fixtures/institutional_holder_analytics.json");

#[test]
fn exact_fixture_decodes_all_39_required_fields_and_wire_casing() {
    let source: serde_json::Value = serde_json::from_slice(ANALYTICS).unwrap();
    assert_eq!(source[0].as_object().unwrap().len(), 39);

    let rows: Vec<InstitutionalHolderAnalytics> = serde_json::from_value(source.clone()).unwrap();
    assert_eq!(
        rows,
        [InstitutionalHolderAnalytics {
            date: Date::from_str("2023-09-30").unwrap(),
            cik: Cik::new("0000102909").unwrap(),
            filing_date: Date::from_str("2023-12-18").unwrap(),
            investor_name: "VANGUARD GROUP INC".to_owned(),
            symbol: Ticker::new("AAPL").unwrap(),
            security_name: "APPLE INC".to_owned(),
            type_of_security: "COM".to_owned(),
            security_cusip: Cusip::new("037833100").unwrap(),
            shares_type: "SH".to_owned(),
            put_call_share: "Share".to_owned(),
            investment_discretion: "SOLE".to_owned(),
            industry_title: "ELECTRONIC COMPUTERS".to_owned(),
            weight: 5.4673,
            last_weight: 5.996,
            change_in_weight: -0.5287,
            change_in_weight_percentage: -8.8175,
            market_value: 222_572_509_140,
            last_market_value: 252_876_459_509,
            change_in_market_value: -30_303_950_369,
            change_in_market_value_percentage: -11.9837,
            shares_number: 1_299_997_133,
            last_shares_number: 1_303_688_506,
            change_in_shares_number: -3_691_373,
            change_in_shares_number_percentage: -0.2831,
            quarter_end_price: 171.21,
            avg_price_paid: 20.65,
            is_new: false,
            is_sold_out: false,
            ownership: 8.3336,
            last_ownership: 8.305,
            change_in_ownership: 0.0286,
            change_in_ownership_percentage: 0.3445,
            holding_period: 75,
            first_added: Date::from_str("2005-03-31").unwrap(),
            performance: -29_671_950_396,
            performance_percentage: -11.7338,
            last_performance: 38_078_179_274,
            change_in_performance: -67_750_129_670,
            is_counted_for_performance: true,
        }]
    );
    assert_eq!(rows[0].cik.as_str(), "0000102909");
    assert_eq!(rows[0].security_cusip.as_str(), "037833100");
    assert_eq!(serde_json::to_value(rows).unwrap(), source);
}

#[test]
fn integer_domains_preserve_above_u32_above_2pow53_and_negative_values() {
    let mut source: serde_json::Value = serde_json::from_slice(ANALYTICS).unwrap();
    source[0]["marketValue"] = serde_json::json!(9_007_199_254_740_993_u64);
    source[0]["lastMarketValue"] = serde_json::json!(u64::MAX);
    source[0]["sharesNumber"] = serde_json::json!(4_294_967_296_u64);
    source[0]["lastSharesNumber"] = serde_json::json!(9_007_199_254_740_993_u64);
    source[0]["changeInMarketValue"] = serde_json::json!(i64::MIN);
    source[0]["changeInSharesNumber"] = serde_json::json!(-4_294_967_297_i64);
    source[0]["performance"] = serde_json::json!(-9_007_199_254_740_993_i64);
    source[0]["lastPerformance"] = serde_json::json!(9_007_199_254_740_993_i64);
    source[0]["changeInPerformance"] = serde_json::json!(i64::MIN);
    source[0]["holdingPeriod"] = serde_json::json!(u64::MAX);

    let rows: Vec<InstitutionalHolderAnalytics> = serde_json::from_value(source).unwrap();
    assert_eq!(rows[0].market_value, 9_007_199_254_740_993);
    assert_eq!(rows[0].last_market_value, u64::MAX);
    assert_eq!(rows[0].shares_number, 4_294_967_296);
    assert_eq!(rows[0].last_shares_number, 9_007_199_254_740_993);
    assert_eq!(rows[0].change_in_market_value, i64::MIN);
    assert_eq!(rows[0].change_in_shares_number, -4_294_967_297);
    assert_eq!(rows[0].performance, -9_007_199_254_740_993);
    assert_eq!(rows[0].last_performance, 9_007_199_254_740_993);
    assert_eq!(rows[0].change_in_performance, i64::MIN);
    assert_eq!(rows[0].holding_period, u64::MAX);
}

#[test]
fn ratios_are_raw_f64_values_and_prices_remain_prices() {
    let mut source: serde_json::Value = serde_json::from_slice(ANALYTICS).unwrap();
    source[0]["weight"] = serde_json::json!(125.25);
    source[0]["changeInWeight"] = serde_json::json!(-101.5);
    source[0]["changeInWeightPercentage"] = serde_json::json!(-250.75);
    source[0]["changeInMarketValuePercentage"] = serde_json::json!(1.5);
    source[0]["changeInSharesNumberPercentage"] = serde_json::json!(-0.0001);
    source[0]["ownership"] = serde_json::json!(150.0);
    source[0]["changeInOwnership"] = serde_json::json!(-1.25);
    source[0]["performancePercentage"] = serde_json::json!(-999.5);
    source[0]["quarterEndPrice"] = serde_json::json!(0.125);
    source[0]["avgPricePaid"] = serde_json::json!(10_000.75);

    let rows: Vec<InstitutionalHolderAnalytics> = serde_json::from_value(source).unwrap();
    assert_eq!(rows[0].weight, 125.25);
    assert_eq!(rows[0].change_in_weight, -101.5);
    assert_eq!(rows[0].change_in_weight_percentage, -250.75);
    assert_eq!(rows[0].change_in_market_value_percentage, 1.5);
    assert_eq!(rows[0].change_in_shares_number_percentage, -0.0001);
    assert_eq!(rows[0].ownership, 150.0);
    assert_eq!(rows[0].change_in_ownership, -1.25);
    assert_eq!(rows[0].performance_percentage, -999.5);
    assert_eq!(rows[0].quarter_end_price, 0.125);
    assert_eq!(rows[0].avg_price_paid, 10_000.75);
}

#[test]
fn every_field_is_required_non_null_dates_are_date_only_and_unknowns_are_accepted() {
    let source: serde_json::Value = serde_json::from_slice(ANALYTICS).unwrap();
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
            serde_json::from_value::<Vec<InstitutionalHolderAnalytics>>(missing).is_err(),
            "accepted missing {key}"
        );

        let mut null = source.clone();
        null[0][&key] = serde_json::Value::Null;
        assert!(
            serde_json::from_value::<Vec<InstitutionalHolderAnalytics>>(null).is_err(),
            "accepted null {key}"
        );
    }

    for field in ["date", "filingDate", "firstAdded"] {
        let mut timestamp = source.clone();
        timestamp[0][field] = serde_json::json!("2023-09-30 00:00:00");
        assert!(serde_json::from_value::<Vec<InstitutionalHolderAnalytics>>(timestamp).is_err());
    }

    let mut forward = source;
    forward[0]["futureProviderField"] = serde_json::json!({ "nested": [1, true, null] });
    let duplicate = forward[0].clone();
    forward.as_array_mut().unwrap().push(duplicate);
    assert_eq!(
        serde_json::from_value::<Vec<InstitutionalHolderAnalytics>>(forward)
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn contract_is_a_bare_vec_preserving_empty_and_rejecting_wrappers() {
    assert!(
        serde_json::from_slice::<Vec<InstitutionalHolderAnalytics>>(b"[]")
            .unwrap()
            .is_empty()
    );
    assert!(
        serde_json::from_value::<Vec<InstitutionalHolderAnalytics>>(serde_json::json!({
            "holders": []
        }))
        .is_err()
    );
}
