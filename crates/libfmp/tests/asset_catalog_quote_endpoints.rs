mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        commodities::{
            QuoteQuery as CommodityQuoteQuery, QuoteShortQuery as CommodityQuoteShortQuery,
            ShortOnlyQuery as CommodityShortOnlyQuery, commodities_list, commodity_quote,
            commodity_quote_short, commodity_quotes,
        },
        crypto::{
            QuoteQuery as CryptocurrencyQuoteQuery,
            QuoteShortQuery as CryptocurrencyQuoteShortQuery,
            ShortOnlyQuery as CryptocurrencyShortOnlyQuery, cryptocurrency_list,
            cryptocurrency_quote, cryptocurrency_quote_short, cryptocurrency_quotes,
        },
        forex::{
            QuoteQuery as ForexQuoteQuery, QuoteShortQuery as ForexQuoteShortQuery,
            ShortOnlyQuery as ForexShortOnlyQuery, forex_list, forex_quote, forex_quote_short,
            forex_quotes,
        },
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
    },
    error::ErrorCategory,
    responses::commodities::{Quote as CommodityQuote, QuoteShort as CommodityQuoteShort},
    transport::{HttpMethod, TransportResponse},
    types::Ticker,
};

use support::{FixtureExecutor, json_fixture};

const COMMODITIES_LIST: &[u8] = include_bytes!("fixtures/commodities_list.json");
const COMMODITY_QUOTE: &[u8] = include_bytes!("fixtures/commodities_quote.json");
const COMMODITY_QUOTE_SHORT: &[u8] = include_bytes!("fixtures/commodities_quote_short.json");
const COMMODITY_QUOTES: &[u8] = include_bytes!("fixtures/commodities_quotes.json");
const FOREX_LIST: &[u8] = include_bytes!("fixtures/forex_list.json");
const FOREX_QUOTE: &[u8] = include_bytes!("fixtures/forex_quote.json");
const FOREX_QUOTE_SHORT: &[u8] = include_bytes!("fixtures/forex_quote_short.json");
const FOREX_QUOTES: &[u8] = include_bytes!("fixtures/forex_quotes.json");
const CRYPTOCURRENCY_LIST: &[u8] = include_bytes!("fixtures/cryptocurrency_list.json");
const CRYPTOCURRENCY_QUOTE: &[u8] = include_bytes!("fixtures/cryptocurrency_quote.json");
const CRYPTOCURRENCY_QUOTE_SHORT: &[u8] =
    include_bytes!("fixtures/cryptocurrency_quote_short.json");
const CRYPTOCURRENCY_QUOTES: &[u8] = include_bytes!("fixtures/cryptocurrency_quotes.json");

#[test]
fn all_twelve_descriptors_use_exact_paths_shared_contracts_and_asset_metadata() {
    let commodity = Ticker::new("GCUSD").unwrap();
    let forex = Ticker::new("EURUSD").unwrap();
    let crypto = Ticker::new("BTCUSD").unwrap();

    assert_catalog(&commodities_list(), "commodities-list");
    assert_detailed(
        &commodity_quote(CommodityQuoteQuery::new(commodity.clone())),
        GeographicAvailability::UsOnly,
    );
    assert_compact(
        &commodity_quote_short(CommodityQuoteShortQuery::new(commodity)),
        GeographicAvailability::UsOnly,
    );
    assert_batch(&commodity_quotes(), "batch-commodity-quotes");

    assert_catalog(&forex_list(), "forex-list");
    assert_detailed(
        &forex_quote(ForexQuoteQuery::new(forex.clone())),
        GeographicAvailability::UsOnly,
    );
    assert_compact(
        &forex_quote_short(ForexQuoteShortQuery::new(forex)),
        GeographicAvailability::UsOnly,
    );
    assert_batch(&forex_quotes(), "batch-forex-quotes");

    assert_catalog(&cryptocurrency_list(), "cryptocurrency-list");
    assert_detailed(
        &cryptocurrency_quote(CryptocurrencyQuoteQuery::new(crypto.clone())),
        GeographicAvailability::Unspecified,
    );
    assert_compact(
        &cryptocurrency_quote_short(CryptocurrencyQuoteShortQuery::new(crypto)),
        GeographicAvailability::Unspecified,
    );
    assert_batch(&cryptocurrency_quotes(), "batch-crypto-quotes");

    assert_eq!(commodity_quotes().query(), &CommodityShortOnlyQuery::new());
    assert_eq!(forex_quotes().query(), &ForexShortOnlyQuery::new());
    assert_eq!(
        cryptocurrency_quotes().query(),
        &CryptocurrencyShortOnlyQuery::new()
    );
    assert_eq!(std::mem::size_of::<CommodityShortOnlyQuery>(), 0);
}

