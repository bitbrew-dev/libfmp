mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        analyst::{
            FinancialEstimatesQuery, HistoricalRatingsQuery, RatingsSnapshotQuery,
            financial_estimates, historical_ratings, ratings_snapshot,
        },
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
    },
    error::ErrorCategory,
    query::RetrievalFrequency,
    responses::analyst::{FinancialEstimate, HistoricalRating, RatingSnapshot},
    transport::HttpMethod,
    types::{Limit, Page, Ticker},
};

use support::{FixtureExecutor, json_fixture};

const ESTIMATES: &[u8] = include_bytes!("fixtures/financial_estimates.json");
const SNAPSHOT: &[u8] = include_bytes!("fixtures/ratings_snapshot.json");
const HISTORICAL: &[u8] = include_bytes!("fixtures/historical_ratings.json");

#[test]
fn descriptors_use_exact_paths_bare_rows_and_only_documented_metadata() {
    let estimates = financial_estimates(FinancialEstimatesQuery::new(
        Ticker::new("AAPL").unwrap(),
        RetrievalFrequency::Annual,
    ));
    assert_facts(
        &estimates,
        "analyst-estimates",
        EndpointBounds::new().with_response_rows(1_000),
    );

    let snapshot = ratings_snapshot(RatingsSnapshotQuery::new(Ticker::new("AAPL").unwrap()));
    assert_facts(
        &snapshot,
        "ratings-snapshot",
        EndpointBounds::new().with_response_rows(1),
    );

    let historical = historical_ratings(HistoricalRatingsQuery::new(Ticker::new("AAPL").unwrap()));
    assert_facts(
        &historical,
        "ratings-historical",
        EndpointBounds::new().with_response_rows(10_000),
    );

    for bounds in [
        estimates.metadata().bounds(),
        snapshot.metadata().bounds(),
        historical.metadata().bounds(),
    ] {
        assert_eq!(bounds.limit(), None);
        assert_eq!(bounds.page(), None);
        assert!(bounds.accepts_page(Page(u32::MAX)));
        assert!(bounds.accepts_limit(Limit(u32::MAX)));
    }
    assert_response_types(&estimates, &snapshot, &historical);
}

fn assert_facts<Q, R>(
    endpoint: &EndpointSpec<Q, Vec<R>>,
    path: &'static str,
    bounds: EndpointBounds,
) {
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

fn assert_response_types(
    _: &EndpointSpec<FinancialEstimatesQuery, Vec<FinancialEstimate>>,
    _: &EndpointSpec<RatingsSnapshotQuery, Vec<RatingSnapshot>>,
    _: &EndpointSpec<HistoricalRatingsQuery, Vec<HistoricalRating>>,
) {
}

#[tokio::test]
async fn custom_proxy_preserves_exact_query_order_auth_headers_and_bare_arrays() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(ESTIMATES),
        json_fixture(SNAPSHOT),
        json_fixture(HISTORICAL),
    ]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("gateway/stable")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "analyst-ratings")
        .executor(executor.clone())
        .build()
        .unwrap();

    let estimates = client
        .financial_estimates(
            FinancialEstimatesQuery::new(
                Ticker::new("BRK.B / Class A").unwrap(),
                RetrievalFrequency::Quarterly,
            )
            .with_page(Page(0))
            .with_limit(Limit(u32::MAX)),
        )
        .await
        .unwrap();
    let snapshot = client
        .ratings_snapshot(RatingsSnapshotQuery::new(Ticker::new("AAPL").unwrap()))
        .await
        .unwrap();
    let historical = client
        .historical_ratings(
            HistoricalRatingsQuery::new(Ticker::new("AAPL").unwrap()).with_limit(Limit(0)),
        )
        .await
        .unwrap();

    assert_eq!(estimates.len(), 1);
    assert_eq!(estimates[0].revenue_high, 735_022_980_353);
    assert_eq!(snapshot.len(), 1);
    assert_eq!(snapshot[0].rating, "B");
    assert_eq!(historical.len(), 1);
    assert_eq!(historical[0].date.to_string(), "2026-07-30");

    let requests = executor.requests();
    assert!(
        requests
            .iter()
            .all(|request| request.method() == HttpMethod::Get)
    );
    assert!(requests.iter().all(|request| {
        request.expose_headers()["x-router-token"] == "Token proxy-secret"
            && request.expose_headers()["x-data-scope"] == "analyst-ratings"
    }));
    assert_eq!(
        requests
            .iter()
            .map(|request| request.expose_url().as_str())
            .collect::<Vec<_>>(),
        [
            "https://proxy.example/router/gateway/stable/analyst-estimates?symbol=BRK.B+%2F+Class+A&period=quarter&page=0&limit=4294967295",
            "https://proxy.example/router/gateway/stable/ratings-snapshot?symbol=AAPL",
            "https://proxy.example/router/gateway/stable/ratings-historical?symbol=AAPL&limit=0",
        ]
    );
}

