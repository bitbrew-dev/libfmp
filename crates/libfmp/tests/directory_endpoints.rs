mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    codecs::TrueFalseFlag,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        directory::{
            CikListQuery, SymbolChangesQuery, actively_trading, cik_list, company_symbols,
            earnings_transcript_list, etf_symbols, financial_statement_symbols, symbol_changes,
        },
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
    },
    transport::HttpMethod,
    types::{Limit, Page},
};

use support::{FixtureExecutor, json_fixture};

const COMPANY: &[u8] = include_bytes!("fixtures/directory_company_symbols.json");
const FINANCIAL: &[u8] = include_bytes!("fixtures/directory_financial_statement_symbols.json");
const CIK: &[u8] = include_bytes!("fixtures/directory_cik_list.json");
const CHANGES: &[u8] = include_bytes!("fixtures/directory_symbol_changes.json");
const ETF: &[u8] = include_bytes!("fixtures/directory_etf_symbols.json");
const ACTIVE: &[u8] = include_bytes!("fixtures/directory_actively_trading.json");
const TRANSCRIPTS: &[u8] = include_bytes!("fixtures/directory_earnings_transcript_list.json");
const EMPTY: &[u8] = include_bytes!("fixtures/directory_empty.json");

#[test]
fn descriptors_use_exact_paths_and_only_documented_geography_and_bounds() {
    let cik_query = CikListQuery::new()
        .with_page(Page(0))
        .with_limit(Limit(1_000));
    let changes_query = SymbolChangesQuery::new()
        .with_invalid(TrueFalseFlag::False)
        .with_limit(Limit(100));

    assert_eq!(cik_query.page(), Some(Page(0)));
    assert_eq!(cik_query.limit(), Some(Limit(1_000)));
    assert_eq!(changes_query.invalid(), Some(TrueFalseFlag::False));
    assert_eq!(changes_query.limit(), Some(Limit(100)));

    let endpoint_facts = [
        facts(&company_symbols()),
        facts(&financial_statement_symbols()),
        facts(&cik_list(cik_query)),
        facts(&symbol_changes(changes_query)),
        facts(&etf_symbols()),
        facts(&actively_trading()),
        facts(&earnings_transcript_list()),
    ];
    assert_eq!(
        endpoint_facts,
        [
            ("stock-list", GeographicAvailability::Worldwide),
            (
                "financial-statement-symbol-list",
                GeographicAvailability::Worldwide
            ),
            ("cik-list", GeographicAvailability::UsOnly),
            ("symbol-change", GeographicAvailability::UsOnly),
            ("etf-list", GeographicAvailability::Worldwide),
            ("actively-trading-list", GeographicAvailability::Worldwide),
            ("earnings-transcript-list", GeographicAvailability::UsOnly),
        ]
    );

    let cik_bounds = cik_list(CikListQuery::new()).metadata().bounds();
    assert_eq!(cik_bounds.limit(), None);
    assert_eq!(cik_bounds.page(), None);
    assert!(cik_bounds.accepts_response_rows(10_000));
    assert!(!cik_bounds.accepts_response_rows(10_001));
    assert!(cik_bounds.accepts_limit(Limit(10_001)));

    for endpoint in [
        company_symbols().metadata(),
        financial_statement_symbols().metadata(),
        symbol_changes(SymbolChangesQuery::new()).metadata(),
        etf_symbols().metadata(),
        actively_trading().metadata(),
        earnings_transcript_list().metadata(),
    ] {
        assert_eq!(endpoint.bounds(), EndpointBounds::new());
    }
}

fn facts<Q, R>(endpoint: &EndpointSpec<Q, R>) -> (&'static str, GeographicAvailability) {
    assert_eq!(endpoint.method(), HttpMethod::Get);
    assert_eq!(endpoint.id(), endpoint.relative_path());
    assert_eq!(endpoint.metadata().access(), AccessRequirement::Unspecified);
    assert_eq!(endpoint.metadata().conditional_plan(), None);
    assert_eq!(endpoint.metadata().realtime(), None);
    (endpoint.id(), endpoint.metadata().geography())
}

