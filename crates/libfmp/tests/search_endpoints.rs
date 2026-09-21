mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
        search::{
            CikSearchQuery, CusipSearchQuery, ExchangeVariantsQuery, IsinSearchQuery,
            NameSearchQuery, SymbolSearchQuery, search_cik, search_cusip, search_exchange_variants,
            search_isin, search_name, search_symbol,
        },
    },
    transport::HttpMethod,
    types::{Cik, Cusip, ExchangeCode, Isin, Limit, SearchTerm, Ticker},
};

use support::{FixtureExecutor, json_fixture};

const SYMBOL: &[u8] = include_bytes!("fixtures/search_symbol.json");
const NAME: &[u8] = include_bytes!("fixtures/search_name.json");
const CIK: &[u8] = include_bytes!("fixtures/search_cik.json");
const CUSIP: &[u8] = include_bytes!("fixtures/search_cusip.json");
const ISIN: &[u8] = include_bytes!("fixtures/search_isin.json");
const VARIANTS: &[u8] = include_bytes!("fixtures/search_exchange_variants.json");
const EMPTY: &[u8] = include_bytes!("fixtures/search_empty.json");
const MULTIPLE: &[u8] = include_bytes!("fixtures/search_symbol_multiple.json");
const UNKNOWN: &[u8] = include_bytes!("fixtures/search_symbol_unknown.json");

#[test]
fn descriptors_use_exact_paths_queries_and_only_sourced_geography() {
    let symbol_query = SymbolSearchQuery::new(SearchTerm::new("AAPL").unwrap())
        .with_limit(Limit(50))
        .with_exchange(ExchangeCode::new("NASDAQ").unwrap());
    let name_query = NameSearchQuery::new(SearchTerm::new("AA").unwrap());
    let cik_query = CikSearchQuery::new(Cik::new("320193").unwrap()).with_limit(Limit(50));
    let cusip_query = CusipSearchQuery::new(Cusip::new("037833100").unwrap());
    let isin_query = IsinSearchQuery::new(Isin::new("US0378331005").unwrap());
    let variants_query = ExchangeVariantsQuery::new(Ticker::new("AAPL").unwrap());

    assert_eq!(symbol_query.query().as_str(), "AAPL");
    assert_eq!(symbol_query.limit(), Some(Limit(50)));
    assert_eq!(symbol_query.exchange().unwrap().as_str(), "NASDAQ");
    assert_eq!(name_query.query().as_str(), "AA");
    assert_eq!(name_query.limit(), None);
    assert_eq!(name_query.exchange(), None);
    assert_eq!(cik_query.cik().as_str(), "320193");
    assert_eq!(cik_query.limit(), Some(Limit(50)));
    assert_eq!(cusip_query.cusip().as_str(), "037833100");
    assert_eq!(isin_query.isin().as_str(), "US0378331005");
    assert_eq!(variants_query.symbol().as_str(), "AAPL");

    let endpoints = [
        endpoint_facts(&search_symbol(symbol_query)),
        endpoint_facts(&search_name(name_query)),
        endpoint_facts(&search_cik(cik_query)),
        endpoint_facts(&search_cusip(cusip_query)),
        endpoint_facts(&search_isin(isin_query)),
        endpoint_facts(&search_exchange_variants(variants_query)),
    ];
    assert_eq!(
        endpoints,
        [
            ("search-symbol", GeographicAvailability::Worldwide),
            ("search-name", GeographicAvailability::Worldwide),
            ("search-cik", GeographicAvailability::UsOnly),
            ("search-cusip", GeographicAvailability::Worldwide),
            ("search-isin", GeographicAvailability::Worldwide),
            (
                "search-exchange-variants",
                GeographicAvailability::Worldwide
            ),
        ]
    );
}

fn endpoint_facts<Q, R>(
    endpoint: &libfmp::endpoints::EndpointSpec<Q, R>,
) -> (&'static str, GeographicAvailability) {
    assert_eq!(endpoint.method(), HttpMethod::Get);
    assert_eq!(endpoint.id(), endpoint.relative_path());
    assert_eq!(endpoint.metadata().access(), AccessRequirement::Unspecified);
    assert_eq!(endpoint.metadata().bounds(), EndpointBounds::new());
    assert_eq!(endpoint.metadata().conditional_plan(), None);
    assert_eq!(endpoint.metadata().realtime(), None);
    (endpoint.id(), endpoint.metadata().geography())
}

