mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        commitment_of_traders::{CotQuery, cot_analysis, cot_report, cot_report_list},
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
    },
    error::ErrorCategory,
    transport::HttpMethod,
    types::{Date, Ticker},
};

use support::{FixtureExecutor, json_fixture};

const REPORT: &[u8] = include_bytes!("fixtures/cot_report.json");
const ANALYSIS: &[u8] = include_bytes!("fixtures/cot_analysis.json");
const REPORT_LIST: &[u8] = include_bytes!("fixtures/cot_report_list.json");

#[test]
fn descriptors_have_exact_get_identity_paths_unit_list_and_only_documented_metadata() {
    let report = cot_report(CotQuery::new());
    let analysis = cot_analysis(
        CotQuery::new()
            .with_from(Date::parse("2020-01-01").unwrap())
            .with_to(Date::parse("2024-01-01").unwrap()),
    );
    let list = cot_report_list();

    assert_common(&report, "commitment-of-traders-report");
    assert_common(&analysis, "commitment-of-traders-analysis");
    assert_common(&list, "commitment-of-traders-list");
    assert_eq!(report.metadata().bounds(), EndpointBounds::new());
    assert_eq!(list.metadata().bounds(), EndpointBounds::new());
    assert_eq!(
        analysis.metadata().bounds(),
        EndpointBounds::new().with_date_range_days(90)
    );
    assert_eq!(list.query(), &());
}

fn assert_common<Q, R>(endpoint: &EndpointSpec<Q, R>, path: &'static str) {
    assert_eq!(endpoint.method(), HttpMethod::Get);
    assert_eq!(endpoint.id(), path);
    assert_eq!(endpoint.relative_path(), path);
    assert_eq!(
        endpoint.metadata().geography(),
        GeographicAvailability::Unspecified
    );
    assert_eq!(endpoint.metadata().access(), AccessRequirement::Unspecified);
    assert_eq!(endpoint.metadata().conditional_plan(), None);
    assert_eq!(endpoint.metadata().realtime(), None);
    assert_eq!(endpoint.metadata().bounds().limit(), None);
    assert_eq!(endpoint.metadata().bounds().response_rows(), None);
    assert_eq!(endpoint.metadata().bounds().page(), None);
}

#[test]
fn query_builders_getters_and_independent_values_preserve_documented_order() {
    let symbol = Ticker::new("VX / Index").unwrap();
    let from = Date::parse("2024-01-01").unwrap();
    let to = Date::parse("2024-03-01").unwrap();
    let query = CotQuery::new()
        .with_symbol(symbol.clone())
        .with_from(from)
        .with_to(to);
    assert_eq!(query.symbol(), Some(&symbol));
    assert_eq!(query.from(), Some(from));
    assert_eq!(query.to(), Some(to));

    let only_from = cot_report(CotQuery::new().with_from(from));
    assert!(only_from.query().symbol().is_none());
    assert_eq!(only_from.query().from(), Some(from));
    assert_eq!(only_from.query().to(), None);

    // The endpoint records the 90-day maximum as metadata without rejecting
    // construction of a broader caller-selected interval.
    let broader = cot_analysis(
        CotQuery::new()
            .with_from(Date::parse("2020-01-01").unwrap())
            .with_to(Date::parse("2024-01-01").unwrap()),
    );
    assert_eq!(broader.query().from().unwrap().to_string(), "2020-01-01");
}