fn assert_catalog<R>(endpoint: &EndpointSpec<(), Vec<R>>, path: &'static str) {
    assert_common(endpoint, path, GeographicAvailability::Unspecified);
}

fn assert_detailed<Q>(
    endpoint: &EndpointSpec<Q, Vec<CommodityQuote>>,
    geography: GeographicAvailability,
) {
    assert_common(endpoint, "quote", geography);
}

fn assert_compact<Q>(
    endpoint: &EndpointSpec<Q, Vec<CommodityQuoteShort>>,
    geography: GeographicAvailability,
) {
    assert_common(endpoint, "quote-short", geography);
}

fn assert_batch<Q>(endpoint: &EndpointSpec<Q, Vec<CommodityQuoteShort>>, path: &'static str) {
    assert_common(endpoint, path, GeographicAvailability::Unspecified);
}

fn assert_common<Q, R>(
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
    assert_eq!(endpoint.metadata().realtime(), None);
}

#[tokio::test]
async fn proxy_routing_uses_all_asset_facades_custom_auth_default_headers_and_exact_fixtures() {
    let executor = Arc::new(FixtureExecutor::new(all_fixtures()));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("/gateway/stable/")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "assets")
        .executor(executor.clone())
        .build()
        .unwrap();

    let commodities = client.commodities_list().await.unwrap();
    let commodity = client
        .commodity_quote(Ticker::new("GCUSD").unwrap())
        .await
        .unwrap();
    let commodity_short = client
        .commodity_quote_short(Ticker::new("GCUSD").unwrap())
        .await
        .unwrap();
    let commodity_batch = client.commodity_quotes().await.unwrap();

    let pairs = client.forex_list().await.unwrap();
    let forex = client
        .forex_quote(Ticker::new("EURUSD").unwrap())
        .await
        .unwrap();
    let forex_short = client
        .forex_quote_short(Ticker::new("EURUSD").unwrap())
        .await
        .unwrap();
    let forex_batch = client.forex_quotes().await.unwrap();

    let cryptocurrencies = client.cryptocurrency_list().await.unwrap();
    let crypto = client
        .cryptocurrency_quote(Ticker::new("BTCUSD").unwrap())
        .await
        .unwrap();
    let crypto_short = client
        .cryptocurrency_quote_short(Ticker::new("BTCUSD").unwrap())
        .await
        .unwrap();
    let crypto_batch = client.cryptocurrency_quotes().await.unwrap();

    assert_eq!(commodities[0].exchange, None);
    assert_eq!(commodity[0].market_cap, None);
    assert_eq!(commodity_short[0].symbol.as_str(), "GCUSD");
    assert_eq!(commodity_batch[0].symbol.as_str(), "DCUSD");
    assert_eq!(pairs[0].from_currency.as_str(), "ARS");
    assert_eq!(forex[0].market_cap, None);
    assert_eq!(forex_short[0].volume, 146_872);
    assert_eq!(forex_batch[0].change, -0.00513532);
    assert_eq!(cryptocurrencies[0].total_supply, 4_788_606_639);
    assert_eq!(crypto[0].market_cap, Some(1_293_361_815_015));
    assert_eq!(crypto_short[0].volume, 32_030_003_200);
    assert_eq!(crypto_batch[0].symbol.as_str(), "00USD");

    let requests = executor.requests();
    assert_eq!(requests.len(), 12);
    assert!(
        requests
            .iter()
            .all(|request| request.method() == HttpMethod::Get)
    );
    assert!(requests.iter().all(|request| {
        request.expose_headers()["x-router-token"] == "Token proxy-secret"
            && request.expose_headers()["x-data-scope"] == "assets"
    }));
    assert_eq!(
        request_urls(&requests),
        [
            "https://proxy.example/router/gateway/stable/commodities-list",
            "https://proxy.example/router/gateway/stable/quote?symbol=GCUSD",
            "https://proxy.example/router/gateway/stable/quote-short?symbol=GCUSD",
            "https://proxy.example/router/gateway/stable/batch-commodity-quotes?short=true",
            "https://proxy.example/router/gateway/stable/forex-list",
            "https://proxy.example/router/gateway/stable/quote?symbol=EURUSD",
            "https://proxy.example/router/gateway/stable/quote-short?symbol=EURUSD",
            "https://proxy.example/router/gateway/stable/batch-forex-quotes?short=true",
            "https://proxy.example/router/gateway/stable/cryptocurrency-list",
            "https://proxy.example/router/gateway/stable/quote?symbol=BTCUSD",
            "https://proxy.example/router/gateway/stable/quote-short?symbol=BTCUSD",
            "https://proxy.example/router/gateway/stable/batch-crypto-quotes?short=true",
        ]
    );
}

