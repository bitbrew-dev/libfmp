mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        metadata::{
            AccessRequirement, DelayScope, EndpointBounds, GeographicAvailability, MarketDataDelay,
            RealtimeAccess, UserDeclarationRequirement,
        },
        quote::{
            ExchangeQuotesQuery, ShortOnlyQuery, commodity_quotes, cryptocurrency_quotes,
            etf_quotes, exchange_quotes, forex_quotes, index_quotes, mutual_fund_quotes,
        },
    },
    transport::HttpMethod,
    types::ExchangeCode,
};

use support::{FixtureExecutor, json_fixture};

const EXCHANGE: &[u8] = include_bytes!("fixtures/quote_exchange_short.json");
const MUTUAL_FUND: &[u8] = include_bytes!("fixtures/quote_mutual_fund_short.json");
const ETF: &[u8] = include_bytes!("fixtures/quote_etf_short.json");
const COMMODITY: &[u8] = include_bytes!("fixtures/quote_commodity_short.json");
const CRYPTOCURRENCY: &[u8] = include_bytes!("fixtures/quote_crypto_short.json");
const FOREX: &[u8] = include_bytes!("fixtures/quote_forex_short.json");
const INDEX: &[u8] = include_bytes!("fixtures/quote_index_short.json");

#[test]
fn descriptors_use_exact_paths_and_documented_metadata() {
    let exchange = ExchangeCode::new("NASDAQ").unwrap();
    let query = ExchangeQuotesQuery::new(exchange.clone());
    assert_eq!(query.exchange(), &exchange);

    assert_facts(
        &exchange_quotes(query),
        "batch-exchange-quote",
        GeographicAvailability::Worldwide,
        true,
    );
    assert_facts(
        &mutual_fund_quotes(),
        "batch-mutualfund-quotes",
        GeographicAvailability::UsOnly,
        false,
    );
    assert_facts(
        &etf_quotes(),
        "batch-etf-quotes",
        GeographicAvailability::Worldwide,
        false,
    );

    for (endpoint, path) in [
        (commodity_quotes(), "batch-commodity-quotes"),
        (cryptocurrency_quotes(), "batch-crypto-quotes"),
        (forex_quotes(), "batch-forex-quotes"),
        (index_quotes(), "batch-index-quotes"),
    ] {
        assert_eq!(endpoint.id(), path);
        assert_eq!(endpoint.relative_path(), path);
        assert_eq!(
            endpoint.metadata().geography(),
            GeographicAvailability::Unspecified
        );
        assert_common_facts(&endpoint);
        assert_eq!(endpoint.metadata().realtime(), None);
    }
}

fn assert_facts<Q, R>(
    endpoint: &EndpointSpec<Q, R>,
    path: &'static str,
    geography: GeographicAvailability,
    delayed_realtime: bool,
) {
    assert_eq!(endpoint.id(), path);
    assert_eq!(endpoint.relative_path(), path);
    assert_eq!(endpoint.metadata().geography(), geography);
    assert_common_facts(endpoint);

    if delayed_realtime {
        assert_eq!(
            endpoint.metadata().realtime(),
            Some(RealtimeAccess::new(
                Some(MarketDataDelay::new(15, DelayScope::Nasdaq)),
                Some(UserDeclarationRequirement::RequiredForRealtime),
            ))
        );
    } else {
        assert_eq!(endpoint.metadata().realtime(), None);
    }
}

fn assert_common_facts<Q, R>(endpoint: &EndpointSpec<Q, R>) {
    assert_eq!(endpoint.method(), HttpMethod::Get);
    assert_eq!(endpoint.metadata().access(), AccessRequirement::Unspecified);
    assert_eq!(endpoint.metadata().bounds(), EndpointBounds::new());
    assert_eq!(endpoint.metadata().conditional_plan(), None);
}

#[test]
fn short_only_query_is_zero_sized_and_exposes_no_boolean_choice() {
    let exchange = exchange_quotes(ExchangeQuotesQuery::new(
        ExchangeCode::new("NASDAQ Global / Select").unwrap(),
    ));
    assert_eq!(
        exchange.query().exchange().as_str(),
        "NASDAQ Global / Select"
    );
    assert_eq!(ShortOnlyQuery::new(), ShortOnlyQuery);
    assert_eq!(std::mem::size_of::<ShortOnlyQuery>(), 0);
}

