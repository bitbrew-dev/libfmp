mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        institutional_ownership::{
            InstitutionalIndustrySummaryQuery, InstitutionalPositionsSummaryQuery,
            institutional_industry_summary, institutional_positions_summary,
        },
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
    },
    query::{Quarter, Year},
    responses::institutional_ownership::{
        InstitutionalIndustrySummary, InstitutionalPositionSummary,
    },
    transport::HttpMethod,
    types::Ticker,
};

use support::{FixtureExecutor, json_fixture};

const POSITIONS: &[u8] = include_bytes!("fixtures/institutional_positions_summary.json");
const INDUSTRY: &[u8] = include_bytes!("fixtures/institutional_industry_summary.json");

#[test]
fn descriptors_use_exact_paths_typed_vec_rows_and_only_us_metadata() {
    let positions: EndpointSpec<
        InstitutionalPositionsSummaryQuery,
        Vec<InstitutionalPositionSummary>,
    > = institutional_positions_summary(InstitutionalPositionsSummaryQuery::new(
        Ticker::new("AAPL").unwrap(),
        Year(2023),
        Quarter::Q3,
    ));
    let industry: EndpointSpec<
        InstitutionalIndustrySummaryQuery,
        Vec<InstitutionalIndustrySummary>,
    > = institutional_industry_summary(InstitutionalIndustrySummaryQuery::new(
        Year(2023),
        Quarter::Q3,
    ));

    assert_facts(
        &positions,
        "institutional-ownership/symbol-positions-summary",
    );
    assert_facts(&industry, "institutional-ownership/industry-summary");
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

#[tokio::test]
async fn proxy_preserves_exact_query_order_complex_symbol_auth_headers_and_source_arrays() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(POSITIONS),
        json_fixture(INDUSTRY),
    ]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("/gateway/stable/")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "institutional-summaries")
        .executor(executor.clone())
        .build()
        .unwrap();

    let positions = client
        .institutional_positions_summary(InstitutionalPositionsSummaryQuery::new(
            Ticker::new("BRK.B / Class A").unwrap(),
            Year(2023),
            Quarter::Q3,
        ))
        .await
        .unwrap();
    let industry = client
        .institutional_industry_summary(InstitutionalIndustrySummaryQuery::new(
            Year(2023),
            Quarter::Q3,
        ))
        .await
        .unwrap();

    assert_eq!(positions.len(), 1);
    assert_eq!(positions[0].symbol.as_str(), "AAPL");
    assert_eq!(positions[0].cik.as_str(), "0000320193");
    assert_eq!(positions[0].total_invested_change, -245_052_087_186.0);
    assert_eq!(industry.len(), 1);
    assert_eq!(
        industry[0].industry_title,
        "ABRASIVE, ASBESTOS & MISC NONMETALLIC MINERAL PRODS"
    );

    let requests = executor.requests();
    assert!(requests.iter().all(|request| {
        request.method() == HttpMethod::Get
            && request.expose_headers()["x-router-token"] == "Token proxy-secret"
            && request.expose_headers()["x-data-scope"] == "institutional-summaries"
    }));
    assert_eq!(
        requests
            .iter()
            .map(|request| request.expose_url().as_str())
            .collect::<Vec<_>>(),
        [
            "https://proxy.example/router/gateway/stable/institutional-ownership/symbol-positions-summary?symbol=BRK.B+%2F+Class+A&year=2023&quarter=3",
            "https://proxy.example/router/gateway/stable/institutional-ownership/industry-summary?year=2023&quarter=3",
        ]
    );
}

#[tokio::test]
async fn direct_header_and_query_auth_apply_to_both_summary_contracts() {
    for (authentication, query_auth, expected_header) in [
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
            json_fixture(POSITIONS),
            json_fixture(INDUSTRY),
        ]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();

        client
            .institutional_positions_summary(InstitutionalPositionsSummaryQuery::new(
                Ticker::new("AAPL").unwrap(),
                Year(2023),
                Quarter::Q3,
            ))
            .await
            .unwrap();
        client
            .institutional_industry_summary(InstitutionalIndustrySummaryQuery::new(
                Year(2023),
                Quarter::Q3,
            ))
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
                    "https://financialmodelingprep.com/stable/institutional-ownership/symbol-positions-summary?symbol=AAPL&year=2023&quarter=3{query_auth}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/institutional-ownership/industry-summary?year=2023&quarter=3{query_auth}"
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
