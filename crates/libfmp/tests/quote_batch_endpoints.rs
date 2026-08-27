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
            BatchAftermarketQuoteQuery, BatchAftermarketTradeQuery, BatchQuoteQuery,
            BatchQuoteShortQuery, batch_aftermarket_quote, batch_aftermarket_trade, batch_quote,
            batch_quote_short,
        },
    },
    transport::HttpMethod,
    types::{Ticker, TickerList, UnixMilliseconds},
};

use support::{FixtureExecutor, json_fixture};

const QUOTE: &[u8] = include_bytes!("fixtures/quote.json");
const QUOTE_SHORT: &[u8] = include_bytes!("fixtures/quote_short.json");
const AFTERMARKET_TRADE: &[u8] = include_bytes!("fixtures/aftermarket_trade.json");
const AFTERMARKET_QUOTE: &[u8] = include_bytes!("fixtures/aftermarket_quote.json");

#[test]
fn descriptors_use_exact_paths_queries_and_documented_metadata() {
    assert!(TickerList::new(Vec::new()).is_err());

    let symbols = tickers(["AAPL", "MSFT"]);
    let quote_query = BatchQuoteQuery::new(symbols.clone());
    let short_query = BatchQuoteShortQuery::new(symbols.clone());
    let trade_query = BatchAftermarketTradeQuery::new(symbols.clone());
    let aftermarket_quote_query = BatchAftermarketQuoteQuery::new(symbols.clone());

    for query_symbols in [
        quote_query.symbols(),
        short_query.symbols(),
        trade_query.symbols(),
        aftermarket_quote_query.symbols(),
    ] {
        assert_eq!(query_symbols.as_slice(), symbols.as_slice());
    }

    assert_facts(
        &batch_quote(quote_query),
        "batch-quote",
        GeographicAvailability::Worldwide,
    );
    assert_facts(
        &batch_quote_short(short_query),
        "batch-quote-short",
        GeographicAvailability::Worldwide,
    );
    assert_facts(
        &batch_aftermarket_trade(trade_query),
        "batch-aftermarket-trade",
        GeographicAvailability::UsOnly,
    );
    assert_facts(
        &batch_aftermarket_quote(aftermarket_quote_query),
        "batch-aftermarket-quote",
        GeographicAvailability::UsOnly,
    );
}

fn assert_facts<Q, R>(
    endpoint: &EndpointSpec<Q, R>,
    path: &'static str,
    geography: GeographicAvailability,
) {
    assert_eq!(endpoint.method(), HttpMethod::Get);
    assert_eq!(endpoint.id(), path);
    assert_eq!(endpoint.relative_path(), path);
    assert_eq!(endpoint.metadata().geography(), geography);
    assert_eq!(endpoint.metadata().access(), AccessRequirement::Unspecified);
    assert_eq!(endpoint.metadata().bounds(), EndpointBounds::new());
    assert_eq!(endpoint.metadata().conditional_plan(), None);
    assert_eq!(
        endpoint.metadata().realtime(),
        Some(RealtimeAccess::new(
            Some(MarketDataDelay::new(15, DelayScope::Nasdaq)),
            Some(UserDeclarationRequirement::RequiredForRealtime),
        ))
    );
}

#[tokio::test]
async fn proxy_client_preserves_symbol_order_comma_encoding_and_decodes_every_shape() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(QUOTE),
        json_fixture(QUOTE_SHORT),
        json_fixture(AFTERMARKET_TRADE),
        json_fixture(AFTERMARKET_QUOTE),
    ]));
    let client = proxy_client(
        executor.clone(),
        Authentication::custom_header("x-router-token", Some("Token ".to_owned()), "proxy-secret"),
    );
    let symbols = tickers(["BRK.B / Class A", "^VIX", "000001.SZ"]);

    let quotes = client.batch_quote(&symbols).await.unwrap();
    let short_quotes = client.batch_quote_short(&symbols).await.unwrap();
    let trades = client.batch_aftermarket_trade(&symbols).await.unwrap();
    let aftermarket_quotes = client.batch_aftermarket_quote(&symbols).await.unwrap();

    assert_eq!(quotes[0].name, "Apple Inc.");
    assert_eq!(short_quotes[0].volume, 28_718_014);
    assert_eq!(trades[0].trade_size, 16);
    assert_eq!(
        aftermarket_quotes[0].timestamp,
        UnixMilliseconds(1_785_430_813_000)
    );

    let requests = executor.requests();
    assert!(
        requests
            .iter()
            .all(|request| request.method() == HttpMethod::Get)
    );
    assert!(
        requests
            .iter()
            .all(|request| request.expose_headers()["x-router-token"] == "Token proxy-secret")
    );
    let urls = requests
        .iter()
        .map(|request| request.expose_url().as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        urls,
        [
            "https://proxy.example/router/stable/batch-quote?symbols=BRK.B+%2F+Class+A%2C%5EVIX%2C000001.SZ",
            "https://proxy.example/router/stable/batch-quote-short?symbols=BRK.B+%2F+Class+A%2C%5EVIX%2C000001.SZ",
            "https://proxy.example/router/stable/batch-aftermarket-trade?symbols=BRK.B+%2F+Class+A%2C%5EVIX%2C000001.SZ",
            "https://proxy.example/router/stable/batch-aftermarket-quote?symbols=BRK.B+%2F+Class+A%2C%5EVIX%2C000001.SZ",
        ]
    );
}

#[tokio::test]
async fn direct_header_and_query_auth_use_the_same_typed_batch_contracts() {
    for (authentication, expected_url, expected_header) in [
        (
            Authentication::fmp_header("header-secret"),
            "https://financialmodelingprep.com/stable/batch-quote?symbols=AAPL%2CMSFT",
            Some(("apikey", "header-secret")),
        ),
        (
            Authentication::fmp_query("query-secret"),
            "https://financialmodelingprep.com/stable/batch-aftermarket-quote?symbols=AAPL%2CMSFT&apikey=query-secret",
            None,
        ),
    ] {
        let response = if expected_header.is_some() {
            QUOTE
        } else {
            AFTERMARKET_QUOTE
        };
        let executor = Arc::new(FixtureExecutor::new([json_fixture(response)]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();
        let symbols = tickers(["AAPL", "MSFT"]);

        if expected_header.is_some() {
            client.batch_quote(symbols).await.unwrap();
        } else {
            client.batch_aftermarket_quote(symbols).await.unwrap();
        }

        let requests = executor.requests();
        assert_eq!(requests[0].expose_url().as_str(), expected_url);
        match expected_header {
            Some((name, value)) => assert_eq!(requests[0].expose_headers()[name], value),
            None => assert!(!requests[0].expose_headers().contains_key("apikey")),
        }
    }
}

fn tickers<const N: usize>(symbols: [&str; N]) -> TickerList {
    TickerList::new(
        symbols
            .into_iter()
            .map(|symbol| Ticker::new(symbol).unwrap())
            .collect(),
    )
    .unwrap()
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
