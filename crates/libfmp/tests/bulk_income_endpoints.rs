mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        bulk::{BulkStatementQuery, bulk_income_statement_growth, bulk_income_statements},
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
    },
    error::ErrorCategory,
    query::{FiscalPeriod, Year},
    responses::bulk::{BulkIncomeStatement, BulkIncomeStatementGrowth},
    transport::{HttpMethod, TransportResponse},
};

use support::{FixtureExecutor, fixture_response, json_fixture};

const INCOME: &[u8] = include_bytes!("fixtures/bulk_income_statements.csv");
const GROWTH: &[u8] = include_bytes!("fixtures/bulk_income_statement_growth.csv");

fn csv_fixture(body: &'static [u8]) -> TransportResponse {
    fixture_response(Some("text/csv"), body)
}

fn fixtures() -> [TransportResponse; 2] {
    [csv_fixture(INCOME), csv_fixture(GROWTH)]
}

#[test]
fn descriptors_have_exact_get_ids_paths_response_types_and_only_documented_metadata() {
    let query = BulkStatementQuery::new(Year(2026), FiscalPeriod::Q1);
    assert_facts(&bulk_income_statements(query), "income-statement-bulk");
    assert_facts(
        &bulk_income_statement_growth(query),
        "income-statement-growth-bulk",
    );
    assert_response_types(
        &bulk_income_statements(query),
        &bulk_income_statement_growth(query),
    );
}

fn assert_facts<Q, R>(endpoint: &EndpointSpec<Q, Vec<R>>, path: &'static str) {
    assert_eq!(endpoint.method(), HttpMethod::Get);
    assert_eq!(endpoint.id(), path);
    assert_eq!(endpoint.relative_path(), path);
    assert_eq!(
        endpoint.metadata().geography(),
        GeographicAvailability::Worldwide
    );
    assert_eq!(endpoint.metadata().access(), AccessRequirement::Unspecified);
    assert_eq!(endpoint.metadata().bounds(), EndpointBounds::new());
    assert_eq!(endpoint.metadata().conditional_plan(), None);
    assert_eq!(endpoint.metadata().realtime(), None);
}

fn assert_response_types(
    _: &EndpointSpec<BulkStatementQuery, Vec<BulkIncomeStatement>>,
    _: &EndpointSpec<BulkStatementQuery, Vec<BulkIncomeStatementGrowth>>,
) {
}

#[tokio::test]
async fn all_five_periods_encode_required_year_then_period_without_defaults_or_ranges() {
    let executor = Arc::new(FixtureExecutor::new(
        std::iter::repeat_with(|| csv_fixture(b"")).take(5),
    ));
    let client = Client::builder()
        .authentication(Authentication::fmp_header("secret"))
        .executor(executor.clone())
        .build()
        .unwrap();

    for period in [
        FiscalPeriod::Q1,
        FiscalPeriod::Q2,
        FiscalPeriod::Q3,
        FiscalPeriod::Q4,
        FiscalPeriod::FullYear,
    ] {
        let query = BulkStatementQuery::new(Year(u32::MAX), period);
        assert_eq!(query.year(), Year(u32::MAX));
        assert_eq!(query.period(), period);
        client.bulk_income_statements(query).await.unwrap();
    }

    assert_eq!(
        request_urls(&executor.requests()),
        [
            "https://financialmodelingprep.com/stable/income-statement-bulk?year=4294967295&period=Q1",
            "https://financialmodelingprep.com/stable/income-statement-bulk?year=4294967295&period=Q2",
            "https://financialmodelingprep.com/stable/income-statement-bulk?year=4294967295&period=Q3",
            "https://financialmodelingprep.com/stable/income-statement-bulk?year=4294967295&period=Q4",
            "https://financialmodelingprep.com/stable/income-statement-bulk?year=4294967295&period=FY",
        ]
    );
}

