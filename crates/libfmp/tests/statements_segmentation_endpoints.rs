mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
        statements::{
            RevenueGeographicSegmentationQuery, RevenueProductSegmentationQuery,
            SegmentationStructure, revenue_geographic_segmentation, revenue_product_segmentation,
        },
    },
    query::RetrievalFrequency,
    transport::HttpMethod,
    types::Ticker,
};

use support::{FixtureExecutor, json_fixture};

const PRODUCT: &[u8] = include_bytes!("fixtures/revenue_product_segmentation.json");
const GEOGRAPHIC: &[u8] = include_bytes!("fixtures/revenue_geographic_segmentation.json");

#[test]
fn descriptors_use_exact_get_paths_bare_shapes_and_only_sourced_metadata() {
    let symbol = Ticker::new("AAPL").unwrap();
    let product_query = RevenueProductSegmentationQuery::new(symbol.clone())
        .with_period(RetrievalFrequency::Annual)
        .with_structure(SegmentationStructure::Flat);
    let geographic_query: RevenueGeographicSegmentationQuery = (&symbol).into();

    assert_eq!(product_query.symbol(), &symbol);
    assert_eq!(product_query.period(), Some(RetrievalFrequency::Annual));
    assert_eq!(product_query.structure(), Some(SegmentationStructure::Flat));
    assert_eq!(geographic_query.symbol(), &symbol);
    assert_eq!(geographic_query.period(), None);
    assert_eq!(geographic_query.structure(), None);
    assert_eq!(SegmentationStructure::Flat.as_str(), "flat");
    assert_eq!(SegmentationStructure::Flat.to_string(), "flat");

    assert_facts(
        &revenue_product_segmentation(product_query),
        "revenue-product-segmentation",
    );
    assert_facts(
        &revenue_geographic_segmentation(geographic_query),
        "revenue-geographic-segmentation",
    );
}

fn assert_facts<Q, R>(endpoint: &EndpointSpec<Q, R>, path: &'static str) {
    assert_eq!(endpoint.method(), HttpMethod::Get);
    assert_eq!(endpoint.id(), path);
    assert_eq!(endpoint.relative_path(), path);
    assert_eq!(
        endpoint.metadata().geography(),
        GeographicAvailability::Unspecified
    );
    assert_eq!(endpoint.metadata().access(), AccessRequirement::Unspecified);
    assert_eq!(
        endpoint.metadata().bounds(),
        EndpointBounds::new().with_response_rows(1_000)
    );
    assert_eq!(endpoint.metadata().conditional_plan(), None);
    assert_eq!(endpoint.metadata().realtime(), None);
}

#[tokio::test]
async fn proxy_preserves_omission_exact_order_both_frequencies_flat_auth_and_headers() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(PRODUCT),
        json_fixture(PRODUCT),
        json_fixture(GEOGRAPHIC),
    ]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("stable")
        .default_header("x-data-scope", "revenue-segments")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .executor(executor.clone())
        .build()
        .unwrap();
    let symbol = Ticker::new("BRK.B / Class A").unwrap();

    let omitted = client.revenue_product_segmentation(&symbol).await.unwrap();
    let product = client
        .revenue_product_segmentation(
            RevenueProductSegmentationQuery::new(symbol.clone())
                .with_period(RetrievalFrequency::Annual)
                .with_structure(SegmentationStructure::Flat),
        )
        .await
        .unwrap();
    let geographic = client
        .revenue_geographic_segmentation(
            RevenueGeographicSegmentationQuery::new(symbol)
                .with_period(RetrievalFrequency::Quarterly)
                .with_structure(SegmentationStructure::Flat),
        )
        .await
        .unwrap();

    assert_eq!(
        omitted[0].data["Mac"],
        serde_json::json!(33_708_000_000_u64)
    );
    assert_eq!(product[0].fiscal_year.get(), 2025);
    assert_eq!(
        geographic[0].data["Greater China Segment"],
        serde_json::json!(64_377_000_000_u64)
    );

    let requests = executor.requests();
    assert!(requests.iter().all(|request| {
        request.method() == HttpMethod::Get
            && request.expose_headers()["x-router-token"] == "Token proxy-secret"
            && request.expose_headers()["x-data-scope"] == "revenue-segments"
    }));
    assert_eq!(
        requests
            .iter()
            .map(|request| request.expose_url().as_str())
            .collect::<Vec<_>>(),
        [
            "https://proxy.example/router/stable/revenue-product-segmentation?symbol=BRK.B+%2F+Class+A",
            "https://proxy.example/router/stable/revenue-product-segmentation?symbol=BRK.B+%2F+Class+A&period=annual&structure=flat",
            "https://proxy.example/router/stable/revenue-geographic-segmentation?symbol=BRK.B+%2F+Class+A&period=quarter&structure=flat",
        ]
    );
}

#[tokio::test]
async fn direct_header_and_query_auth_use_the_same_typed_contracts() {
    for (authentication, expected_url, expected_header) in [
        (
            Authentication::fmp_header("header-secret"),
            "https://financialmodelingprep.com/stable/revenue-product-segmentation?symbol=AAPL",
            Some(("apikey", "header-secret")),
        ),
        (
            Authentication::fmp_query("query-secret"),
            "https://financialmodelingprep.com/stable/revenue-geographic-segmentation?symbol=AAPL&apikey=query-secret",
            None,
        ),
    ] {
        let response = if expected_header.is_some() {
            PRODUCT
        } else {
            GEOGRAPHIC
        };
        let executor = Arc::new(FixtureExecutor::new([json_fixture(response)]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();

        if expected_header.is_some() {
            client
                .revenue_product_segmentation(Ticker::new("AAPL").unwrap())
                .await
                .unwrap();
        } else {
            client
                .revenue_geographic_segmentation(Ticker::new("AAPL").unwrap())
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
