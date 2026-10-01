#[path = "support/bulk_csv.rs"]
mod bulk_csv;
#[allow(dead_code)] // The CSV helpers use only the fixture executor.
mod support;

use libfmp::{
    endpoints::bulk::{
        BulkStatementQuery, bulk_cash_flow_statement_growth, bulk_cash_flow_statements, bulk_eod,
    },
    query::{FiscalPeriod, Year},
    types::Date,
};

use bulk_csv::{assert_empty_bodies, assert_members, assert_round_trip, decode, first_row};

const CASH_FLOW: &[u8] = include_bytes!("fixtures/bulk_cash_flow_statements.csv");
const GROWTH: &[u8] = include_bytes!("fixtures/bulk_cash_flow_statement_growth.csv");
const EOD: &[u8] = include_bytes!("fixtures/bulk_eod.csv");

fn query() -> BulkStatementQuery {
    BulkStatementQuery::new(Year(2024), FiscalPeriod::Q1)
}

fn day() -> Date {
    Date::parse("2025-06-02").unwrap()
}

#[test]
fn live_csv_fixtures_round_trip_every_cell_with_exact_field_counts() {
    assert_round_trip(&bulk_cash_flow_statements(query()), CASH_FLOW, 47);
    assert_round_trip(&bulk_cash_flow_statement_growth(query()), GROWTH, 42);
    assert_round_trip(&bulk_eod(day().into()), EOD, 8);
}

#[test]
fn every_column_is_required_and_no_member_accepts_an_empty_cell() {
    assert_members(&bulk_cash_flow_statements(query()), CASH_FLOW, &[]);
    assert_members(&bulk_cash_flow_statement_growth(query()), GROWTH, &[]);
    assert_members(&bulk_eod(day().into()), EOD, &[]);
}

#[test]
fn empty_and_header_only_bodies_decode_to_no_rows() {
    assert_empty_bodies(&bulk_cash_flow_statements(query()), CASH_FLOW);
    assert_empty_bodies(&bulk_cash_flow_statement_growth(query()), GROWTH);
    assert_empty_bodies(&bulk_eod(day().into()), EOD);
}

#[test]
fn identity_fields_and_prices_preserve_provider_representations() {
    let cash_flow = decode(&bulk_cash_flow_statements(query()), CASH_FLOW)
        .unwrap()
        .remove(0);
    assert_eq!(cash_flow.cik.as_str(), "0000000000");
    assert_eq!(cash_flow.accepted_date.to_string(), "2024-03-30 20:00:00");
    assert_eq!(cash_flow.period, FiscalPeriod::Q1);
    assert_eq!(cash_flow.net_income.as_str(), "14932000000");

    let eod = decode(&bulk_eod(day().into()), EOD).unwrap();
    assert_eq!(eod[1].symbol.as_str(), "HKDCNH");
    assert_eq!(eod[1].date, day());
    assert_eq!(eod[1].open.as_str(), "0.91858");
    assert_eq!(eod[1].volume.as_str(), "0");
}

#[test]
fn activities_typos_keep_exact_provider_wire_names() {
    let growth = first_row(&bulk_cash_flow_statement_growth(query()), GROWTH);
    for key in [
        "growthNetCashProvidedByOperatingActivites",
        "growthOtherInvestingActivites",
        "growthNetCashUsedForInvestingActivites",
        "growthOtherFinancingActivites",
        "growthNetCashUsedProvidedByFinancingActivities",
    ] {
        assert!(growth.contains_key(key), "missing exact wire key {key}");
    }
    assert!(!growth.contains_key("growthOtherInvestingActivities"));
}
