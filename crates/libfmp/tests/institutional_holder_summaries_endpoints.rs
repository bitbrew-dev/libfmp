mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        institutional_ownership::{
            HolderIndustryBreakdownQuery, HolderPerformanceSummaryQuery, holder_industry_breakdown,
            holder_performance_summary,
        },
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
    },
    query::{Quarter, Year},
    responses::institutional_ownership::{HolderIndustryBreakdown, HolderPerformanceSummary},
    transport::HttpMethod,
    types::{Cik, Page},
};

use support::{FixtureExecutor, json_fixture};

const PERFORMANCE: &[u8] = include_bytes!("fixtures/holder_performance_summary.json");
const INDUSTRY: &[u8] = include_bytes!("fixtures/holder_industry_breakdown.json");

#[test]
fn descriptors_use_exact_paths_typed_rows_and_only_us_metadata() {
    let cik = Cik::new("0001067983").unwrap();
    let performance: EndpointSpec<HolderPerformanceSummaryQuery, Vec<HolderPerformanceSummary>> =
        holder_performance_summary(HolderPerformanceSummaryQuery::new(cik.clone()));
    let industry: EndpointSpec<HolderIndustryBreakdownQuery, Vec<HolderIndustryBreakdown>> =
        holder_industry_breakdown(HolderIndustryBreakdownQuery::new(
            cik,
            Year(2023),
            Quarter::Q3,
        ));
    assert_facts(
        &performance,
        "institutional-ownership/holder-performance-summary",
    );
    assert_facts(
        &industry,
        "institutional-ownership/holder-industry-breakdown",
    );

    assert_eq!(performance.metadata().bounds(), EndpointBounds::new());
    assert!(performance.metadata().bounds().accepts_page(Page(u32::MAX)));
}

fn assert_facts<Q, R>(endpoint: &EndpointSpec<Q, R>, path: &'static str) {
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
async fn custom_proxy_preserves_exact_queries_auth_headers_and_source_arrays() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(PERFORMANCE),
        json_fixture(INDUSTRY),
    ]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("gateway/stable")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "holder-summaries")
        .executor(executor.clone())
        .build()
        .unwrap();
    let cik = Cik::new("0001067983").unwrap();

    let performance = client
        .holder_performance_summary(
            HolderPerformanceSummaryQuery::new(cik.clone()).with_page(Page(0)),
        )
        .await
        .unwrap();
    let industry = client
        .holder_industry_breakdown(HolderIndustryBreakdownQuery::new(
            cik,
            Year(2023),
            Quarter::Q3,
        ))
        .await
        .unwrap();

    assert_eq!(performance.len(), 1);
    assert_eq!(performance[0].cik.as_str(), "0001067983");
    assert_eq!(performance[0].change_in_performance, -14_398_745_159.0);
    assert_eq!(industry.len(), 1);
    assert_eq!(
        industry[0].industry_title.as_deref(),
        Some("ELECTRONIC COMPUTERS")
    );
    assert_eq!(industry[0].change_in_performance, -47_453_494_598.0);

    let requests = executor.requests();
    assert!(
        requests
            .iter()
            .all(|request| request.method() == HttpMethod::Get)
    );
    assert!(requests.iter().all(|request| {
        request.expose_headers()["x-router-token"] == "Token proxy-secret"
            && request.expose_headers()["x-data-scope"] == "holder-summaries"
    }));
    assert_eq!(
        requests
            .iter()
            .map(|request| request.expose_url().as_str())
            .collect::<Vec<_>>(),
        [
            "https://proxy.example/router/gateway/stable/institutional-ownership/holder-performance-summary?cik=0001067983&page=0",
            "https://proxy.example/router/gateway/stable/institutional-ownership/holder-industry-breakdown?cik=0001067983&year=2023&quarter=3",
        ]
    );
}

#[tokio::test]
async fn direct_header_and_query_auth_preserve_page_omission_zero_and_high_values() {
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
            json_fixture(PERFORMANCE),
            json_fixture(PERFORMANCE),
            json_fixture(PERFORMANCE),
            json_fixture(INDUSTRY),
        ]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();
        let cik = Cik::new("0001067983").unwrap();

        client
            .holder_performance_summary(cik.clone())
            .await
            .unwrap();
        client
            .holder_performance_summary(
                HolderPerformanceSummaryQuery::new(cik.clone()).with_page(Page(0)),
            )
            .await
            .unwrap();
        client
            .holder_performance_summary(
                HolderPerformanceSummaryQuery::new(cik.clone()).with_page(Page(u32::MAX)),
            )
            .await
            .unwrap();
        client
            .holder_industry_breakdown(HolderIndustryBreakdownQuery::new(
                cik,
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
                    "https://financialmodelingprep.com/stable/institutional-ownership/holder-performance-summary?cik=0001067983{later_auth}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/institutional-ownership/holder-performance-summary?cik=0001067983&page=0{later_auth}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/institutional-ownership/holder-performance-summary?cik=0001067983&page=4294967295{later_auth}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/institutional-ownership/holder-industry-breakdown?cik=0001067983&year=2023&quarter=3{later_auth}"
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
