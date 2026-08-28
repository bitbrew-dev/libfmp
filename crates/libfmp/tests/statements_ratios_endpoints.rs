mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
        statements::{
            FinancialRatiosQuery, FinancialRatiosTtmQuery, financial_ratios, financial_ratios_ttm,
        },
    },
    query::{FiscalPeriod, RetrievalFrequency},
    transport::HttpMethod,
    types::{Limit, Ticker},
};

use support::{FixtureExecutor, json_fixture};

const RATIOS: &[u8] = include_bytes!("fixtures/financial_ratios.json");
const RATIOS_TTM: &[u8] = include_bytes!("fixtures/financial_ratios_ttm.json");

#[test]
fn descriptors_use_exact_paths_bare_vec_contracts_and_distinct_metadata() {
    let symbol = Ticker::new("AAPL").unwrap();
    let regular_query = FinancialRatiosQuery::new(symbol.clone())
        .with_limit(Limit(1_000))
        .with_period(RetrievalFrequency::Quarterly);
    let ttm_query = FinancialRatiosTtmQuery::new(symbol.clone());

    assert_eq!(regular_query.symbol(), &symbol);
    assert_eq!(regular_query.limit(), Some(Limit(1_000)));
    assert_eq!(
        regular_query.period(),
        Some(RetrievalFrequency::Quarterly.into())
    );
    assert_eq!(ttm_query.symbol(), &symbol);

    assert_facts(
        &financial_ratios(regular_query),
        "ratios",
        EndpointBounds::new().with_response_rows(1_000),
    );
    assert_facts(
        &financial_ratios_ttm(ttm_query),
        "ratios-ttm",
        EndpointBounds::new(),
    );
}

fn assert_facts<Q, R>(endpoint: &EndpointSpec<Q, R>, path: &'static str, bounds: EndpointBounds) {
    assert_eq!(endpoint.method(), HttpMethod::Get);
    assert_eq!(endpoint.id(), path);
    assert_eq!(endpoint.relative_path(), path);
    assert_eq!(
        endpoint.metadata().geography(),
        GeographicAvailability::Worldwide
    );
    assert_eq!(endpoint.metadata().access(), AccessRequirement::Unspecified);
    assert_eq!(endpoint.metadata().bounds(), bounds);
    assert_eq!(endpoint.metadata().conditional_plan(), None);
    assert_eq!(endpoint.metadata().realtime(), None);
}

#[tokio::test]
async fn custom_proxy_preserves_exact_query_order_encoding_headers_and_shapes() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(RATIOS),
        json_fixture(RATIOS),
        json_fixture(RATIOS),
        json_fixture(RATIOS_TTM),
    ]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("stable")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-route", "fmp")
        .executor(executor.clone())
        .build()
        .unwrap();
    let symbol = Ticker::new("BRK.B / Class A").unwrap();

    let omitted = client
        .financial_ratios(FinancialRatiosQuery::new(symbol.clone()))
        .await
        .unwrap();
    let annual = client
        .financial_ratios(
            FinancialRatiosQuery::new(symbol.clone())
                .with_limit(Limit(5))
                .with_period(RetrievalFrequency::Annual),
        )
        .await
        .unwrap();
    let quarterly = client
        .financial_ratios(
            FinancialRatiosQuery::new(symbol.clone())
                .with_limit(Limit(7))
                .with_period(FiscalPeriod::Q4),
        )
        .await
        .unwrap();
    let ttm = client.financial_ratios_ttm(symbol).await.unwrap();

    assert_eq!(omitted[0].dividend_yield_percentage, 0.4038238951672435);
    assert_eq!(annual[0].period, FiscalPeriod::FullYear);
    assert_eq!(quarterly[0].interest_coverage_ratio, 0.0);
    assert_eq!(ttm[0].enterprise_value_ttm, 4_922_455_686_740);

    let requests = executor.requests();
    assert!(
        requests
            .iter()
            .all(|request| request.method() == HttpMethod::Get)
    );
    assert!(requests.iter().all(|request| {
        request.expose_headers()["x-router-token"] == "Token proxy-secret"
            && request.expose_headers()["x-route"] == "fmp"
    }));
    let urls = requests
        .iter()
        .map(|request| request.expose_url().as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        urls,
        [
            "https://proxy.example/router/stable/ratios?symbol=BRK.B+%2F+Class+A",
            "https://proxy.example/router/stable/ratios?symbol=BRK.B+%2F+Class+A&limit=5&period=annual",
            "https://proxy.example/router/stable/ratios?symbol=BRK.B+%2F+Class+A&limit=7&period=Q4",
            "https://proxy.example/router/stable/ratios-ttm?symbol=BRK.B+%2F+Class+A",
        ]
    );
}

#[tokio::test]
async fn direct_header_and_query_auth_use_the_same_typed_contracts() {
    for (authentication, expected_url, expected_header) in [
        (
            Authentication::fmp_header("header-secret"),
            "https://financialmodelingprep.com/stable/ratios?symbol=AAPL",
            Some(("apikey", "header-secret")),
        ),
        (
            Authentication::fmp_query("query-secret"),
            "https://financialmodelingprep.com/stable/ratios-ttm?symbol=AAPL&apikey=query-secret",
            None,
        ),
    ] {
        let response = if expected_header.is_some() {
            RATIOS
        } else {
            RATIOS_TTM
        };
        let executor = Arc::new(FixtureExecutor::new([json_fixture(response)]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();

        if expected_header.is_some() {
            client
                .financial_ratios(Ticker::new("AAPL").unwrap())
                .await
                .unwrap();
        } else {
            client
                .financial_ratios_ttm(Ticker::new("AAPL").unwrap())
                .await
                .unwrap();
        }

        let requests = executor.requests();
        assert_eq!(requests[0].expose_url().as_str(), expected_url);
        match expected_header {
            Some((name, value)) => assert_eq!(requests[0].expose_headers()[name], value),
            None => assert!(!requests[0].expose_headers().contains_key("apikey")),
        }
    }
}