#[tokio::test]
async fn proxy_client_uses_every_exact_path_and_decodes_documented_short_rows() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(EXCHANGE),
        json_fixture(MUTUAL_FUND),
        json_fixture(ETF),
        json_fixture(COMMODITY),
        json_fixture(CRYPTOCURRENCY),
        json_fixture(FOREX),
        json_fixture(INDEX),
    ]));
    let client = proxy_client(
        executor.clone(),
        Authentication::custom_header("x-router-token", Some("Token ".to_owned()), "proxy-secret"),
    );

    let exchange = client
        .exchange_quotes(ExchangeCode::new("NASDAQ Global / Select").unwrap())
        .await
        .unwrap();
    let mutual_funds = client.mutual_fund_quotes().await.unwrap();
    let etfs = client.etf_quotes().await.unwrap();
    let commodities = client.commodity_quotes().await.unwrap();
    let cryptocurrencies = client.cryptocurrency_quotes().await.unwrap();
    let forex = client.forex_quotes().await.unwrap();
    let indexes = client.index_quotes().await.unwrap();

    assert_eq!(exchange[0].symbol.as_str(), "AAACX");
    assert_eq!(mutual_funds[0].price, 44.0);
    assert_eq!(etfs[0].volume, 1.0);
    assert_eq!(commodities[0].change, 0.02);
    assert_eq!(cryptocurrencies[0].symbol.as_str(), "00USD");
    assert_eq!(forex[0].price, 0.38716);
    assert_eq!(indexes[0].symbol.as_str(), "^SPROME10");

    let requests = executor.requests();
    assert!(
        requests
            .iter()
            .all(|request| request.method() == HttpMethod::Get)
    );
    assert!(
        requests
            .iter()
            .all(|request| { request.expose_headers()["x-router-token"] == "Token proxy-secret" })
    );
    let urls = requests
        .iter()
        .map(|request| request.expose_url().as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        urls,
        [
            "https://proxy.example/router/stable/batch-exchange-quote?exchange=NASDAQ+Global+%2F+Select&short=true",
            "https://proxy.example/router/stable/batch-mutualfund-quotes?short=true",
            "https://proxy.example/router/stable/batch-etf-quotes?short=true",
            "https://proxy.example/router/stable/batch-commodity-quotes?short=true",
            "https://proxy.example/router/stable/batch-crypto-quotes?short=true",
            "https://proxy.example/router/stable/batch-forex-quotes?short=true",
            "https://proxy.example/router/stable/batch-index-quotes?short=true",
        ]
    );
}

#[tokio::test]
async fn direct_header_and_query_auth_use_the_same_closed_short_contracts() {
    for (authentication, expected_url, expected_header) in [
        (
            Authentication::fmp_header("header-secret"),
            "https://financialmodelingprep.com/stable/batch-etf-quotes?short=true",
            Some(("apikey", "header-secret")),
        ),
        (
            Authentication::fmp_query("query-secret"),
            "https://financialmodelingprep.com/stable/batch-index-quotes?short=true&apikey=query-secret",
            None,
        ),
    ] {
        let executor = Arc::new(FixtureExecutor::new([json_fixture(QUOTE_SHORT)]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();

        if expected_header.is_some() {
            client.etf_quotes().await.unwrap();
        } else {
            client.index_quotes().await.unwrap();
        }

        let requests = executor.requests();
        assert_eq!(requests[0].expose_url().as_str(), expected_url);
        match expected_header {
            Some((name, value)) => assert_eq!(requests[0].expose_headers()[name], value),
            None => assert!(!requests[0].expose_headers().contains_key("apikey")),
        }
    }
}

const QUOTE_SHORT: &[u8] = include_bytes!("fixtures/quote_short.json");

fn proxy_client(executor: Arc<FixtureExecutor>, authentication: Authentication) -> Client {
    Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("stable")
        .authentication(authentication)
        .executor(executor)
        .build()
        .unwrap()
}
