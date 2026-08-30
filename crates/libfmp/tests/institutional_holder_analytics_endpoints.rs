mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        institutional_ownership::{
            InstitutionalHolderAnalyticsQuery, institutional_holder_analytics,
        },
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
    },
    query::{Quarter, Year},
    responses::institutional_ownership::InstitutionalHolderAnalytics,
    transport::HttpMethod,
    types::{Limit, Page, Ticker},
};

use support::{FixtureExecutor, json_fixture};

const ANALYTICS: &[u8] = include_bytes!("fixtures/institutional_holder_analytics.json");

#[test]
fn descriptor_uses_exact_path_query_row_and_us_only_metadata_without_bounds() {
    let symbol = Ticker::new("AAPL").unwrap();
    let query = InstitutionalHolderAnalyticsQuery::new(symbol.clone(), Year(2023), Quarter::Q3);
    assert_eq!(query.symbol(), &symbol);
    assert_eq!(query.year(), Year(2023));
    assert_eq!(query.quarter(), Quarter::Q3);
    assert_eq!(query.page(), None);
    assert_eq!(query.limit(), None);
    let zero = query.clone().with_page(Page(0)).with_limit(Limit(0));
    assert_eq!(zero.page(), Some(Page(0)));
    assert_eq!(zero.limit(), Some(Limit(0)));

    let endpoint = institutional_holder_analytics(query);
    assert_eq!(endpoint.method(), HttpMethod::Get);
    assert_eq!(
        endpoint.id(),
        "institutional-ownership/extract-analytics/holder"
    );
    assert_eq!(endpoint.relative_path(), endpoint.id());
    assert_eq!(
        endpoint.metadata().geography(),
        GeographicAvailability::UsOnly
    );
    assert_eq!(endpoint.metadata().access(), AccessRequirement::Unspecified);
    assert_eq!(endpoint.metadata().bounds(), EndpointBounds::new());
    assert_eq!(endpoint.metadata().conditional_plan(), None);
    assert_eq!(endpoint.metadata().realtime(), None);
    assert_response_type(&endpoint);
}

fn assert_response_type(
    _: &EndpointSpec<InstitutionalHolderAnalyticsQuery, Vec<InstitutionalHolderAnalytics>>,
) {
}

#[tokio::test]
async fn custom_proxy_preserves_exact_query_order_auth_headers_and_bare_array() {
    let executor = Arc::new(FixtureExecutor::new([json_fixture(ANALYTICS)]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("gateway/stable")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "holder-analytics")
        .executor(executor.clone())
        .build()
        .unwrap();

    let rows = client
        .institutional_holder_analytics(
            InstitutionalHolderAnalyticsQuery::new(
                Ticker::new("AAPL").unwrap(),
                Year(2023),
                Quarter::Q3,
            )
            .with_page(Page(0))
            .with_limit(Limit(10)),
        )
        .await
        .unwrap();

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].cik.as_str(), "0000102909");
    assert_eq!(rows[0].security_cusip.as_str(), "037833100");
    assert_eq!(rows[0].change_in_performance, -67_750_129_670);

    let requests = executor.requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].method(), HttpMethod::Get);
    assert_eq!(
        requests[0].expose_url().as_str(),
        "https://proxy.example/router/gateway/stable/institutional-ownership/extract-analytics/holder?symbol=AAPL&year=2023&quarter=3&page=0&limit=10"
    );
    assert_eq!(
        requests[0].expose_headers()["x-router-token"],
        "Token proxy-secret"
    );
    assert_eq!(
        requests[0].expose_headers()["x-data-scope"],
        "holder-analytics"
    );
}

#[tokio::test]
async fn direct_header_and_query_auth_preserve_omitted_zero_and_high_pagination() {
    for (authentication, later_auth, expected_header) in [
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
            json_fixture(ANALYTICS),
            json_fixture(ANALYTICS),
            json_fixture(ANALYTICS),
        ]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();
        let base = InstitutionalHolderAnalyticsQuery::new(
            Ticker::new("AAPL").unwrap(),
            Year(2023),
            Quarter::Q3,
        );

        client
            .institutional_holder_analytics(base.clone())
            .await
            .unwrap();
        client
            .institutional_holder_analytics(base.clone().with_page(Page(0)).with_limit(Limit(0)))
            .await
            .unwrap();
        let high = base.with_page(Page(u32::MAX)).with_limit(Limit(u32::MAX));
        assert_eq!(high.page(), Some(Page(u32::MAX)));
        assert_eq!(high.limit(), Some(Limit(u32::MAX)));
        client.institutional_holder_analytics(high).await.unwrap();

        let requests = executor.requests();
        assert_eq!(
            requests
                .iter()
                .map(|request| request.expose_url().as_str())
                .collect::<Vec<_>>(),
            [
                format!(
                    "https://financialmodelingprep.com/stable/institutional-ownership/extract-analytics/holder?symbol=AAPL&year=2023&quarter=3{later_auth}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/institutional-ownership/extract-analytics/holder?symbol=AAPL&year=2023&quarter=3&page=0&limit=0{later_auth}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/institutional-ownership/extract-analytics/holder?symbol=AAPL&year=2023&quarter=3&page=4294967295&limit=4294967295{later_auth}"
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
