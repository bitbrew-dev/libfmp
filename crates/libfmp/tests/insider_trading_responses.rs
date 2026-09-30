use std::str::FromStr;

use libfmp::{
    responses::insider_trading::{
        BeneficialOwnershipAcquisition, InsiderReportingName, InsiderTrade, InsiderTradeStatistics,
        InsiderTransactionType,
    },
    types::{CalendarQuarter, CalendarYear, Date},
};
use serde::{Serialize, de::DeserializeOwned};

const LATEST: &[u8] = include_bytes!("fixtures/latest_insider_trades.json");
const SEARCH: &[u8] = include_bytes!("fixtures/searched_insider_trades.json");
const REPORTING_NAMES: &[u8] = include_bytes!("fixtures/insider_reporting_names.json");
const TRANSACTION_TYPES: &[u8] = include_bytes!("fixtures/insider_transaction_types.json");
const STATISTICS: &[u8] = include_bytes!("fixtures/insider_trade_statistics.json");
const OWNERSHIP: &[u8] = include_bytes!("fixtures/beneficial_ownership_acquisitions.json");

#[test]
fn exact_latest_and_search_fixtures_share_the_16_field_trade_row() {
    assert_field_count(LATEST, 16);
    assert_field_count(SEARCH, 16);
    let latest_source: serde_json::Value = serde_json::from_slice(LATEST).unwrap();
    let search_source: serde_json::Value = serde_json::from_slice(SEARCH).unwrap();
    let latest: Vec<InsiderTrade> = serde_json::from_value(latest_source).unwrap();
    let searched: Vec<InsiderTrade> = serde_json::from_value(search_source).unwrap();
    assert_eq!(latest, searched);
    assert_eq!(latest[0].symbol.as_str(), "TRMK");
    assert_eq!(latest[0].filing_date, Date::from_str("2026-07-30").unwrap());
    assert_eq!(
        latest[0].transaction_date,
        Date::from_str("2026-07-28").unwrap()
    );
    assert_eq!(latest[0].reporting_cik.as_str(), "0001661867");
    assert_eq!(latest[0].company_cik.as_str(), "0000036146");
    assert_eq!(
        latest[0].transaction_type.as_ref().unwrap().as_str(),
        "A-Award"
    );
    assert_eq!(latest[0].securities_owned, Some(62_959.0));
    assert_eq!(latest[0].securities_transacted, 1_608.0);
    assert_eq!(latest[0].price, 0.0);
    assert_eq!(latest[0].form_type.as_str(), "4");
}

#[test]
fn exact_reporting_name_and_transaction_taxonomy_fixtures_decode() {
    assert_field_count(REPORTING_NAMES, 2);
    assert_field_count(TRANSACTION_TYPES, 1);
    let names_source: serde_json::Value = serde_json::from_slice(REPORTING_NAMES).unwrap();
    let names: Vec<InsiderReportingName> = serde_json::from_value(names_source.clone()).unwrap();
    assert_eq!(names[0].reporting_cik.as_str(), "0001548760");
    assert_eq!(names[0].reporting_name, "Zuckerberg Mark");
    assert_eq!(serde_json::to_value(names).unwrap(), names_source);

    let types_source: serde_json::Value = serde_json::from_slice(TRANSACTION_TYPES).unwrap();
    let types: Vec<InsiderTransactionType> = serde_json::from_value(types_source.clone()).unwrap();
    assert_eq!(types[0].transaction_type.as_str(), "A-Award");
    assert_eq!(serde_json::to_value(types).unwrap(), types_source);
}

#[test]
fn exact_statistics_fixture_preserves_calendar_units_counts_and_decimals() {
    assert_field_count(STATISTICS, 13);
    let source: serde_json::Value = serde_json::from_slice(STATISTICS).unwrap();
    let rows: Vec<InsiderTradeStatistics> = serde_json::from_value(source.clone()).unwrap();
    assert_eq!(rows[0].year, CalendarYear(2026));
    assert_eq!(rows[0].quarter, CalendarQuarter::new(2).unwrap());
    assert_eq!(rows[0].acquired_transactions, 7);
    assert_eq!(rows[0].disposed_transactions, 40);
    assert_eq!(rows[0].acquired_disposed_ratio, 0.175);
    assert_eq!(rows[0].average_acquired, 43_314.142_9);
    assert_eq!(rows[0].average_disposed, 23_184.5);
    assert_eq!(rows[0].total_purchases, 0);
    assert_eq!(serde_json::to_value(rows).unwrap(), source);
}

