mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        company::{
            DelistedCompaniesQuery, EmployeeCountQuery, HistoricalEmployeeCountQuery,
            delisted_companies, employee_count, historical_employee_count,
        },
        metadata::{AccessRequirement, GeographicAvailability},
    },
    transport::HttpMethod,
    types::{Limit, Page, Ticker},
};

use support::{FixtureExecutor, json_fixture};

const DELISTED: &[u8] = include_bytes!("fixtures/company_delisted.json");
const EMPLOYEES: &[u8] = include_bytes!("fixtures/company_employee_count.json");
const EMPTY: &[u8] = include_bytes!("fixtures/company_empty.json");

#[test]
fn descriptors_use_exact_paths_queries_geography_and_response_row_bounds() {
    let delisted_query = DelistedCompaniesQuery::new()
        .with_page(Page(0))
        .with_limit(Limit(101));
    let current_query =
        EmployeeCountQuery::new(Ticker::new("AAPL").unwrap()).with_limit(Limit(10_001));
    let historical_query =
        HistoricalEmployeeCountQuery::new(Ticker::new("AAPL").unwrap()).with_limit(Limit(10_001));

    assert_eq!(delisted_query.page(), Some(Page(0)));
    assert_eq!(delisted_query.limit(), Some(Limit(101)));
    assert_eq!(current_query.symbol().as_str(), "AAPL");
    assert_eq!(current_query.limit(), Some(Limit(10_001)));
    assert_eq!(historical_query.symbol().as_str(), "AAPL");
    assert_eq!(historical_query.limit(), Some(Limit(10_001)));

    let delisted = delisted_companies(delisted_query);
    let current = employee_count(current_query);
    let historical = historical_employee_count(historical_query);
    assert_eq!(facts(&delisted), "delisted-companies");
    assert_eq!(facts(&current), "employee-count");
    assert_eq!(facts(&historical), "historical-employee-count");

    let delisted_bounds = delisted.metadata().bounds();
    assert_eq!(delisted_bounds.limit(), None);
    assert_eq!(delisted_bounds.page(), None);
    assert!(delisted_bounds.accepts_limit(Limit(u32::MAX)));
    assert!(delisted_bounds.accepts_page(Page(u32::MAX)));
    assert!(delisted_bounds.accepts_response_rows(100));
    assert!(!delisted_bounds.accepts_response_rows(101));

    for bounds in [current.metadata().bounds(), historical.metadata().bounds()] {
        assert_eq!(bounds.limit(), None);
        assert_eq!(bounds.page(), None);
        assert!(bounds.accepts_limit(Limit(u32::MAX)));
        assert!(bounds.accepts_response_rows(10_000));
        assert!(!bounds.accepts_response_rows(10_001));
    }
}

fn facts<Q, R>(endpoint: &EndpointSpec<Q, R>) -> &'static str {
    assert_eq!(endpoint.method(), HttpMethod::Get);
    assert_eq!(endpoint.id(), endpoint.relative_path());
    assert_eq!(
        endpoint.metadata().geography(),
        GeographicAvailability::UsOnly
    );
    assert_eq!(endpoint.metadata().access(), AccessRequirement::Unspecified);
    assert_eq!(endpoint.metadata().conditional_plan(), None);
    assert_eq!(endpoint.metadata().realtime(), None);
    endpoint.id()
}

#[tokio::test]
async fn proxy_client_omits_absent_options_and_emits_present_options_in_documented_order() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(DELISTED),
        json_fixture(DELISTED),
        json_fixture(EMPLOYEES),
        json_fixture(EMPLOYEES),
        json_fixture(EMPLOYEES),
        json_fixture(EMPLOYEES),
    ]));
    let client = proxy_client(executor.clone());

    assert_eq!(
        client
            .delisted_companies(DelistedCompaniesQuery::new())
            .await
            .unwrap()[0]
            .symbol
            .as_str(),
        "CCIX"
    );
    client
        .delisted_companies(
            DelistedCompaniesQuery::new()
                .with_page(Page(0))
                .with_limit(Limit(101)),
        )
        .await
        .unwrap();
    assert_eq!(
        client
            .employee_count(Ticker::new("BRK.B / Class A").unwrap())
            .await
            .unwrap()[0]
            .cik
            .as_str(),
        "0000320193"
    );
    client
        .employee_count(
            EmployeeCountQuery::new(Ticker::new("AAPL").unwrap()).with_limit(Limit(10_001)),
        )
        .await
        .unwrap();
    client
        .historical_employee_count(Ticker::new("AAPL").unwrap())
        .await
        .unwrap();
    client
        .historical_employee_count(
            HistoricalEmployeeCountQuery::new(Ticker::new("AAPL").unwrap())
                .with_limit(Limit(10_001)),
        )
        .await
        .unwrap();

    let requests = executor.requests();
    let urls = requests
        .iter()
        .map(|request| request.expose_url().as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        urls,
        [
            "https://proxy.example/router/stable/delisted-companies",
            "https://proxy.example/router/stable/delisted-companies?page=0&limit=101",
            "https://proxy.example/router/stable/employee-count?symbol=BRK.B+%2F+Class+A",
            "https://proxy.example/router/stable/employee-count?symbol=AAPL&limit=10001",
            "https://proxy.example/router/stable/historical-employee-count?symbol=AAPL",
            "https://proxy.example/router/stable/historical-employee-count?symbol=AAPL&limit=10001",
        ]
    );
    assert!(!urls[0].contains('?'));
    assert!(
        requests
            .iter()
            .all(|request| request.method() == HttpMethod::Get)
    );
}

#[tokio::test]
async fn every_workforce_client_method_preserves_empty_arrays() {
    let executor = Arc::new(FixtureExecutor::new(
        std::iter::repeat_with(|| json_fixture(EMPTY)).take(3),
    ));
    let client = proxy_client(executor);

    assert!(
        client
            .delisted_companies(DelistedCompaniesQuery::new())
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        client
            .employee_count(Ticker::new("AAPL").unwrap())
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        client
            .historical_employee_count(Ticker::new("AAPL").unwrap())
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn direct_fmp_auth_uses_the_same_workforce_contracts() {
    for (authentication, expected_url, expected_header) in [
        (
            Authentication::fmp_header("header-secret"),
            "https://financialmodelingprep.com/stable/delisted-companies?page=0",
            Some(("apikey", "header-secret")),
        ),
        (
            Authentication::fmp_query("query-secret"),
            "https://financialmodelingprep.com/stable/employee-count?symbol=AAPL&apikey=query-secret",
            None,
        ),
    ] {
        let response = if expected_header.is_some() {
            DELISTED
        } else {
            EMPLOYEES
        };
        let executor = Arc::new(FixtureExecutor::new([json_fixture(response)]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();

        if expected_header.is_some() {
            client
                .delisted_companies(DelistedCompaniesQuery::new().with_page(Page(0)))
                .await
                .unwrap();
        } else {
            client
                .employee_count(Ticker::new("AAPL").unwrap())
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

fn proxy_client(executor: Arc<FixtureExecutor>) -> Client {
    Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("stable")
        .authentication(Authentication::None)
        .executor(executor)
        .build()
        .unwrap()
}