#[tokio::test]
async fn proxy_routes_all_methods_with_exact_queries_auth_headers_and_fixtures() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(REPORT),
        json_fixture(ANALYSIS),
        json_fixture(REPORT_LIST),
    ]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("/gateway/stable/")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "cot")
        .executor(executor.clone())
        .build()
        .unwrap();
    let query = CotQuery::new()
        .with_symbol(Ticker::new("VX / Index").unwrap())
        .with_from(Date::parse("2024-01-01").unwrap())
        .with_to(Date::parse("2024-03-01").unwrap());

    assert_eq!(
        client.cot_report(query.clone()).await.unwrap()[0].name,
        "CBOE VIX (VX)"
    );
    assert_eq!(
        client.cot_analysis(query).await.unwrap()[0].net_position,
        -12_315
    );
    assert_eq!(
        client.cot_report_list().await.unwrap()[0].symbol.as_str(),
        "NG"
    );

    let requests = executor.requests();
    assert_eq!(requests.len(), 3);
    assert!(requests.iter().all(|request| {
        request.method() == HttpMethod::Get
            && request.expose_headers()["x-router-token"] == "Token proxy-secret"
            && request.expose_headers()["x-data-scope"] == "cot"
    }));
    assert_eq!(
        request_urls(&requests),
        [
            "https://proxy.example/router/gateway/stable/commitment-of-traders-report?symbol=VX+%2F+Index&from=2024-01-01&to=2024-03-01",
            "https://proxy.example/router/gateway/stable/commitment-of-traders-analysis?symbol=VX+%2F+Index&from=2024-01-01&to=2024-03-01",
            "https://proxy.example/router/gateway/stable/commitment-of-traders-list",
        ]
    );
}

#[tokio::test]
async fn optional_query_values_are_independent_and_omitted() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(REPORT),
        json_fixture(ANALYSIS),
    ]));
    let client = Client::builder()
        .base_url("https://proxy.example")
        .authentication(Authentication::None)
        .executor(executor.clone())
        .build()
        .unwrap();

    client
        .cot_report(CotQuery::new().with_to(Date::parse("2024-03-01").unwrap()))
        .await
        .unwrap();
    client
        .cot_analysis(CotQuery::new().with_symbol(Ticker::new("PA").unwrap()))
        .await
        .unwrap();

    assert_eq!(
        request_urls(&executor.requests()),
        [
            "https://proxy.example/stable/commitment-of-traders-report?to=2024-03-01",
            "https://proxy.example/stable/commitment-of-traders-analysis?symbol=PA",
        ]
    );
}

#[tokio::test]
async fn direct_fmp_header_and_query_auth_preserve_cot_contracts() {
    for (authentication, report, expected_url, expected_header) in [
        (
            Authentication::fmp_header("header-secret"),
            true,
            "https://financialmodelingprep.com/stable/commitment-of-traders-report?symbol=VX",
            Some("header-secret"),
        ),
        (
            Authentication::fmp_query("query-secret"),
            false,
            "https://financialmodelingprep.com/stable/commitment-of-traders-list?apikey=query-secret",
            None,
        ),
    ] {
        let executor = Arc::new(FixtureExecutor::new([json_fixture(if report {
            REPORT
        } else {
            REPORT_LIST
        })]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();

        if report {
            client
                .cot_report(CotQuery::new().with_symbol(Ticker::new("VX").unwrap()))
                .await
                .unwrap();
        } else {
            client.cot_report_list().await.unwrap();
        }

        let requests = executor.requests();
        assert_eq!(requests[0].expose_url().as_str(), expected_url);
        match expected_header {
            Some(value) => assert_eq!(requests[0].expose_headers()["apikey"], value),
            None => assert!(!requests[0].expose_headers().contains_key("apikey")),
        }
    }
}

#[tokio::test]
async fn malformed_non_arrays_preserve_all_endpoint_identities() {
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

    let errors = [
        client.cot_report(CotQuery::new()).await.unwrap_err(),
        client.cot_analysis(CotQuery::new()).await.unwrap_err(),
        client.cot_report_list().await.unwrap_err(),
    ];
    for (error, id) in errors.iter().zip([
        "commitment-of-traders-report",
        "commitment-of-traders-analysis",
        "commitment-of-traders-list",
    ]) {
        assert_eq!(error.category(), ErrorCategory::Decode);
        assert_eq!(error.endpoint(), Some(id));
        assert_eq!(error.status_code(), Some(200));
    }
}

fn request_urls(requests: &[libfmp::transport::PreparedRequest]) -> Vec<String> {
    requests
        .iter()
        .map(|request| request.expose_url().as_str().to_owned())
        .collect()
}
