mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        funds::{
            FundDisclosureDatesQuery, FundDisclosureHolderSearchQuery, FundDisclosureQuery,
            LatestFundDisclosureHoldersQuery, fund_disclosure_dates, fund_disclosures,
            latest_fund_disclosure_holders, search_fund_disclosure_holders,
        },
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
    },
    error::ErrorCategory,
    query::{Quarter, Year},
    responses::funds::{
        FundDisclosure, FundDisclosureDate, FundDisclosureHolder, FundDisclosureSearchResult,
    },
    transport::HttpMethod,
    types::{ApiDateTime, Cik, SearchTerm, Ticker},
};

use support::{FixtureExecutor, json_fixture};

const LATEST_HOLDERS: &[u8] = include_bytes!("fixtures/latest_fund_disclosure_holders.json");
const DISCLOSURES: &[u8] = include_bytes!("fixtures/fund_disclosures.json");
const SEARCH: &[u8] = include_bytes!("fixtures/fund_disclosure_holder_search.json");
const DATES: &[u8] = include_bytes!("fixtures/fund_disclosure_dates.json");

#[test]
fn descriptors_use_exact_paths_bare_types_and_only_documented_metadata() {
    let latest = latest_fund_disclosure_holders(LatestFundDisclosureHoldersQuery::new(
        Ticker::new("AAPL").unwrap(),
    ));
    let disclosures = fund_disclosures(FundDisclosureQuery::new(
        Ticker::new("VWO").unwrap(),
        Year(2023),
        Quarter::Q4,
    ));
    let search = search_fund_disclosure_holders(FundDisclosureHolderSearchQuery::new(
        SearchTerm::new("Federated Hermes Government Income Securities, Inc.").unwrap(),
    ));
    let dates = fund_disclosure_dates(FundDisclosureDatesQuery::new(Ticker::new("VWO").unwrap()));

    assert_facts(&latest, "funds/disclosure-holders-latest");
    assert_facts(&disclosures, "funds/disclosure");
    assert_facts(&search, "funds/disclosure-holders-search");
    assert_facts(&dates, "funds/disclosure-dates");
    assert_response_types(&latest, &disclosures, &search, &dates);
}

fn assert_facts<Q, R>(endpoint: &EndpointSpec<Q, Vec<R>>, path: &'static str) {
    assert_eq!(endpoint.method(), HttpMethod::Get);
    assert_eq!(endpoint.id(), path);
    assert_eq!(endpoint.relative_path(), path);
    assert_eq!(
        endpoint.metadata().geography(),
        GeographicAvailability::UsOnly
    );
    assert_eq!(endpoint.metadata().access(), AccessRequirement::Unspecified);
    assert_eq!(endpoint.metadata().bounds(), EndpointBounds::new());
    assert_eq!(endpoint.metadata().conditional_plan(), None);
    assert_eq!(endpoint.metadata().realtime(), None);
}

fn assert_response_types(
    _: &EndpointSpec<LatestFundDisclosureHoldersQuery, Vec<FundDisclosureHolder>>,
    _: &EndpointSpec<FundDisclosureQuery, Vec<FundDisclosure>>,
    _: &EndpointSpec<FundDisclosureHolderSearchQuery, Vec<FundDisclosureSearchResult>>,
    _: &EndpointSpec<FundDisclosureDatesQuery, Vec<FundDisclosureDate>>,
) {
}

#[tokio::test]
async fn custom_proxy_preserves_exact_query_order_encoding_headers_and_fixture_identity() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(LATEST_HOLDERS),
        json_fixture(DISCLOSURES),
        json_fixture(SEARCH),
        json_fixture(DATES),
    ]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("gateway/stable")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "fund disclosures")
        .executor(executor.clone())
        .build()
        .unwrap();

    let latest = client
        .latest_fund_disclosure_holders(LatestFundDisclosureHoldersQuery::new(
            Ticker::new("BRK.B / Class A").unwrap(),
        ))
        .await
        .unwrap();
    let disclosures = client
        .fund_disclosures(
            FundDisclosureQuery::new(Ticker::new("VWO").unwrap(), Year(2023), Quarter::Q4)
                .with_cik(Cik::new("0000857489").unwrap()),
        )
        .await
        .unwrap();
    let search = client
        .search_fund_disclosure_holders(FundDisclosureHolderSearchQuery::new(
            SearchTerm::new("Federated Hermes Government Income Securities, Inc.").unwrap(),
        ))
        .await
        .unwrap();
    let dates = client
        .fund_disclosure_dates(
            FundDisclosureDatesQuery::new(Ticker::new("VWO").unwrap())
                .with_cik(Cik::new("0000036405").unwrap()),
        )
        .await
        .unwrap();

    assert_eq!(latest[0].cik.as_str(), "0000866256");
    assert_eq!(latest[0].security_cusip.as_str(), "037833100");
    assert_eq!(latest[0].change, -316_881.0);
    assert_eq!(
        disclosures[0].symbol.as_ref().unwrap().as_str(),
        "000089.SZ"
    );
    assert_eq!(disclosures[0].cusip.as_str(), "N/A");
    assert_eq!(disclosures[0].currency_code.as_str(), "CNY");
    assert_eq!(disclosures[0].fair_val_level.as_str(), "2");
    assert_eq!(
        disclosures[0].accepted_date,
        ApiDateTime::parse("2023-12-28 09:26:13").unwrap()
    );
    assert_eq!(search[0].cik.as_str(), "0000355691");
    assert_eq!(search[0].entity_org_type.as_ref().unwrap().as_str(), "30");
    assert_eq!(dates[0].year.get(), 2026);
    assert_eq!(dates[0].quarter.get(), 2);

    let requests = executor.requests();
    assert!(requests.iter().all(|request| {
        request.method() == HttpMethod::Get
            && request.expose_headers()["x-router-token"] == "Token proxy-secret"
            && request.expose_headers()["x-data-scope"] == "fund disclosures"
    }));
    assert_eq!(
        requests
            .iter()
            .map(|request| request.expose_url().as_str())
            .collect::<Vec<_>>(),
        [
            "https://proxy.example/router/gateway/stable/funds/disclosure-holders-latest?symbol=BRK.B+%2F+Class+A",
            "https://proxy.example/router/gateway/stable/funds/disclosure?symbol=VWO&year=2023&quarter=4&cik=0000857489",
            "https://proxy.example/router/gateway/stable/funds/disclosure-holders-search?name=Federated+Hermes+Government+Income+Securities%2C+Inc.",
            "https://proxy.example/router/gateway/stable/funds/disclosure-dates?symbol=VWO&cik=0000036405",
        ]
    );
}

