mod support;

use std::{str::FromStr, sync::Arc};

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
        news::{
            ArticlesQuery, LatestGeneralNewsQuery, LatestPressReleasesQuery, articles,
            latest_general_news, latest_press_releases,
        },
    },
    transport::HttpMethod,
    types::{Date, Limit, Page},
};

use support::{FixtureExecutor, json_fixture};

const FMP_ARTICLES: &[u8] = include_bytes!("fixtures/fmp_articles.json");
const GENERAL: &[u8] = include_bytes!("fixtures/latest_general_news.json");
const PRESS_RELEASES: &[u8] = include_bytes!("fixtures/latest_press_releases.json");

#[test]
fn descriptors_use_exact_paths_vec_rows_and_only_documented_metadata() {
    assert_facts(
        &articles(ArticlesQuery::new()),
        "fmp-articles",
        GeographicAvailability::UsOnly,
        EndpointBounds::new(),
    );
    let paged_bounds = EndpointBounds::new().with_response_rows(250).with_page(100);
    assert_facts(
        &latest_general_news(LatestGeneralNewsQuery::new()),
        "news/general-latest",
        GeographicAvailability::Worldwide,
        paged_bounds,
    );
    assert_facts(
        &latest_press_releases(LatestPressReleasesQuery::new()),
        "news/press-releases-latest",
        GeographicAvailability::UsOnly,
        paged_bounds,
    );

    assert_eq!(paged_bounds.limit(), None);
    assert!(paged_bounds.accepts_limit(Limit(251)));
    assert!(paged_bounds.accepts_page(Page(0)));
    assert!(paged_bounds.accepts_page(Page(100)));
    assert!(!paged_bounds.accepts_page(Page(101)));
    assert_eq!(paged_bounds.date_range_days(), None);
    assert!(EndpointBounds::new().accepts_page(Page(u32::MAX)));
    assert!(EndpointBounds::new().accepts_limit(Limit(u32::MAX)));
}

fn assert_facts<Q, R>(
    endpoint: &EndpointSpec<Q, R>,
    path: &'static str,
    geography: GeographicAvailability,
    bounds: EndpointBounds,
) {
    assert_eq!(endpoint.method(), HttpMethod::Get);
    assert_eq!(endpoint.id(), path);
    assert_eq!(endpoint.relative_path(), path);
    assert_eq!(endpoint.metadata().geography(), geography);
    assert_eq!(endpoint.metadata().access(), AccessRequirement::Unspecified);
    assert_eq!(endpoint.metadata().bounds(), bounds);
    assert_eq!(endpoint.metadata().conditional_plan(), None);
    assert_eq!(endpoint.metadata().realtime(), None);
}

#[tokio::test]
async fn custom_proxy_preserves_exact_query_order_auth_headers_and_bare_arrays() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(FMP_ARTICLES),
        json_fixture(GENERAL),
        json_fixture(PRESS_RELEASES),
    ]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("stable")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "authored-general-press")
        .executor(executor.clone())
        .build()
        .unwrap();
    let from = Date::from_str("2026-01-27").unwrap();
    let to = Date::from_str("2026-04-28").unwrap();

    let authored = client
        .articles(
            ArticlesQuery::new()
                .with_page(Page(0))
                .with_limit(Limit(20)),
        )
        .await
        .unwrap();
    let general = client
        .latest_general_news(
            LatestGeneralNewsQuery::new()
                .with_from(from)
                .with_to(to)
                .with_page(Page(100))
                .with_limit(Limit(251)),
        )
        .await
        .unwrap();
    let press = client
        .latest_press_releases(
            LatestPressReleasesQuery::new()
                .with_from(from)
                .with_to(to)
                .with_page(Page(0))
                .with_limit(Limit(251)),
        )
        .await
        .unwrap();

    assert_eq!(authored.len(), 1);
    assert!(authored[0].content.starts_with("<ul>\n"));
    assert_eq!(general.len(), 1);
    assert_eq!(general[0].symbol, None);
    assert_eq!(
        general[0].text,
        "Polaris Renewable Energy Inc. (PIF:CA) Q2 2026 Earnings Call Transcript"
    );
    assert_eq!(press.len(), 1);
    assert_eq!(press[0].symbol.as_ref().unwrap().as_str(), "RXT");

    let requests = executor.requests();
    assert!(
        requests
            .iter()
            .all(|request| request.method() == HttpMethod::Get)
    );
    assert!(requests.iter().all(|request| {
        request.expose_headers()["x-router-token"] == "Token proxy-secret"
            && request.expose_headers()["x-data-scope"] == "authored-general-press"
    }));
    assert_eq!(
        requests
            .iter()
            .map(|request| request.expose_url().as_str())
            .collect::<Vec<_>>(),
        [
            "https://proxy.example/router/stable/fmp-articles?page=0&limit=20",
            "https://proxy.example/router/stable/news/general-latest?from=2026-01-27&to=2026-04-28&page=100&limit=251",
            "https://proxy.example/router/stable/news/press-releases-latest?from=2026-01-27&to=2026-04-28&page=0&limit=251",
        ]
    );
}

#[tokio::test]
async fn direct_header_and_query_auth_preserve_omission_and_independent_dates() {
    for (authentication, query_suffix, expected_header) in [
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
            json_fixture(FMP_ARTICLES),
            json_fixture(GENERAL),
            json_fixture(PRESS_RELEASES),
        ]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();
        let from = Date::from_str("2026-01-27").unwrap();
        let to = Date::from_str("2026-04-28").unwrap();

        client
            .articles(ArticlesQuery::new().with_page(Page(0)))
            .await
            .unwrap();
        client
            .latest_general_news(LatestGeneralNewsQuery::new().with_from(from))
            .await
            .unwrap();
        client
            .latest_press_releases(LatestPressReleasesQuery::new().with_to(to))
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
                    "https://financialmodelingprep.com/stable/fmp-articles?page=0{query_suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/news/general-latest?from=2026-01-27{query_suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/news/press-releases-latest?to=2026-04-28{query_suffix}"
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
