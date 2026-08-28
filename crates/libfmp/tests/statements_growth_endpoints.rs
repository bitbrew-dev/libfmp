mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
        statements::{
            BalanceSheetStatementGrowthQuery, IncomeStatementGrowthQuery,
            balance_sheet_statement_growth, income_statement_growth,
        },
    },
    query::{FiscalPeriod, RetrievalFrequency},
    transport::HttpMethod,
    types::{Limit, Ticker},
};

use support::{FixtureExecutor, json_fixture};

const INCOME: &[u8] = include_bytes!("fixtures/income_statement_growth.json");
const BALANCE: &[u8] = include_bytes!("fixtures/balance_sheet_statement_growth.json");

#[test]
fn descriptors_use_exact_paths_distinct_rows_and_documented_metadata() {
    let symbol = Ticker::new("AAPL").unwrap();
    let income_query = IncomeStatementGrowthQuery::new(symbol.clone())
        .with_limit(Limit(1_000))
        .with_period(RetrievalFrequency::Annual);
    let balance_query = BalanceSheetStatementGrowthQuery::new(symbol.clone())
        .with_limit(Limit(1_000))
        .with_period(FiscalPeriod::FullYear);

    assert_eq!(income_query.symbol(), &symbol);
    assert_eq!(income_query.limit(), Some(Limit(1_000)));
    assert_eq!(
        income_query.period(),
        Some(RetrievalFrequency::Annual.into())
    );
    assert_eq!(balance_query.symbol(), &symbol);
    assert_eq!(balance_query.limit(), Some(Limit(1_000)));
    assert_eq!(balance_query.period(), Some(FiscalPeriod::FullYear.into()));

    assert_facts(
        &income_statement_growth(income_query),
        "income-statement-growth",
    );
    assert_facts(
        &balance_sheet_statement_growth(balance_query),
        "balance-sheet-statement-growth",
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
async fn custom_proxy_preserves_exact_query_order_auth_headers_and_bare_arrays() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(INCOME),
        json_fixture(BALANCE),
    ]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("stable")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "statement-growth")
        .executor(executor.clone())
        .build()
        .unwrap();
    let symbol = Ticker::new("BRK.B / Class A").unwrap();

    let income = client
        .income_statement_growth(
            IncomeStatementGrowthQuery::new(symbol.clone())
                .with_limit(Limit(5))
                .with_period(FiscalPeriod::Q1),
        )
        .await
        .unwrap();
    let balance = client
        .balance_sheet_statement_growth(
            BalanceSheetStatementGrowthQuery::new(symbol)
                .with_limit(Limit(7))
                .with_period(RetrievalFrequency::Quarterly),
        )
        .await
        .unwrap();

    assert_eq!(income.len(), 1);
    assert_eq!(income[0].growth_selling_and_marketing_expenses, -1.0);
    assert_eq!(balance.len(), 1);
    assert_eq!(balance[0].growth_tax_payables, -1.0);

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
    let urls = requests
        .iter()
        .map(|request| request.expose_url().as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        urls,
        [
            "https://proxy.example/router/stable/income-statement-growth?symbol=BRK.B+%2F+Class+A&limit=5&period=Q1",
            "https://proxy.example/router/stable/balance-sheet-statement-growth?symbol=BRK.B+%2F+Class+A&limit=7&period=quarter",
        ]
    );
}

#[tokio::test]
async fn direct_header_and_query_auth_use_the_same_typed_growth_contracts() {
    for (authentication, expected_url, expected_header) in [
        (
            Authentication::fmp_header("header-secret"),
            "https://financialmodelingprep.com/stable/income-statement-growth?symbol=AAPL",
            Some(("apikey", "header-secret")),
        ),
        (
            Authentication::fmp_query("query-secret"),
            "https://financialmodelingprep.com/stable/balance-sheet-statement-growth?symbol=AAPL&apikey=query-secret",
            None,
        ),
    ] {
        let response = if expected_header.is_some() {
            INCOME
        } else {
            BALANCE
        };
        let executor = Arc::new(FixtureExecutor::new([json_fixture(response)]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();

        if expected_header.is_some() {
            client
                .income_statement_growth(Ticker::new("AAPL").unwrap())
                .await
                .unwrap();
        } else {
            client
                .balance_sheet_statement_growth(Ticker::new("AAPL").unwrap())
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
