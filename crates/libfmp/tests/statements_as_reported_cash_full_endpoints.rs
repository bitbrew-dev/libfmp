mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
        statements::{
            CashFlowStatementAsReportedQuery, FinancialStatementFullAsReportedQuery,
            cash_flow_statement_as_reported, financial_statement_full_as_reported,
        },
    },
    query::RetrievalFrequency,
    transport::HttpMethod,
    types::{Limit, Ticker},
};

use support::{FixtureExecutor, json_fixture};

const CASH: &[u8] = include_bytes!("fixtures/cash_flow_statement_as_reported.json");
const FULL: &[u8] = include_bytes!("fixtures/financial_statement_full_as_reported.json");

#[test]
fn descriptors_use_exact_paths_shared_bare_row_and_only_documented_response_bound() {
    let symbol = Ticker::new("AAPL").unwrap();
    let cash_query = CashFlowStatementAsReportedQuery::new(symbol.clone())
        .with_limit(Limit(1_000))
        .with_period(RetrievalFrequency::Annual);
    let full_query = FinancialStatementFullAsReportedQuery::new(symbol.clone())
        .with_limit(Limit(1_000))
        .with_period(RetrievalFrequency::Quarterly);

    assert_eq!(cash_query.symbol(), &symbol);
    assert_eq!(cash_query.limit(), Some(Limit(1_000)));
    assert_eq!(cash_query.period(), Some(RetrievalFrequency::Annual));
    assert_eq!(full_query.symbol(), &symbol);
    assert_eq!(full_query.limit(), Some(Limit(1_000)));
    assert_eq!(full_query.period(), Some(RetrievalFrequency::Quarterly));

    assert_facts(
        &cash_flow_statement_as_reported(cash_query),
        "cash-flow-statement-as-reported",
    );
    assert_facts(
        &financial_statement_full_as_reported(full_query),
        "financial-statement-full-as-reported",
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
async fn custom_proxy_preserves_exact_queries_auth_default_headers_and_dynamic_arrays() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(CASH),
        json_fixture(FULL),
    ]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("stable")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "as-reported-raw")
        .executor(executor.clone())
        .build()
        .unwrap();
    let symbol = Ticker::new("BRK.B / Class A").unwrap();

    let cash = client
        .cash_flow_statement_as_reported(
            CashFlowStatementAsReportedQuery::new(symbol.clone())
                .with_limit(Limit(5))
                .with_period(RetrievalFrequency::Annual),
        )
        .await
        .unwrap();
    let full = client
        .financial_statement_full_as_reported(
            FinancialStatementFullAsReportedQuery::new(symbol)
                .with_limit(Limit(7))
                .with_period(RetrievalFrequency::Quarterly),
        )
        .await
        .unwrap();

    assert_eq!(cash[0].data.len(), 28);
    assert_eq!(full[0].data.len(), 300);

    let requests = executor.requests();
    assert!(
        requests
            .iter()
            .all(|request| request.method() == HttpMethod::Get)
    );
    assert!(requests.iter().all(|request| {
        request.expose_headers()["x-router-token"] == "Token proxy-secret"
            && request.expose_headers()["x-data-scope"] == "as-reported-raw"
    }));
    assert_eq!(
        requests
            .iter()
            .map(|request| request.expose_url().as_str())
            .collect::<Vec<_>>(),
        [
            "https://proxy.example/router/stable/cash-flow-statement-as-reported?symbol=BRK.B+%2F+Class+A&limit=5&period=annual",
            "https://proxy.example/router/stable/financial-statement-full-as-reported?symbol=BRK.B+%2F+Class+A&limit=7&period=quarter",
        ]
    );
}

#[tokio::test]
async fn direct_header_and_query_auth_apply_to_both_as_reported_contracts() {
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
            json_fixture(CASH),
            json_fixture(FULL),
        ]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();
        let symbol = Ticker::new("AAPL").unwrap();

        client
            .cash_flow_statement_as_reported(
                CashFlowStatementAsReportedQuery::new(symbol.clone()).with_limit(Limit(5)),
            )
            .await
            .unwrap();
        client
            .financial_statement_full_as_reported(
                FinancialStatementFullAsReportedQuery::new(symbol)
                    .with_period(RetrievalFrequency::Quarterly),
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
                    "https://financialmodelingprep.com/stable/cash-flow-statement-as-reported?symbol=AAPL&limit=5{query_suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/financial-statement-full-as-reported?symbol=AAPL&period=quarter{query_suffix}"
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
