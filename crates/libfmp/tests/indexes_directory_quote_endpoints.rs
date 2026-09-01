mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        indexes::{
            QuoteQuery, QuoteShortQuery, ShortOnlyQuery, index_list, index_quote,
            index_quote_short, index_quotes,
        },
        metadata::{
            AccessRequirement, DelayScope, EndpointBounds, GeographicAvailability, MarketDataDelay,
            RealtimeAccess, UserDeclarationRequirement,
        },
    },
    responses::{
        indexes::IndexListing,
        quote::{Quote, QuoteShort},
    },
    transport::HttpMethod,
    types::Ticker,
};

use support::{FixtureExecutor, json_fixture};

const LIST: &[u8] = include_bytes!("fixtures/indexes_list.json");
const QUOTE: &[u8] = include_bytes!("fixtures/indexes_quote.json");
const QUOTE_SHORT: &[u8] = include_bytes!("fixtures/indexes_quote_short.json");
const QUOTES: &[u8] = include_bytes!("fixtures/indexes_quotes.json");

#[test]
fn descriptors_reuse_exact_quote_contracts_and_preserve_documented_metadata() {
    let symbol = Ticker::new("^VIX").unwrap();
    let detailed = index_quote(QuoteQuery::new(symbol.clone()));
    let compact = index_quote_short(QuoteShortQuery::new(symbol.clone()));
    let batch = index_quotes();
    let directory = index_list();

    assert_quote_contract(&detailed);
    assert_quote_short_contract(&compact);
    assert_batch_contract(&batch);
    assert_directory_contract(&directory);

    assert_eq!(detailed.query().symbol().as_str(), "^VIX");
    assert_eq!(compact.query().symbol().as_str(), "^VIX");
    assert_eq!(batch.query(), &ShortOnlyQuery::new());
    assert_eq!(std::mem::size_of::<ShortOnlyQuery>(), 0);

    let delayed_realtime = Some(RealtimeAccess::new(
        Some(MarketDataDelay::new(15, DelayScope::Nasdaq)),
        Some(UserDeclarationRequirement::RequiredForRealtime),
    ));
    assert_eq!(detailed.metadata().realtime(), delayed_realtime);
    assert_eq!(compact.metadata().realtime(), delayed_realtime);
    assert_eq!(batch.metadata().realtime(), None);
    assert_eq!(directory.metadata().realtime(), None);
}

fn assert_quote_contract(endpoint: &EndpointSpec<QuoteQuery, Vec<Quote>>) {
    assert_endpoint_facts(endpoint, "quote", GeographicAvailability::Worldwide);
}

fn assert_quote_short_contract(endpoint: &EndpointSpec<QuoteShortQuery, Vec<QuoteShort>>) {
    assert_endpoint_facts(endpoint, "quote-short", GeographicAvailability::Worldwide);
}

fn assert_batch_contract(endpoint: &EndpointSpec<ShortOnlyQuery, Vec<QuoteShort>>) {
    assert_endpoint_facts(
        endpoint,
        "batch-index-quotes",
        GeographicAvailability::Unspecified,
    );
}

fn assert_directory_contract(endpoint: &EndpointSpec<(), Vec<IndexListing>>) {
    assert_endpoint_facts(endpoint, "index-list", GeographicAvailability::Worldwide);
}

fn assert_endpoint_facts<Q, R>(
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
}

#[tokio::test]
async fn proxy_client_uses_exact_index_paths_caret_symbols_and_closed_batch_query() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(LIST),
        json_fixture(QUOTE),
        json_fixture(QUOTE_SHORT),
        json_fixture(QUOTES),
    ]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("stable")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .executor(executor.clone())
        .build()
        .unwrap();
    let symbol = Ticker::new("^VIX").unwrap();

    let listings = client.index_list().await.unwrap();
    let detailed = client.index_quote(&symbol).await.unwrap();
    let compact = client.index_quote_short(&symbol).await.unwrap();
    let batch = client.index_quotes().await.unwrap();

    assert_eq!(listings[0].symbol.as_str(), "^TTIN");
    assert_eq!(detailed[0].symbol.as_str(), "^VIX");
    assert_eq!(compact[0].symbol.as_str(), "^VIX");
    assert_eq!(batch[0].symbol.as_str(), "^SPROME10");

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
    assert_eq!(
        requests
            .iter()
            .map(|request| request.expose_url().as_str())
            .collect::<Vec<_>>(),
        [
            "https://proxy.example/router/stable/index-list",
            "https://proxy.example/router/stable/quote?symbol=%5EVIX",
            "https://proxy.example/router/stable/quote-short?symbol=%5EVIX",
            "https://proxy.example/router/stable/batch-index-quotes?short=true",
        ]
    );
}