#[test]
fn exact_ownership_fixture_preserves_dates_identifiers_and_quoted_numbers() {
    assert_field_count(OWNERSHIP, 15);
    let source: serde_json::Value = serde_json::from_slice(OWNERSHIP).unwrap();
    let rows: Vec<BeneficialOwnershipAcquisition> = serde_json::from_value(source.clone()).unwrap();
    let row = &rows[0];
    assert_eq!(row.cik.as_str(), "0000320193");
    assert_eq!(row.cusip.as_ref().unwrap().as_str(), "037833100");
    assert_eq!(row.accepted_date, Date::from_str("2026-04-29").unwrap());
    assert_eq!(row.sole_voting_power.as_str(), "0");
    assert_eq!(row.amount_beneficially_owned.as_str(), "1099168953");
    assert_eq!(row.percent_of_class.as_str(), "7.48");
    assert_eq!(serde_json::to_value(rows).unwrap(), source);
}

#[test]
fn counts_preserve_u64_quantities_decode_as_f64_and_integer_prices_decode_as_prices() {
    let mut trade: serde_json::Value = serde_json::from_slice(LATEST).unwrap();
    trade[0]["securitiesOwned"] = serde_json::json!(u64::MAX);
    trade[0]["securitiesTransacted"] = serde_json::json!(u64::MAX);
    trade[0]["price"] = serde_json::json!(225);
    let rows: Vec<InsiderTrade> = serde_json::from_value(trade).unwrap();
    assert_eq!(rows[0].securities_owned, Some(u64::MAX as f64));
    assert_eq!(rows[0].securities_transacted, u64::MAX as f64);
    assert_eq!(rows[0].price, 225.0);

    let count_fields = [
        "acquiredTransactions",
        "disposedTransactions",
        "totalPurchases",
        "totalSales",
    ];
    for field in count_fields {
        let mut source: serde_json::Value = serde_json::from_slice(STATISTICS).unwrap();
        source[0][field] = serde_json::json!(u64::MAX);
        let rows: Vec<InsiderTradeStatistics> = serde_json::from_value(source.clone()).unwrap();
        assert_eq!(serde_json::to_value(rows).unwrap()[0][field], u64::MAX);
    }
    for field in ["totalAcquired", "totalDisposed"] {
        let mut source: serde_json::Value = serde_json::from_slice(STATISTICS).unwrap();
        source[0][field] = serde_json::json!(1_500.5);
        let rows: Vec<InsiderTradeStatistics> = serde_json::from_value(source).unwrap();
        assert_eq!(serde_json::to_value(rows).unwrap()[0][field], 1_500.5);
    }
}

#[test]
fn nullable_trade_members_decode_null_and_empty_type_as_none() {
    let mut source: serde_json::Value = serde_json::from_slice(LATEST).unwrap();
    source[0]["transactionType"] = serde_json::json!("");
    source[0]["securitiesOwned"] = serde_json::Value::Null;
    source[0]["directOrIndirect"] = serde_json::Value::Null;
    let rows: Vec<InsiderTrade> = serde_json::from_value(source.clone()).unwrap();
    assert_eq!(rows[0].transaction_type, None);
    assert_eq!(rows[0].securities_owned, None);
    assert_eq!(rows[0].direct_or_indirect, None);
    let wire = serde_json::to_value(&rows).unwrap();
    assert!(wire[0]["transactionType"].is_null());
    assert!(wire[0]["securitiesOwned"].is_null());
    assert!(wire[0]["directOrIndirect"].is_null());

    source[0]["transactionType"] = serde_json::Value::Null;
    let rows: Vec<InsiderTrade> = serde_json::from_value(source.clone()).unwrap();
    assert_eq!(rows[0].transaction_type, None);

    source[0]["transactionType"] = serde_json::json!(" ");
    assert!(serde_json::from_value::<Vec<InsiderTrade>>(source).is_err());
}