#[tokio::test]
async fn custom_proxy_sends_one_exact_request_per_method_with_auth_headers_and_decodes_identities()
{
    let executor = Arc::new(FixtureExecutor::new(fixtures()));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("gateway/stable")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Bearer ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "bulk income")
        .executor(executor.clone())
        .build()
        .unwrap();
    let query = BulkStatementQuery::new(Year(2026), FiscalPeriod::Q1);

    let income = client.bulk_income_statements(query).await.unwrap();
    let growth = client.bulk_income_statement_growth(query).await.unwrap();

    assert_eq!(income[0].symbol.as_str(), "000001.SZ");
    assert_eq!(income[0].cik.as_str(), "0000000000");
    assert_eq!(income[0].accepted_date.to_string(), "2024-12-31 00:00:00");
    assert_eq!(growth[0].reported_currency.as_str(), "CNY");
    assert_eq!(growth[0].period, FiscalPeriod::FullYear);

    let requests = executor.requests();
    assert_eq!(requests.len(), 2);
    assert!(requests.iter().all(|request| {
        request.method() == HttpMethod::Get
            && request.expose_headers()["x-router-token"] == "Bearer proxy-secret"
            && request.expose_headers()["x-data-scope"] == "bulk income"
    }));
    assert_eq!(
        request_urls(&requests),
        [
            "https://proxy.example/router/gateway/stable/income-statement-bulk?year=2026&period=Q1",
            "https://proxy.example/router/gateway/stable/income-statement-growth-bulk?year=2026&period=Q1",
        ]
    );
}

#[tokio::test]
async fn direct_fmp_header_and_query_auth_preserve_paths_and_exact_query_order() {
    for (authentication, suffix, expected_header) in [
        (
            Authentication::fmp_header("header-secret"),
            "?year=2026&period=Q1",
            Some("header-secret"),
        ),
        (
            Authentication::fmp_query("query-secret"),
            "?year=2026&period=Q1&apikey=query-secret",
            None,
        ),
    ] {
        let executor = Arc::new(FixtureExecutor::new(fixtures()));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();
        let query = BulkStatementQuery::new(Year(2026), FiscalPeriod::Q1);

        client.bulk_income_statements(query).await.unwrap();
        client.bulk_income_statement_growth(query).await.unwrap();

        let requests = executor.requests();
        assert_eq!(requests.len(), 2);
        assert_eq!(
            request_urls(&requests),
            [
                format!("https://financialmodelingprep.com/stable/income-statement-bulk{suffix}"),
                format!(
                    "https://financialmodelingprep.com/stable/income-statement-growth-bulk{suffix}"
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
async fn empty_bodies_decode_and_json_bodies_keep_both_endpoint_identities() {
    let executor = Arc::new(FixtureExecutor::new([
        csv_fixture(b""),
        csv_fixture(b""),
        json_fixture(b"[]"),
        json_fixture(b"[]"),
    ]));
    let client = Client::builder()
        .authentication(Authentication::fmp_header("secret"))
        .executor(executor.clone())
        .build()
        .unwrap();
    let query = BulkStatementQuery::new(Year(2026), FiscalPeriod::Q1);

    assert!(
        client
            .bulk_income_statements(query)
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        client
            .bulk_income_statement_growth(query)
            .await
            .unwrap()
            .is_empty()
    );
    let errors = [
        client.bulk_income_statements(query).await.unwrap_err(),
        client
            .bulk_income_statement_growth(query)
            .await
            .unwrap_err(),
    ];
    for (error, id) in errors
        .iter()
        .zip(["income-statement-bulk", "income-statement-growth-bulk"])
    {
        assert_eq!(error.category(), ErrorCategory::Decode);
        assert_eq!(error.endpoint(), Some(id));
        assert_eq!(error.status_code(), Some(200));
    }
    assert_eq!(executor.requests().len(), 4);
}

fn request_urls(requests: &[libfmp::transport::PreparedRequest]) -> Vec<String> {
    requests
        .iter()
        .map(|request| request.expose_url().as_str().to_owned())
        .collect()
}