#[tokio::test]
async fn direct_fmp_header_and_query_auth_work_through_asset_specific_clients() {
    for (authentication, response, call, expected_url, expected_header) in [
        (
            Authentication::fmp_header("header-secret"),
            COMMODITIES_LIST,
            DirectCall::CommoditiesList,
            "https://financialmodelingprep.com/stable/commodities-list",
            Some("header-secret"),
        ),
        (
            Authentication::fmp_query("query-secret"),
            CRYPTOCURRENCY_QUOTE,
            DirectCall::CryptocurrencyQuote,
            "https://financialmodelingprep.com/stable/quote?symbol=BTCUSD&apikey=query-secret",
            None,
        ),
    ] {
        let executor = Arc::new(FixtureExecutor::new([json_fixture(response)]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();

        match call {
            DirectCall::CommoditiesList => {
                client.commodities_list().await.unwrap();
            }
            DirectCall::CryptocurrencyQuote => {
                client
                    .cryptocurrency_quote(Ticker::new("BTCUSD").unwrap())
                    .await
                    .unwrap();
            }
        }

        let requests = executor.requests();
        assert_eq!(requests[0].expose_url().as_str(), expected_url);
        match expected_header {
            Some(value) => assert_eq!(requests[0].expose_headers()["apikey"], value),
            None => assert!(!requests[0].expose_headers().contains_key("apikey")),
        }
    }
}

#[derive(Clone, Copy)]
enum DirectCall {
    CommoditiesList,
    CryptocurrencyQuote,
}

#[tokio::test]
async fn malformed_non_array_responses_keep_catalog_quote_and_batch_endpoint_identities() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(b"{}"),
        json_fixture(b"{}"),
        json_fixture(b"{}"),
    ]));
    let client = Client::builder()
        .authentication(Authentication::fmp_header("secret"))
        .executor(executor)
        .build()
        .unwrap();

    let errors = [
        client.commodities_list().await.unwrap_err(),
        client
            .forex_quote(Ticker::new("EURUSD").unwrap())
            .await
            .unwrap_err(),
        client.cryptocurrency_quotes().await.unwrap_err(),
    ];

    for (error, id) in errors
        .iter()
        .zip(["commodities-list", "quote", "batch-crypto-quotes"])
    {
        assert_eq!(error.category(), ErrorCategory::Decode);
        assert_eq!(error.endpoint(), Some(id));
        assert_eq!(error.status_code(), Some(200));
    }
}

fn all_fixtures() -> [TransportResponse; 12] {
    [
        json_fixture(COMMODITIES_LIST),
        json_fixture(COMMODITY_QUOTE),
        json_fixture(COMMODITY_QUOTE_SHORT),
        json_fixture(COMMODITY_QUOTES),
        json_fixture(FOREX_LIST),
        json_fixture(FOREX_QUOTE),
        json_fixture(FOREX_QUOTE_SHORT),
        json_fixture(FOREX_QUOTES),
        json_fixture(CRYPTOCURRENCY_LIST),
        json_fixture(CRYPTOCURRENCY_QUOTE),
        json_fixture(CRYPTOCURRENCY_QUOTE_SHORT),
        json_fixture(CRYPTOCURRENCY_QUOTES),
    ]
}

fn request_urls(requests: &[libfmp::transport::PreparedRequest]) -> Vec<String> {
    requests
        .iter()
        .map(|request| request.expose_url().as_str().to_owned())
        .collect()
}
