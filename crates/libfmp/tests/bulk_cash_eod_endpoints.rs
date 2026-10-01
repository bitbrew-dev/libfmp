mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        bulk::{
            BulkEodQuery, BulkStatementQuery, bulk_cash_flow_statement_growth,
            bulk_cash_flow_statements, bulk_eod,
        },
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
    },
    error::ErrorCategory,
    query::{FiscalPeriod, Year},
    responses::bulk::{BulkCashFlowStatement, BulkCashFlowStatementGrowth, BulkEodBar},
    transport::{HttpMethod, TransportResponse},
    types::Date,
};

use support::{FixtureExecutor, fixture_response, json_fixture};

const CASH_FLOW: &[u8] = include_bytes!("fixtures/bulk_cash_flow_statements.csv");
const GROWTH: &[u8] = include_bytes!("fixtures/bulk_cash_flow_statement_growth.csv");
const EOD: &[u8] = include_bytes!("fixtures/bulk_eod.csv");

fn csv_fixture(body: &'static [u8]) -> TransportResponse {
    fixture_response(Some("text/csv"), body)
}

fn fixtures() -> [TransportResponse; 3] {
    [
        csv_fixture(CASH_FLOW),
        csv_fixture(GROWTH),
        csv_fixture(EOD),
    ]
}

#[test]
fn descriptors_have_exact_get_ids_paths_response_types_and_only_documented_metadata() {
    let statement = BulkStatementQuery::new(Year(2026), FiscalPeriod::Q1);
    let eod = BulkEodQuery::new(Date::parse("2024-10-22").unwrap());
    assert_facts(
        &bulk_cash_flow_statements(statement),
        "cash-flow-statement-bulk",
    );
    assert_facts(
        &bulk_cash_flow_statement_growth(statement),
        "cash-flow-statement-growth-bulk",
    );
    assert_facts(&bulk_eod(eod), "eod-bulk");
    assert_response_types(
        &bulk_cash_flow_statements(statement),
        &bulk_cash_flow_statement_growth(statement),
        &bulk_eod(eod),
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
    _: &EndpointSpec<BulkStatementQuery, Vec<BulkCashFlowStatement>>,
    _: &EndpointSpec<BulkStatementQuery, Vec<BulkCashFlowStatementGrowth>>,
    _: &EndpointSpec<BulkEodQuery, Vec<BulkEodBar>>,
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
        client.bulk_cash_flow_statements(query).await.unwrap();
    }

    assert_eq!(
        request_urls(&executor.requests()),
        [
            "https://financialmodelingprep.com/stable/cash-flow-statement-bulk?year=4294967295&period=Q1",
            "https://financialmodelingprep.com/stable/cash-flow-statement-bulk?year=4294967295&period=Q2",
            "https://financialmodelingprep.com/stable/cash-flow-statement-bulk?year=4294967295&period=Q3",
            "https://financialmodelingprep.com/stable/cash-flow-statement-bulk?year=4294967295&period=Q4",
            "https://financialmodelingprep.com/stable/cash-flow-statement-bulk?year=4294967295&period=FY",
        ]
    );
}

#[tokio::test]
async fn custom_proxy_sends_one_exact_request_per_method_with_auth_headers_and_decodes_rows() {
    let executor = Arc::new(FixtureExecutor::new(fixtures()));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("gateway/stable")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Bearer ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "bulk cash eod")
        .executor(executor.clone())
        .build()
        .unwrap();
    let statement = BulkStatementQuery::new(Year(2026), FiscalPeriod::Q1);
    let date = Date::parse("2024-10-22").unwrap();

    let cash_flow = client.bulk_cash_flow_statements(statement).await.unwrap();
    let growth = client
        .bulk_cash_flow_statement_growth(statement)
        .await
        .unwrap();
    let eod = client.bulk_eod(date).await.unwrap();

    assert_eq!(cash_flow[0].symbol.as_str(), "000001.SZ");
    assert_eq!(cash_flow[0].cik.as_str(), "0000000000");
    assert_eq!(growth[0].reported_currency.as_str(), "CNY");
    assert_eq!(growth[0].period, FiscalPeriod::FullYear);
    assert_eq!(eod[0].symbol.as_str(), "PCFD.SG");
    assert_eq!(eod[0].date, Date::parse("2025-06-02").unwrap());

    let requests = executor.requests();
    assert_eq!(requests.len(), 3);
    assert!(requests.iter().all(|request| {
        request.method() == HttpMethod::Get
            && request.expose_headers()["x-router-token"] == "Bearer proxy-secret"
            && request.expose_headers()["x-data-scope"] == "bulk cash eod"
    }));
    assert_eq!(
        request_urls(&requests),
        [
            "https://proxy.example/router/gateway/stable/cash-flow-statement-bulk?year=2026&period=Q1",
            "https://proxy.example/router/gateway/stable/cash-flow-statement-growth-bulk?year=2026&period=Q1",
            "https://proxy.example/router/gateway/stable/eod-bulk?date=2024-10-22",
        ]
    );
}

