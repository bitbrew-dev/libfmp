mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
        statements::{
            CashFlowStatementGrowthQuery, FinancialStatementGrowthQuery,
            cash_flow_statement_growth, financial_statement_growth,
        },
    },
    query::{FiscalPeriod, RetrievalFrequency},
    transport::HttpMethod,
    types::{Limit, Ticker},
};

use support::{FixtureExecutor, json_fixture};

const CASH: &[u8] = include_bytes!("fixtures/cash_flow_statement_growth.json");
const COMBINED: &[u8] = include_bytes!("fixtures/financial_statement_growth.json");

#[test]
fn descriptors_use_exact_paths_distinct_rows_and_documented_metadata() {
    let symbol = Ticker::new("AAPL").unwrap();
    let cash_query = CashFlowStatementGrowthQuery::new(symbol.clone())
        .with_limit(Limit(1_000))
        .with_period(RetrievalFrequency::Annual);
    let combined_query = FinancialStatementGrowthQuery::new(symbol.clone())
        .with_limit(Limit(1_000))
        .with_period(FiscalPeriod::FullYear);

    assert_eq!(cash_query.symbol(), &symbol);
    assert_eq!(cash_query.limit(), Some(Limit(1_000)));
    assert_eq!(cash_query.period(), Some(RetrievalFrequency::Annual.into()));
    assert_eq!(combined_query.symbol(), &symbol);
    assert_eq!(combined_query.limit(), Some(Limit(1_000)));
    assert_eq!(combined_query.period(), Some(FiscalPeriod::FullYear.into()));

    assert_facts(
        &cash_flow_statement_growth(cash_query),
        "cash-flow-statement-growth",
    );
    assert_facts(
        &financial_statement_growth(combined_query),
        "financial-growth",
    );
}

fn assert_facts<Q, R>(endpoint: &EndpointSpec<Q, R>, path: &'static str) {
    assert_eq!(endpoint.method(), HttpMethod::Get);
    assert_eq!(endpoint.id(), path);
    assert_eq!(endpoint.relative_path(), path);
    assert_eq!(
        endpoint.metadata().geography(),
        GeographicAvailability::Worldwide
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
async fn custom_proxy_auth_headers_preserve_exact_paths_queries_and_bare_arrays() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(CASH),
        json_fixture(COMBINED),
    ]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("stable")
        .default_header("x-data-scope", "statement-growth")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .executor(executor.clone())
        .build()
        .unwrap();
    let symbol = Ticker::new("BRK.B / Class A").unwrap();

    let cash = client
        .cash_flow_statement_growth(
            CashFlowStatementGrowthQuery::new(symbol.clone())
                .with_limit(Limit(5))
                .with_period(FiscalPeriod::Q1),
        )
        .await
        .unwrap();
    let combined = client
        .financial_statement_growth(
            FinancialStatementGrowthQuery::new(symbol)
                .with_limit(Limit(7))
                .with_period(RetrievalFrequency::Quarterly),
        )
        .await
        .unwrap();

    assert_eq!(cash[0].growth_net_change_in_cash, 8.545340050377833);
    assert_eq!(combined[0].eps_growth, 0.22585924713584285);

    let requests = executor.requests();
    assert!(
        requests
            .iter()
            .all(|request| request.method() == HttpMethod::Get)
    );
    assert!(requests.iter().all(|request| {
        request.expose_headers()["x-router-token"] == "Token proxy-secret"
            && request.expose_headers()["x-data-scope"] == "statement-growth"
    }));
    assert_eq!(
        requests
            .iter()
            .map(|request| request.expose_url().as_str())
            .collect::<Vec<_>>(),
        [
            "https://proxy.example/router/stable/cash-flow-statement-growth?symbol=BRK.B+%2F+Class+A&limit=5&period=Q1",
            "https://proxy.example/router/stable/financial-growth?symbol=BRK.B+%2F+Class+A&limit=7&period=quarter",
        ]
    );
}

#[tokio::test]
async fn direct_header_and_query_auth_apply_to_both_growth_contracts() {
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
            json_fixture(COMBINED),
        ]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();
        let symbol = Ticker::new("AAPL").unwrap();

        client
            .cash_flow_statement_growth(
                CashFlowStatementGrowthQuery::new(symbol.clone()).with_limit(Limit(5)),
            )
            .await
            .unwrap();
        client
            .financial_statement_growth(
                FinancialStatementGrowthQuery::new(symbol).with_period(FiscalPeriod::Q2),
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
                    "https://financialmodelingprep.com/stable/cash-flow-statement-growth?symbol=AAPL&limit=5{query_suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/financial-growth?symbol=AAPL&period=Q2{query_suffix}"
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
