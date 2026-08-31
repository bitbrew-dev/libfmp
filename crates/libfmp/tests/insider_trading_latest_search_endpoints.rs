mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        insider_trading::{
            InsiderTradesSearchQuery, LatestInsiderTradesQuery, latest_insider_trades,
            search_insider_trades,
        },
        metadata::{AccessRequirement, GeographicAvailability},
    },
    error::ErrorCategory,
    responses::insider_trading::InsiderTrade,
    transport::HttpMethod,
    types::{Cik, Date, Limit, Page, Ticker, TransactionTypeCode},
};

use support::{FixtureExecutor, json_fixture};

const LATEST: &[u8] = include_bytes!("fixtures/latest_insider_trades.json");
const SEARCH: &[u8] = include_bytes!("fixtures/searched_insider_trades.json");

#[test]
fn descriptors_use_exact_paths_shared_bare_rows_and_only_documented_metadata() {
    let latest = latest_insider_trades(LatestInsiderTradesQuery::new());
    let search = search_insider_trades(InsiderTradesSearchQuery::new());

    assert_facts(&latest, "insider-trading/latest");
    assert_facts(&search, "insider-trading/search");
    assert_response_types(&latest, &search);
}

fn assert_facts<Q>(endpoint: &EndpointSpec<Q, Vec<InsiderTrade>>, path: &'static str) {
    assert_eq!(endpoint.method(), HttpMethod::Get);
    assert_eq!(endpoint.id(), path);
    assert_eq!(endpoint.relative_path(), path);
    assert_eq!(
        endpoint.metadata().geography(),
        GeographicAvailability::UsOnly
    );
    assert_eq!(endpoint.metadata().access(), AccessRequirement::Unspecified);
    let bounds = endpoint.metadata().bounds();
    assert_eq!(bounds.limit(), None);
    assert_eq!(bounds.response_rows().unwrap().maximum(), 1_000);
    assert_eq!(bounds.page().unwrap().maximum(), 100);
    assert_eq!(bounds.date_range_days(), None);
    assert_eq!(endpoint.metadata().conditional_plan(), None);
    assert_eq!(endpoint.metadata().realtime(), None);
}

fn assert_response_types(
    _: &EndpointSpec<LatestInsiderTradesQuery, Vec<InsiderTrade>>,
    _: &EndpointSpec<InsiderTradesSearchQuery, Vec<InsiderTrade>>,
) {
}

#[tokio::test]
async fn custom_proxy_preserves_exact_order_encoding_headers_and_fixture_identity() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(LATEST),
        json_fixture(SEARCH),
    ]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("gateway/stable")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "insider trades")
        .executor(executor.clone())
        .build()
        .unwrap();

    let latest = client
        .latest_insider_trades(
            LatestInsiderTradesQuery::new()
                .with_date(Date::parse("2026-01-27").unwrap())
                .with_page(Page(0))
                .with_limit(Limit(u32::MAX)),
        )
        .await
        .unwrap();
    let search = client
        .search_insider_trades(
            InsiderTradesSearchQuery::new()
                .with_symbol(Ticker::new("BRK.B / Class A").unwrap())
                .with_page(Page(u32::MAX))
                .with_limit(Limit(0))
                .with_reporting_cik(Cik::new("0001496686").unwrap())
                .with_company_cik(Cik::new("0000320193").unwrap())
                .with_transaction_type(TransactionTypeCode::new("S-Sale / future").unwrap()),
        )
        .await
        .unwrap();

    assert_eq!(latest.len(), 1);
    assert_eq!(search.len(), 1);
    assert_eq!(latest[0], search[0]);
    assert_eq!(latest[0].symbol.as_str(), "TRMK");
    assert_eq!(latest[0].reporting_cik.as_str(), "0001661867");
    assert_eq!(latest[0].company_cik.as_str(), "0000036146");
    assert_eq!(latest[0].transaction_type.as_str(), "A-Award");
    assert_eq!(latest[0].price, 0.0);

    let requests = executor.requests();
    assert!(requests.iter().all(|request| {
        request.method() == HttpMethod::Get
            && request.expose_headers()["x-router-token"] == "Token proxy-secret"
            && request.expose_headers()["x-data-scope"] == "insider trades"
    }));
    assert_eq!(
        requests
            .iter()
            .map(|request| request.expose_url().as_str())
            .collect::<Vec<_>>(),
        [
            format!(
                "https://proxy.example/router/gateway/stable/insider-trading/latest?date=2026-01-27&page=0&limit={}",
                u32::MAX
            ),
            format!(
                "https://proxy.example/router/gateway/stable/insider-trading/search?symbol=BRK.B+%2F+Class+A&page={}&limit=0&reportingCik=0001496686&companyCik=0000320193&transactionType=S-Sale+%2F+future",
                u32::MAX
            ),
        ]
    );
}

#[tokio::test]
async fn direct_auth_preserves_fully_empty_queries_and_adds_only_authentication() {
    for (authentication, suffix, expected_header) in [
        (
            Authentication::fmp_header("header-secret"),
            "",
            Some("header-secret"),
        ),
        (
            Authentication::fmp_query("query-secret"),
            "?apikey=query-secret",
            None,
        ),
    ] {
        let executor = Arc::new(FixtureExecutor::new([
            json_fixture(LATEST),
            json_fixture(SEARCH),
        ]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();

        client
            .latest_insider_trades(LatestInsiderTradesQuery::new())
            .await
            .unwrap();
        client
            .search_insider_trades(InsiderTradesSearchQuery::new())
            .await
            .unwrap();

        let requests = executor.requests();
        assert_eq!(
            requests
                .iter()
                .map(|request| request.expose_url().as_str())
                .collect::<Vec<_>>(),
            [
                format!("https://financialmodelingprep.com/stable/insider-trading/latest{suffix}"),
                format!("https://financialmodelingprep.com/stable/insider-trading/search{suffix}"),
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
async fn malformed_non_array_responses_keep_each_endpoint_identity() {
    let executor = Arc::new(FixtureExecutor::new([
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
            .latest_insider_trades(LatestInsiderTradesQuery::new())
            .await
            .unwrap_err(),
        client
            .search_insider_trades(InsiderTradesSearchQuery::new())
            .await
            .unwrap_err(),
    ];

    for (error, endpoint) in errors
        .iter()
        .zip(["insider-trading/latest", "insider-trading/search"])
    {
        assert_eq!(error.category(), ErrorCategory::Decode);
        assert_eq!(error.endpoint(), Some(endpoint));
        assert_eq!(error.status_code(), Some(200));
    }
}
