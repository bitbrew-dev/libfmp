#[path = "support/bulk_csv.rs"]
mod bulk_csv;
#[allow(dead_code)] // The CSV helpers use only the fixture executor.
mod support;

use libfmp::{
    endpoints::bulk::{
        BulkPartQuery, bulk_company_profiles, bulk_dcf_valuations, bulk_etf_holdings,
        bulk_financial_scores, bulk_price_target_summaries, bulk_stock_ratings,
        bulk_upgrades_downgrades_consensus,
    },
    types::{BulkPart, Date, Ticker},
};

use bulk_csv::{assert_empty_bodies, assert_members, assert_round_trip, decode};

const PROFILE: &[u8] = include_bytes!("fixtures/bulk_company_profiles.csv");
const RATING: &[u8] = include_bytes!("fixtures/bulk_stock_ratings.csv");
const DCF: &[u8] = include_bytes!("fixtures/bulk_dcf_valuations.csv");
const SCORES: &[u8] = include_bytes!("fixtures/bulk_financial_scores.csv");
const TARGET: &[u8] = include_bytes!("fixtures/bulk_price_target_summaries.csv");
const ETF: &[u8] = include_bytes!("fixtures/bulk_etf_holdings.csv");
const CONSENSUS: &[u8] = include_bytes!("fixtures/bulk_upgrades_downgrades_consensus.csv");

fn part() -> BulkPartQuery {
    BulkPart::new("0").unwrap().into()
}

#[test]
fn live_csv_fixtures_round_trip_every_cell_with_exact_field_counts() {
    assert_round_trip(&bulk_company_profiles(part()), PROFILE, 36);
    assert_round_trip(&bulk_stock_ratings(), RATING, 9);
    assert_round_trip(&bulk_dcf_valuations(), DCF, 4);
    assert_round_trip(&bulk_financial_scores(), SCORES, 11);
    assert_round_trip(&bulk_price_target_summaries(), TARGET, 10);
    assert_round_trip(&bulk_etf_holdings(part()), ETF, 9);
    assert_round_trip(&bulk_upgrades_downgrades_consensus(), CONSENSUS, 7);
}

#[test]
fn every_column_is_required_and_only_optional_or_text_members_accept_an_empty_cell() {
    assert_members(
        &bulk_company_profiles(part()),
        PROFILE,
        &[
            "range",
            "companyName",
            "cik",
            "cusip",
            "exchangeFullName",
            "website",
            "description",
            "ceo",
            "fullTimeEmployees",
            "phone",
            "address",
            "city",
            "state",
            "zip",
            "image",
            "ipoDate",
        ],
    );
    assert_members(&bulk_stock_ratings(), RATING, &["rating"]);
    assert_members(&bulk_dcf_valuations(), DCF, &[]);
    assert_members(&bulk_financial_scores(), SCORES, &[]);
    assert_members(&bulk_price_target_summaries(), TARGET, &["publishers"]);
    assert_members(&bulk_etf_holdings(part()), ETF, &["name", "cusip"]);
    assert_members(
        &bulk_upgrades_downgrades_consensus(),
        CONSENSUS,
        &["symbol", "consensus"],
    );
}

#[test]
fn empty_and_header_only_bodies_decode_to_no_rows() {
    assert_empty_bodies(&bulk_company_profiles(part()), PROFILE);
    assert_empty_bodies(&bulk_stock_ratings(), RATING);
    assert_empty_bodies(&bulk_dcf_valuations(), DCF);
    assert_empty_bodies(&bulk_financial_scores(), SCORES);
    assert_empty_bodies(&bulk_price_target_summaries(), TARGET);
    assert_empty_bodies(&bulk_etf_holdings(part()), ETF);
    assert_empty_bodies(&bulk_upgrades_downgrades_consensus(), CONSENSUS);
}

#[test]
fn provider_spellings_and_quoted_text_are_preserved_verbatim() {
    let dcf = decode(&bulk_dcf_valuations(), DCF).unwrap().remove(0);
    assert_eq!(dcf.stock_price.as_str(), "7.62");
    assert_eq!(dcf.dcf.as_str(), "2.525226853334803");

    let target = decode(&bulk_price_target_summaries(), TARGET)
        .unwrap()
        .remove(0);
    assert_eq!(
        target.publishers,
        r#"["StreetInsider","Benzinga","Pulse 2.0"]"#
    );

    let etf = decode(&bulk_etf_holdings(part()), ETF).unwrap().remove(0);
    assert_eq!(etf.symbol.as_str(), " -- ");
    assert_eq!(etf.asset, Ticker::new("TDW").unwrap());
    assert_eq!(etf.last_updated, Date::parse("2026-09-27").unwrap());

    let profile = decode(&bulk_company_profiles(part()), PROFILE)
        .unwrap()
        .remove(0);
    assert_eq!(profile.symbol, Ticker::new("WMB").unwrap());
    assert_eq!(profile.company_name, "The Williams Companies, Inc.");
    assert!(!profile.is_etf);
}