#[tokio::test]
async fn direct_header_and_query_auth_preserve_omission_zero_and_full_domains() {
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
            json_fixture(ESTIMATES),
            json_fixture(ESTIMATES),
            json_fixture(SNAPSHOT),
            json_fixture(HISTORICAL),
            json_fixture(HISTORICAL),
        ]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();

        client
            .financial_estimates(FinancialEstimatesQuery::new(
                Ticker::new("AAPL").unwrap(),
                RetrievalFrequency::Annual,
            ))
            .await
            .unwrap();
        client
            .financial_estimates(
                FinancialEstimatesQuery::new(
                    Ticker::new("AAPL").unwrap(),
                    RetrievalFrequency::Quarterly,
                )
                .with_page(Page(0))
                .with_limit(Limit(u32::MAX)),
            )
            .await
            .unwrap();
        client
            .ratings_snapshot(RatingsSnapshotQuery::new(Ticker::new("AAPL").unwrap()))
            .await
            .unwrap();
        client
            .historical_ratings(HistoricalRatingsQuery::new(Ticker::new("AAPL").unwrap()))
            .await
            .unwrap();
        client
            .historical_ratings(
                HistoricalRatingsQuery::new(Ticker::new("AAPL").unwrap())
                    .with_limit(Limit(u32::MAX)),
            )
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
                    "https://financialmodelingprep.com/stable/analyst-estimates?symbol=AAPL&period=annual{suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/analyst-estimates?symbol=AAPL&period=quarter&page=0&limit=4294967295{suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/ratings-snapshot?symbol=AAPL{suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/ratings-historical?symbol=AAPL{suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/ratings-historical?symbol=AAPL&limit=4294967295{suffix}"
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
async fn malformed_non_array_responses_keep_each_endpoint_identity() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(b"{}"),
        json_fixture(b"{}"),
        json_fixture(b"{}"),
    ]));
    let client = Client::builder()
        .authentication(Authentication::fmp_header("secret"))
        .executor(executor)
        .build()
        .unwrap();

    let estimates_error = client
        .financial_estimates(FinancialEstimatesQuery::new(
            Ticker::new("AAPL").unwrap(),
            RetrievalFrequency::Annual,
        ))
        .await
        .unwrap_err();
    assert_eq!(estimates_error.category(), ErrorCategory::Decode);
    assert_eq!(estimates_error.endpoint(), Some("analyst-estimates"));
    assert_eq!(estimates_error.status_code(), Some(200));

    let snapshot_error = client
        .ratings_snapshot(RatingsSnapshotQuery::new(Ticker::new("AAPL").unwrap()))
        .await
        .unwrap_err();
    assert_eq!(snapshot_error.category(), ErrorCategory::Decode);
    assert_eq!(snapshot_error.endpoint(), Some("ratings-snapshot"));
    assert_eq!(snapshot_error.status_code(), Some(200));

    let historical_error = client
        .historical_ratings(HistoricalRatingsQuery::new(Ticker::new("AAPL").unwrap()))
        .await
        .unwrap_err();
    assert_eq!(historical_error.category(), ErrorCategory::Decode);
    assert_eq!(historical_error.endpoint(), Some("ratings-historical"));
    assert_eq!(historical_error.status_code(), Some(200));
}
