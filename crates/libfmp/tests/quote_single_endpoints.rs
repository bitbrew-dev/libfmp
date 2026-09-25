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
            AftermarketQuoteQuery, AftermarketTradeQuery, QuoteQuery, QuoteShortQuery,
            StockPriceChangeQuery, aftermarket_quote, aftermarket_trade, quote, quote_short,
            stock_price_change,
        },
    },
    transport::HttpMethod,
    types::{Ticker, UnixMilliseconds},
};

use support::{FixtureExecutor, json_fixture};

const QUOTE: &[u8] = include_bytes!("fixtures/quote.json");
const QUOTE_SHORT: &[u8] = include_bytes!("fixtures/quote_short.json");
const AFTERMARKET_TRADE: &[u8] = include_bytes!("fixtures/aftermarket_trade.json");
const AFTERMARKET_QUOTE: &[u8] = include_bytes!("fixtures/aftermarket_quote.json");
const STOCK_PRICE_CHANGE: &[u8] = include_bytes!("fixtures/stock_price_change.json");

#[test]
fn descriptors_use_exact_paths_queries_and_documented_metadata() {
    let symbol = Ticker::new("AAPL").unwrap();
    let quote_query = QuoteQuery::new(symbol.clone());
    let quote_short_query = QuoteShortQuery::new(symbol.clone());
    let trade_query = AftermarketTradeQuery::new(symbol.clone());
    let aftermarket_quote_query = AftermarketQuoteQuery::new(symbol.clone());
    let change_query = StockPriceChangeQuery::new(symbol.clone());

    assert_eq!(quote_query.symbol(), &symbol);
    assert_eq!(quote_short_query.symbol(), &symbol);
    assert_eq!(trade_query.symbol(), &symbol);
    assert_eq!(aftermarket_quote_query.symbol(), &symbol);
    assert_eq!(change_query.symbol(), &symbol);

    assert_facts(
        &quote(quote_query),
        "quote",
        GeographicAvailability::Worldwide,
        true,
    );
    assert_facts(
        &quote_short(quote_short_query),
        "quote-short",
        GeographicAvailability::Worldwide,
        true,
    );
    assert_facts(
        &aftermarket_trade(trade_query),
        "aftermarket-trade",
        GeographicAvailability::UsOnly,
        true,
    );
    assert_facts(
        &aftermarket_quote(aftermarket_quote_query),
        "aftermarket-quote",
        GeographicAvailability::UsOnly,
        true,
    );
    assert_facts(
        &stock_price_change(change_query),
        "stock-price-change",
        GeographicAvailability::Worldwide,
        false,
    );
}

fn assert_facts<Q, R>(
    endpoint: &EndpointSpec<Q, R>,
    path: &'static str,
    geography: GeographicAvailability,
    delayed_realtime: bool,
) {
    assert_eq!(endpoint.method(), HttpMethod::Get);
    assert_eq!(endpoint.id(), path);
    assert_eq!(endpoint.relative_path(), path);
    assert_eq!(endpoint.metadata().geography(), geography);
    assert_eq!(endpoint.metadata().access(), AccessRequirement::Unspecified);
    assert_eq!(endpoint.metadata().bounds(), EndpointBounds::new());
    assert_eq!(endpoint.metadata().conditional_plan(), None);

    let realtime = endpoint.metadata().realtime();
    if delayed_realtime {
        let realtime = realtime.expect("documented Nasdaq real-time caveat");
        assert_eq!(
            realtime,
            RealtimeAccess::new(
                Some(MarketDataDelay::new(15, DelayScope::Nasdaq)),
                Some(UserDeclarationRequirement::RequiredForRealtime),
            )
        );
    } else {
        assert_eq!(realtime, None);
    }
}

#[tokio::test]
async fn proxy_client_uses_exact_get_paths_and_required_symbol_encoding() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(QUOTE),
        json_fixture(QUOTE_SHORT),
        json_fixture(AFTERMARKET_TRADE),
        json_fixture(AFTERMARKET_QUOTE),
        json_fixture(STOCK_PRICE_CHANGE),
    ]));
    let client = proxy_client(
        executor.clone(),
        Authentication::custom_header("x-router-token", Some("Token ".to_owned()), "proxy-secret"),
    );
    let symbol = Ticker::new("BRK.B / Class A").unwrap();

    let quotes = client.quote(&symbol).await.unwrap();
    let short_quotes = client.quote_short(&symbol).await.unwrap();
    let trades = client.aftermarket_trade(&symbol).await.unwrap();
    let aftermarket_quotes = client.aftermarket_quote(&symbol).await.unwrap();
    let changes = client.stock_price_change(&symbol).await.unwrap();

    assert_eq!(quotes[0].name, "Apple Inc.");
    assert_eq!(short_quotes[0].volume, 28_718_014.0);
    assert_eq!(trades[0].trade_size, 16.0);
    assert_eq!(
        aftermarket_quotes[0].timestamp,
        UnixMilliseconds(1_785_430_813_000)
    );
    assert_eq!(changes[0].ten_years, 1151.81068);

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
            "https://proxy.example/router/stable/quote?symbol=BRK.B+%2F+Class+A",
            "https://proxy.example/router/stable/quote-short?symbol=BRK.B+%2F+Class+A",
            "https://proxy.example/router/stable/aftermarket-trade?symbol=BRK.B+%2F+Class+A",
            "https://proxy.example/router/stable/aftermarket-quote?symbol=BRK.B+%2F+Class+A",
            "https://proxy.example/router/stable/stock-price-change?symbol=BRK.B+%2F+Class+A",
        ]
    );
}

#[tokio::test]
async fn direct_header_and_query_auth_use_the_same_typed_contracts() {
    for (authentication, expected_url, expected_header) in [
        (
            Authentication::fmp_header("header-secret"),
            "https://financialmodelingprep.com/stable/quote?symbol=AAPL",
            Some(("apikey", "header-secret")),
        ),
        (
            Authentication::fmp_query("query-secret"),
            "https://financialmodelingprep.com/stable/stock-price-change?symbol=AAPL&apikey=query-secret",
            None,
        ),
    ] {
        let response = if expected_header.is_some() {
            QUOTE
        } else {
            STOCK_PRICE_CHANGE
        };
        let executor = Arc::new(FixtureExecutor::new([json_fixture(response)]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();

        if expected_header.is_some() {
            client.quote(Ticker::new("AAPL").unwrap()).await.unwrap();
        } else {
            client
                .stock_price_change(Ticker::new("AAPL").unwrap())
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

fn proxy_client(executor: Arc<FixtureExecutor>, authentication: Authentication) -> Client {
    Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("stable")
        .authentication(authentication)
        .executor(executor)
        .build()
        .unwrap()
}