#[tokio::test]
async fn proxy_client_executes_all_directory_paths_with_exact_queries_and_no_stray_markers() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(COMPANY),
        json_fixture(FINANCIAL),
        json_fixture(CIK),
        json_fixture(CHANGES),
        json_fixture(ETF),
        json_fixture(ACTIVE),
        json_fixture(TRANSCRIPTS),
        json_fixture(CIK),
        json_fixture(CHANGES),
    ]));
    let client = proxy_client(executor.clone());

    assert_eq!(client.company_symbols().await.unwrap().len(), 1);
    assert_eq!(client.financial_statement_symbols().await.unwrap().len(), 1);
    assert_eq!(
        client.cik_list(CikListQuery::new()).await.unwrap()[0]
            .cik
            .as_str(),
        "0002137358"
    );
    assert_eq!(
        client
            .symbol_changes(SymbolChangesQuery::new())
            .await
            .unwrap()
            .len(),
        1
    );
    assert_eq!(client.etf_symbols().await.unwrap().len(), 1);
    assert_eq!(client.actively_trading().await.unwrap().len(), 1);
    assert_eq!(
        client.earnings_transcript_list().await.unwrap()[0]
            .no_of_transcripts
            .as_str(),
        "6"
    );
    client
        .cik_list(
            CikListQuery::new()
                .with_page(Page(0))
                .with_limit(Limit(10_001)),
        )
        .await
        .unwrap();
    client
        .symbol_changes(
            SymbolChangesQuery::new()
                .with_invalid(TrueFalseFlag::False)
                .with_limit(Limit(100)),
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
            "https://proxy.example/router/stable/stock-list",
            "https://proxy.example/router/stable/financial-statement-symbol-list",
            "https://proxy.example/router/stable/cik-list",
            "https://proxy.example/router/stable/symbol-change",
            "https://proxy.example/router/stable/etf-list",
            "https://proxy.example/router/stable/actively-trading-list",
            "https://proxy.example/router/stable/earnings-transcript-list",
            "https://proxy.example/router/stable/cik-list?page=0&limit=10001",
            "https://proxy.example/router/stable/symbol-change?invalid=false&limit=100",
        ]
    );
    assert!(urls[..7].iter().all(|url| !url.contains('?')));
    assert!(
        requests
            .iter()
            .all(|request| request.method() == HttpMethod::Get)
    );
}

#[tokio::test]
async fn client_preserves_empty_arrays_for_all_seven_directory_endpoints() {
    let executor = Arc::new(FixtureExecutor::new(
        std::iter::repeat_with(|| json_fixture(EMPTY)).take(7),
    ));
    let client = proxy_client(executor);

    assert!(client.company_symbols().await.unwrap().is_empty());
    assert!(
        client
            .financial_statement_symbols()
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        client
            .cik_list(CikListQuery::new())
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        client
            .symbol_changes(SymbolChangesQuery::new())
            .await
            .unwrap()
            .is_empty()
    );
    assert!(client.etf_symbols().await.unwrap().is_empty());
    assert!(client.actively_trading().await.unwrap().is_empty());
    assert!(client.earnings_transcript_list().await.unwrap().is_empty());
}

#[tokio::test]
async fn direct_fmp_auth_uses_the_same_company_symbols_contract() {
    for (authentication, expected_url, expected_header) in [
        (
            Authentication::fmp_header("header-secret"),
            "https://financialmodelingprep.com/stable/stock-list",
            Some(("apikey", "header-secret")),
        ),
        (
            Authentication::fmp_query("query-secret"),
            "https://financialmodelingprep.com/stable/stock-list?apikey=query-secret",
            None,
        ),
    ] {
        let executor = Arc::new(FixtureExecutor::new([json_fixture(COMPANY)]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();

        client.company_symbols().await.unwrap();

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
