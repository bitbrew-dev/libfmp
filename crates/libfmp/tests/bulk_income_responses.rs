#[path = "support/bulk_csv.rs"]
mod bulk_csv;
#[allow(dead_code)] // The CSV helpers use only the fixture executor.
mod support;

use libfmp::{
    endpoints::bulk::{BulkStatementQuery, bulk_income_statement_growth, bulk_income_statements},
    error::DecodeErrorKind,
    query::{FiscalPeriod, Year},
};

use bulk_csv::{
    assert_cell_failure, assert_empty_bodies, assert_members, assert_round_trip, decode, first_row,
    with_cells,
};

const INCOME: &[u8] = include_bytes!("fixtures/bulk_income_statements.csv");
const GROWTH: &[u8] = include_bytes!("fixtures/bulk_income_statement_growth.csv");

fn query() -> BulkStatementQuery {
    BulkStatementQuery::new(Year(2024), FiscalPeriod::FullYear)
}

#[test]
fn live_csv_fixtures_round_trip_every_cell_with_exact_field_counts() {
    assert_round_trip(&bulk_income_statements(query()), INCOME, 39);
    assert_round_trip(&bulk_income_statement_growth(query()), GROWTH, 34);
}

#[test]
fn every_column_is_required_and_no_member_accepts_an_empty_cell() {
    assert_members(&bulk_income_statements(query()), INCOME, &[]);
    assert_members(&bulk_income_statement_growth(query()), GROWTH, &[]);
}

#[test]
fn empty_and_header_only_bodies_decode_to_no_rows() {
    assert_empty_bodies(&bulk_income_statements(query()), INCOME);
    assert_empty_bodies(&bulk_income_statement_growth(query()), GROWTH);
}

#[test]
fn identity_fields_are_narrow_and_preserve_provider_representations() {
    let income = decode(&bulk_income_statements(query()), INCOME)
        .unwrap()
        .remove(0);
    assert_eq!(income.symbol.as_str(), "000001.SZ");
    assert_eq!(income.reported_currency.as_str(), "CNY");
    assert_eq!(income.cik.as_str(), "0000000000");
    assert_eq!(income.date.to_string(), "2024-12-31");
    assert_eq!(income.accepted_date.to_string(), "2024-12-31 00:00:00");
    assert_eq!(income.fiscal_year.as_str(), "2024");
    assert_eq!(income.period, FiscalPeriod::FullYear);
    assert_eq!(income.revenue.as_str(), "251641000000");

    for (member, cell) in [
        ("date", "2024-12-31 00:00:00"),
        ("acceptedDate", "2024-12-31"),
    ] {
        assert_cell_failure(
            decode(
                &bulk_income_statements(query()),
                with_cells(INCOME, &[(member, cell)]),
            ),
            member,
            DecodeErrorKind::InvalidValue,
        );
    }
}

#[test]
fn acronym_hazards_keep_exact_provider_wire_names() {
    let growth = first_row(&bulk_income_statement_growth(query()), GROWTH);
    for key in [
        "growthEBITDA",
        "growthEPS",
        "growthEPSDiluted",
        "growthEBIT",
    ] {
        assert!(growth.contains_key(key), "missing exact wire key {key}");
    }
    for incorrect in [
        "growthEbitda",
        "growthEps",
        "growthEpsDiluted",
        "growthEbit",
    ] {
        assert!(!growth.contains_key(incorrect));
    }
}