#[tokio::test]
async fn direct_fmp_header_and_query_auth_preserve_paths_and_exact_query_order() {
    for (authentication, statement_suffix, eod_suffix, expected_header) in [
        (
            Authentication::fmp_header("header-secret"),
            "?year=2026&period=Q1",
            "?date=2024-10-22",
            Some("header-secret"),
        ),
        (
            Authentication::fmp_query("query-secret"),
            "?year=2026&period=Q1&apikey=query-secret",
            "?date=2024-10-22&apikey=query-secret",
            None,
        ),
    ] {
        let executor = Arc::new(FixtureExecutor::new(fixtures()));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();
        let statement = BulkStatementQuery::new(Year(2026), FiscalPeriod::Q1);

        client.bulk_cash_flow_statements(statement).await.unwrap();
        client
            .bulk_cash_flow_statement_growth(statement)
            .await
            .unwrap();
        client
            .bulk_eod(Date::parse("2024-10-22").unwrap())
            .await
            .unwrap();

        let requests = executor.requests();
        assert_eq!(requests.len(), 3);
        assert_eq!(
            request_urls(&requests),
            [
                format!(
                    "https://financialmodelingprep.com/stable/cash-flow-statement-bulk{statement_suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/cash-flow-statement-growth-bulk{statement_suffix}"
                ),
                format!("https://financialmodelingprep.com/stable/eod-bulk{eod_suffix}"),
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
async fn empty_bodies_decode_and_json_bodies_keep_all_endpoint_identities() {
    let executor = Arc::new(FixtureExecutor::new([
        csv_fixture(b""),
        csv_fixture(b""),
        csv_fixture(b""),
        json_fixture(b"[]"),
        json_fixture(b"[]"),
        json_fixture(b"[]"),
    ]));
    let client = Client::builder()
        .authentication(Authentication::fmp_header("secret"))
        .executor(executor.clone())
        .build()
        .unwrap();
    let statement = BulkStatementQuery::new(Year(2026), FiscalPeriod::Q1);
    let eod = BulkEodQuery::new(Date::parse("2024-10-22").unwrap());

    assert!(
        client
            .bulk_cash_flow_statements(statement)
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        client
            .bulk_cash_flow_statement_growth(statement)
            .await
            .unwrap()
            .is_empty()
    );
    assert!(client.bulk_eod(eod).await.unwrap().is_empty());
    let errors = [
        client
            .bulk_cash_flow_statements(statement)
            .await
            .unwrap_err(),
        client
            .bulk_cash_flow_statement_growth(statement)
            .await
            .unwrap_err(),
        client.bulk_eod(eod).await.unwrap_err(),
    ];
    for (error, id) in errors.iter().zip([
        "cash-flow-statement-bulk",
        "cash-flow-statement-growth-bulk",
        "eod-bulk",
    ]) {
        assert_eq!(error.category(), ErrorCategory::Decode);
        assert_eq!(error.endpoint(), Some(id));
        assert_eq!(error.status_code(), Some(200));
    }
    assert_eq!(executor.requests().len(), 6);
}

fn request_urls(requests: &[libfmp::transport::PreparedRequest]) -> Vec<String> {
    requests
        .iter()
        .map(|request| request.expose_url().as_str().to_owned())
        .collect()
}
