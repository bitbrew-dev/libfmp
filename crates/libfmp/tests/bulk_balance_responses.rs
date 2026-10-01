#[path = "support/bulk_csv.rs"]
mod bulk_csv;
#[allow(dead_code)] // The CSV helpers use only the fixture executor.
mod support;

use libfmp::{
    endpoints::bulk::{
        BulkStatementQuery, bulk_balance_sheet_statement_growth, bulk_balance_sheet_statements,
    },
    query::{FiscalPeriod, Year},
};

use bulk_csv::{assert_empty_bodies, assert_members, assert_round_trip, decode, first_row};

const BALANCE: &[u8] = include_bytes!("fixtures/bulk_balance_sheet_statements.csv");
const GROWTH: &[u8] = include_bytes!("fixtures/bulk_balance_sheet_statement_growth.csv");

fn query() -> BulkStatementQuery {
    BulkStatementQuery::new(Year(2024), FiscalPeriod::FullYear)
}

#[test]
fn live_csv_fixtures_round_trip_every_cell_with_exact_field_counts() {
    assert_round_trip(&bulk_balance_sheet_statements(query()), BALANCE, 61);
    assert_round_trip(&bulk_balance_sheet_statement_growth(query()), GROWTH, 56);
}

#[test]
fn every_column_is_required_and_no_member_accepts_an_empty_cell() {
    assert_members(&bulk_balance_sheet_statements(query()), BALANCE, &[]);
    assert_members(&bulk_balance_sheet_statement_growth(query()), GROWTH, &[]);
}

#[test]
fn empty_and_header_only_bodies_decode_to_no_rows() {
    assert_empty_bodies(&bulk_balance_sheet_statements(query()), BALANCE);
    assert_empty_bodies(&bulk_balance_sheet_statement_growth(query()), GROWTH);
}

#[test]
fn identity_fields_are_narrow_and_preserve_provider_representations() {
    let balance = decode(&bulk_balance_sheet_statements(query()), BALANCE).unwrap();
    assert_eq!(balance[1].symbol.as_str(), "0002.KL");
    assert_eq!(balance[1].reported_currency.as_str(), "MYR");
    assert_eq!(balance[1].cik.as_str(), "0000000000");
    assert_eq!(balance[1].accepted_date.to_string(), "2025-06-30 00:00:00");
    assert_eq!(balance[1].fiscal_year.as_str(), "2024");
    assert_eq!(balance[0].total_assets.as_str(), "5769270000000");
}

#[test]
fn provider_typo_and_liabilities_equity_wording_keep_exact_wire_names() {
    let growth = first_row(&bulk_balance_sheet_statement_growth(query()), GROWTH);
    for key in [
        "growthOthertotalStockholdersEquity",
        "growthTotalLiabilitiesAndStockholdersEquity",
    ] {
        assert!(growth.contains_key(key), "missing exact wire key {key}");
    }
    for incorrect in [
        "growthOtherTotalStockholdersEquity",
        "growthTotalLiabilitiesAndTotalEquity",
    ] {
        assert!(!growth.contains_key(incorrect));
    }
}