#[test]
fn nullable_ownership_members_decode_null_as_none() {
    let mut source: serde_json::Value = serde_json::from_slice(OWNERSHIP).unwrap();
    for field in [
        "cusip",
        "citizenshipOrPlaceOfOrganization",
        "sharedVotingPower",
    ] {
        source[0][field] = serde_json::Value::Null;
    }
    let rows: Vec<BeneficialOwnershipAcquisition> = serde_json::from_value(source.clone()).unwrap();
    assert_eq!(rows[0].cusip, None);
    assert_eq!(rows[0].citizenship_or_place_of_organization, None);
    assert_eq!(rows[0].shared_voting_power, None);
    assert_eq!(serde_json::to_value(rows).unwrap(), source);
}

#[test]
fn quoted_numeric_ownership_fields_preserve_exact_decimal_text() {
    let fields = [
        "soleVotingPower",
        "sharedVotingPower",
        "soleDispositivePower",
        "sharedDispositivePower",
        "amountBeneficiallyOwned",
        "percentOfClass",
    ];
    for (index, field) in fields.into_iter().enumerate() {
        let representation = format!("{}.{:02}", u64::MAX, index);
        let mut source: serde_json::Value = serde_json::from_slice(OWNERSHIP).unwrap();
        source[0][field] = serde_json::Value::String(representation.clone());
        let rows: Vec<BeneficialOwnershipAcquisition> =
            serde_json::from_value(source.clone()).unwrap();
        assert_eq!(
            serde_json::to_value(rows).unwrap()[0][field],
            representation
        );
    }
}

#[test]
fn all_fields_are_required_non_null_and_unknowns_are_accepted() {
    let trade_nullable = ["transactionType", "securitiesOwned", "directOrIndirect"];
    assert_contract::<InsiderTrade>(LATEST, &trade_nullable);
    assert_contract::<InsiderTrade>(SEARCH, &trade_nullable);
    assert_contract::<InsiderReportingName>(REPORTING_NAMES, &[]);
    assert_contract::<InsiderTransactionType>(TRANSACTION_TYPES, &[]);
    assert_contract::<InsiderTradeStatistics>(STATISTICS, &[]);
    assert_contract::<BeneficialOwnershipAcquisition>(
        OWNERSHIP,
        &[
            "cusip",
            "citizenshipOrPlaceOfOrganization",
            "sharedVotingPower",
        ],
    );
}

#[test]
fn all_six_contracts_are_bare_arrays_preserving_empty_and_multiple_rows() {
    assert_bare_array::<InsiderTrade>(LATEST);
    assert_bare_array::<InsiderTrade>(SEARCH);
    assert_bare_array::<InsiderReportingName>(REPORTING_NAMES);
    assert_bare_array::<InsiderTransactionType>(TRANSACTION_TYPES);
    assert_bare_array::<InsiderTradeStatistics>(STATISTICS);
    assert_bare_array::<BeneficialOwnershipAcquisition>(OWNERSHIP);
    assert!(
        serde_json::from_value::<Vec<InsiderTrade>>(serde_json::json!({ "trades": [] })).is_err()
    );
}

fn assert_contract<T: DeserializeOwned>(fixture: &[u8], nullable: &[&str]) {
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
            "{key} unexpectedly accepted when missing"
        );

        let mut null = source.clone();
        null[0][&key] = serde_json::Value::Null;
        assert_eq!(
            serde_json::from_value::<Vec<T>>(null).is_ok(),
            nullable.contains(&key.as_str()),
            "{key} null acceptance differs from the nullable list"
        );
    }

    let mut unknown = source;
    unknown[0]["futureProviderField"] = serde_json::json!({ "kept": "irrelevant" });
    assert!(serde_json::from_value::<Vec<T>>(unknown).is_ok());
}

fn assert_bare_array<T>(fixture: &[u8])
where
    T: DeserializeOwned + Serialize,
{
    assert!(serde_json::from_value::<Vec<T>>(serde_json::json!([])).is_ok());
    let source: serde_json::Value = serde_json::from_slice(fixture).unwrap();
    let row = source[0].clone();
    let rows: Vec<T> = serde_json::from_value(serde_json::json!([row.clone(), row])).unwrap();
    assert_eq!(rows.len(), 2);
}

fn assert_field_count(fixture: &[u8], expected: usize) {
    let source: serde_json::Value = serde_json::from_slice(fixture).unwrap();
    assert_eq!(source[0].as_object().unwrap().len(), expected);
}