#[tokio::test]
async fn proxy_client_executes_all_search_paths_with_exact_query_order_and_encoding() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(SYMBOL),
        json_fixture(NAME),
        json_fixture(CIK),
        json_fixture(CUSIP),
        json_fixture(ISIN),
        json_fixture(VARIANTS),
    ]));
    let client = proxy_client(executor.clone(), Authentication::None);

    client
        .search_symbol(
            SymbolSearchQuery::new(SearchTerm::new("Apple / Class A").unwrap())
                .with_limit(Limit(u32::MAX))
                .with_exchange(ExchangeCode::new("NASDAQ Global").unwrap()),
        )
        .await
        .unwrap();
    client
        .search_name(
            NameSearchQuery::new(SearchTerm::new("AA").unwrap())
                .with_limit(Limit(0))
                .with_exchange(ExchangeCode::new("CRYPTO").unwrap()),
        )
        .await
        .unwrap();
    client
        .search_cik(CikSearchQuery::new(Cik::new("0000320193").unwrap()).with_limit(Limit(50)))
        .await
        .unwrap();
    client
        .search_cusip(Cusip::new("037833100").unwrap())
        .await
        .unwrap();
    client
        .search_isin(Isin::new("US0378331005").unwrap())
        .await
        .unwrap();
    client
        .search_exchange_variants(Ticker::new("^VIX").unwrap())
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
            "https://proxy.example/router/stable/search-symbol?query=Apple+%2F+Class+A&limit=4294967295&exchange=NASDAQ+Global",
            "https://proxy.example/router/stable/search-name?query=AA&limit=0&exchange=CRYPTO",
            "https://proxy.example/router/stable/search-cik?cik=0000320193&limit=50",
            "https://proxy.example/router/stable/search-cusip?cusip=037833100",
            "https://proxy.example/router/stable/search-isin?isin=US0378331005",
            "https://proxy.example/router/stable/search-exchange-variants?symbol=%5EVIX",
        ]
    );
    assert!(
        requests
            .iter()
            .all(|request| request.method() == HttpMethod::Get)
    );
}

#[tokio::test]
async fn direct_fmp_auth_executes_the_same_typed_search_contracts() {
    for (authentication, expected_url, expected_header) in [
        (
            Authentication::fmp_header("header-secret"),
            "https://financialmodelingprep.com/stable/search-symbol?query=AAPL",
            Some(("apikey", "header-secret")),
        ),
        (
            Authentication::fmp_query("query-secret"),
            "https://financialmodelingprep.com/stable/search-symbol?query=AAPL&apikey=query-secret",
            None,
        ),
    ] {
        let executor = Arc::new(FixtureExecutor::new([json_fixture(SYMBOL)]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();

        client
            .search_symbol(SymbolSearchQuery::new(SearchTerm::new("AAPL").unwrap()))
            .await
            .unwrap();

        let requests = executor.requests();
        assert_eq!(requests[0].expose_url().as_str(), expected_url);
        match expected_header {
            Some((name, value)) => assert_eq!(requests[0].expose_headers()[name], value),
            None => assert!(!requests[0].expose_headers().contains_key("apikey")),
        }
    }
}

#[tokio::test]
async fn client_preserves_empty_multiple_and_unknown_field_arrays() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(EMPTY),
        json_fixture(MULTIPLE),
        json_fixture(UNKNOWN),
    ]));
    let client = proxy_client(executor, Authentication::None);
    let query = || SymbolSearchQuery::new(SearchTerm::new("A").unwrap());

    let empty = client.search_symbol(query()).await.unwrap();
    let multiple = client.search_symbol(query()).await.unwrap();
    let unknown = client.search_symbol(query()).await.unwrap();

    assert!(empty.is_empty());
    assert_eq!(multiple.len(), 2);
    assert_eq!(multiple[0].symbol.as_str(), "000001.SZ");
    assert_eq!(multiple[1].symbol.as_str(), "^VIX");
    assert_eq!(unknown.len(), 1);
    assert_eq!(unknown[0].symbol.as_str(), "AAPL");
}

fn proxy_client(executor: Arc<FixtureExecutor>, authentication: Authentication) -> Client {
    Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("stable")
        .authentication(authentication)
        .executor(executor)
        .build()
        .unwrap()
}
