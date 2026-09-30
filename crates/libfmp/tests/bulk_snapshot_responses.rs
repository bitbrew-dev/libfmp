use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Map, Value, json};

use libfmp::{
    responses::{
        bulk::{
            BulkDcfValuation, BulkEtfHolding, BulkFinancialScore, BulkPriceTargetSummary,
            BulkStockRating, BulkUpgradesDowngradesConsensus,
        },
        company::CompanyProfile,
    },
    types::Ticker,
};

const PROFILE: &[u8] = include_bytes!("fixtures/bulk_company_profiles.json");
const RATING: &[u8] = include_bytes!("fixtures/bulk_stock_ratings.json");
const DCF: &[u8] = include_bytes!("fixtures/bulk_dcf_valuations.json");
const SCORES: &[u8] = include_bytes!("fixtures/bulk_financial_scores.json");
const TARGET: &[u8] = include_bytes!("fixtures/bulk_price_target_summaries.json");
const ETF: &[u8] = include_bytes!("fixtures/bulk_etf_holdings.json");
const CONSENSUS: &[u8] = include_bytes!("fixtures/bulk_upgrades_downgrades_consensus.json");

#[test]
fn exact_outer_md_fixtures_round_trip_with_exact_field_counts() {
    assert_exact::<CompanyProfile>(PROFILE, 36);
    assert_exact::<BulkStockRating>(RATING, 9);
    assert_exact::<BulkDcfValuation>(DCF, 4);
    assert_exact::<BulkFinancialScore>(SCORES, 11);
    assert_exact::<BulkPriceTargetSummary>(TARGET, 10);
    assert_exact::<BulkEtfHolding>(ETF, 9);
    assert_exact::<BulkUpgradesDowngradesConsensus>(CONSENSUS, 7);
}

#[test]
fn every_documented_field_is_required_non_null_and_unknown_fields_are_tolerated() {
    assert_required_nullable::<CompanyProfile>(
        PROFILE,
        &["cik", "cusip", "fullTimeEmployees", "phone", "ipoDate"],
    );
    assert_required::<BulkStockRating>(RATING);
    assert_required::<BulkDcfValuation>(DCF);
    assert_required::<BulkFinancialScore>(SCORES);
    assert_required::<BulkPriceTargetSummary>(TARGET);
    assert_required::<BulkEtfHolding>(ETF);
    assert_required::<BulkUpgradesDowngradesConsensus>(CONSENSUS);
}

#[test]
fn numeric_string_fields_reject_json_numbers_in_every_new_contract() {
    assert_number_rejected::<BulkStockRating>(RATING, "discountedCashFlowScore");
    assert_number_rejected::<BulkDcfValuation>(DCF, "dcf");
    assert_number_rejected::<BulkDcfValuation>(DCF, "Stock Price");
    assert_number_rejected::<BulkFinancialScore>(SCORES, "altmanZScore");
    assert_number_rejected::<BulkPriceTargetSummary>(TARGET, "allTimeAvgPriceTarget");
    assert_number_rejected::<BulkEtfHolding>(ETF, "marketValue");
    assert_number_rejected::<BulkUpgradesDowngradesConsensus>(CONSENSUS, "strongBuy");
}

#[test]
fn malformed_documented_strings_and_wire_keys_are_preserved_verbatim() {
    let target = rows::<BulkPriceTargetSummary>(TARGET).remove(0);
    assert_eq!(target.publishers, "[\"\"TheFly\"");

    let etf = rows::<BulkEtfHolding>(ETF).remove(0);
    assert_eq!(etf.cusip, "");
    assert_eq!(etf.last_updated_raw, "2024-09-06\"");
    let etf_json = serde_json::to_value(etf).unwrap();
    assert_eq!(etf_json["lastUpdated\""], "2024-09-06\"");
    assert!(etf_json.get("lastUpdated").is_none());

    let consensus = rows::<BulkUpgradesDowngradesConsensus>(CONSENSUS).remove(0);
    assert_eq!(consensus.symbol, "");
}

#[test]
fn company_profile_is_the_existing_typed_contract_and_new_symbols_are_typed_where_proven() {
    let profile = rows::<CompanyProfile>(PROFILE).remove(0);
    assert_eq!(profile.symbol, Ticker::new("AAPL").unwrap());

    let rating = rows::<BulkStockRating>(RATING).remove(0);
    assert_eq!(rating.symbol.as_str(), "000001.SZ");
    let etf = rows::<BulkEtfHolding>(ETF).remove(0);
    assert_eq!(etf.asset.as_str(), "009150.KS");
}

#[test]
fn all_seven_contracts_require_bare_array_roots_and_accept_empty_arrays() {
    assert_array_contract::<CompanyProfile>();
    assert_array_contract::<BulkStockRating>();
    assert_array_contract::<BulkDcfValuation>();
    assert_array_contract::<BulkFinancialScore>();
    assert_array_contract::<BulkPriceTargetSummary>();
    assert_array_contract::<BulkEtfHolding>();
    assert_array_contract::<BulkUpgradesDowngradesConsensus>();
}

fn rows<T: DeserializeOwned>(fixture: &[u8]) -> Vec<T> {
    serde_json::from_slice(fixture).unwrap()
}

fn source_row(fixture: &[u8]) -> Map<String, Value> {
    serde_json::from_slice::<Value>(fixture).unwrap()[0]
        .as_object()
        .unwrap()
        .clone()
}

fn assert_exact<T>(fixture: &[u8], fields: usize)
where
    T: DeserializeOwned + Serialize,
{
    let source: Value = serde_json::from_slice(fixture).unwrap();
    assert_eq!(source[0].as_object().unwrap().len(), fields);
    assert_eq!(serde_json::to_value(rows::<T>(fixture)).unwrap(), source);
}

fn assert_required<T>(fixture: &[u8])
where
    T: DeserializeOwned,
{
    assert_required_nullable::<T>(fixture, &[]);
}

fn assert_required_nullable<T>(fixture: &[u8], nullable: &[&str])
where
    T: DeserializeOwned,
{
    let row = source_row(fixture);
    for field in row.keys() {
        let mut missing = row.clone();
        missing.remove(field);
        assert!(
            serde_json::from_value::<T>(Value::Object(missing)).is_err(),
            "missing {field} unexpectedly decoded"
        );

        let mut null = row.clone();
        null.insert(field.clone(), Value::Null);
        assert_eq!(
            serde_json::from_value::<T>(Value::Object(null)).is_ok(),
            nullable.contains(&field.as_str()),
            "null {field} decoded contrary to its nullability"
        );
    }

    let mut future = row;
    future.insert("futureField".into(), json!({"nested": true}));
    assert!(serde_json::from_value::<T>(Value::Object(future)).is_ok());
}

fn assert_number_rejected<T>(fixture: &[u8], field: &str)
where
    T: DeserializeOwned,
{
    let mut row = source_row(fixture);
    row.insert(field.into(), json!(123.5));
    assert!(serde_json::from_value::<T>(Value::Object(row)).is_err());
}

fn assert_array_contract<T>()
where
    T: DeserializeOwned,
{
    assert!(serde_json::from_slice::<Vec<T>>(b"[]").unwrap().is_empty());
    assert!(serde_json::from_slice::<Vec<T>>(b"{}").is_err());
}