#[tokio::test]
async fn optional_ciks_are_omitted_and_direct_auth_preserves_all_exact_urls() {
    for (authentication, suffix, expected_header) in [
        (
            Authentication::fmp_header("header-secret"),
            "",
            Some("header-secret"),
        ),
        (
            Authentication::fmp_query("query-secret"),
            "&apikey=query-secret",
            None,
        ),
    ] {
        let executor = Arc::new(FixtureExecutor::new([
            json_fixture(LATEST_HOLDERS),
            json_fixture(DISCLOSURES),
            json_fixture(SEARCH),
            json_fixture(DATES),
        ]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();

        client
            .latest_fund_disclosure_holders(LatestFundDisclosureHoldersQuery::new(
                Ticker::new("AAPL").unwrap(),
            ))
            .await
            .unwrap();
        client
            .fund_disclosures(FundDisclosureQuery::new(
                Ticker::new("VWO").unwrap(),
                Year(2023),
                Quarter::Q4,
            ))
            .await
            .unwrap();
        client
            .search_fund_disclosure_holders(FundDisclosureHolderSearchQuery::new(
                SearchTerm::new("Vanguard Total World").unwrap(),
            ))
            .await
            .unwrap();
        client
            .fund_disclosure_dates(FundDisclosureDatesQuery::new(Ticker::new("VWO").unwrap()))
            .await
            .unwrap();

        let requests = executor.requests();
        assert_eq!(
            requests
                .iter()
                .map(|request| request.expose_url().as_str())
                .collect::<Vec<_>>(),
            [
                format!(
                    "https://financialmodelingprep.com/stable/funds/disclosure-holders-latest?symbol=AAPL{suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/funds/disclosure?symbol=VWO&year=2023&quarter=4{suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/funds/disclosure-holders-search?name=Vanguard+Total+World{suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/funds/disclosure-dates?symbol=VWO{suffix}"
                ),
            ]
        );
        for request in requests.iter() {
            match expected_header {
                Some(value) => assert_eq!(request.expose_headers()["apikey"], value),
                None => assert!(!request.expose_headers().contains_key("apikey")),
            }
        }
    }
}

#[tokio::test]
async fn malformed_non_arrays_keep_each_endpoint_identity() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(b"{}"),
        json_fixture(b"{}"),
        json_fixture(b"{}"),
        json_fixture(b"{}"),
    ]));
    let client = Client::builder()
        .authentication(Authentication::fmp_header("secret"))
        .executor(executor)
        .build()
        .unwrap();

    let errors = [
        client
            .latest_fund_disclosure_holders(LatestFundDisclosureHoldersQuery::new(
                Ticker::new("AAPL").unwrap(),
            ))
            .await
            .unwrap_err(),
        client
            .fund_disclosures(FundDisclosureQuery::new(
                Ticker::new("VWO").unwrap(),
                Year(2023),
                Quarter::Q4,
            ))
            .await
            .unwrap_err(),
        client
            .search_fund_disclosure_holders(FundDisclosureHolderSearchQuery::new(
                SearchTerm::new("Vanguard").unwrap(),
            ))
            .await
            .unwrap_err(),
        client
            .fund_disclosure_dates(FundDisclosureDatesQuery::new(Ticker::new("VWO").unwrap()))
            .await
            .unwrap_err(),
    ];
    for (error, endpoint) in errors.iter().zip([
        "funds/disclosure-holders-latest",
        "funds/disclosure",
        "funds/disclosure-holders-search",
        "funds/disclosure-dates",
    ]) {
        assert_eq!(error.category(), ErrorCategory::Decode);
        assert_eq!(error.endpoint(), Some(endpoint));
        assert_eq!(error.status_code(), Some(200));
    }
}
