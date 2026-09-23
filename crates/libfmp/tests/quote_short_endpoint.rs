mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::quote::{QuoteShortQuery, quote_short},
    transport::HttpMethod,
    types::Ticker,
};

use support::{FixtureExecutor, json_fixture};

const QUOTE_SHORT: &[u8] = include_bytes!("fixtures/quote_short.json");
const QUOTE_SHORT_EMPTY: &[u8] = include_bytes!("fixtures/quote_short_empty.json");
const QUOTE_SHORT_MULTIPLE: &[u8] = include_bytes!("fixtures/quote_short_multiple.json");
const QUOTE_SHORT_UNKNOWN: &[u8] = include_bytes!("fixtures/quote_short_unknown.json");
const QUOTE_SHORT_FRACTIONAL_VOLUME: &[u8] =
    include_bytes!("fixtures/quote_short_fractional_volume.json");

#[test]
fn descriptor_uses_the_documented_method_path_query_and_response_shape() {
    let ticker = Ticker::new("AAPL").unwrap();
    let endpoint = quote_short(QuoteShortQuery::new(ticker.clone()));

    assert_eq!(endpoint.id(), "quote-short");
    assert_eq!(endpoint.method(), HttpMethod::Get);
    assert_eq!(endpoint.relative_path(), "quote-short");
    assert_eq!(endpoint.query().symbol(), &ticker);
}

#[tokio::test]
async fn client_decodes_every_documented_field_without_selecting_a_first_row() {
    let executor = Arc::new(FixtureExecutor::new([json_fixture(QUOTE_SHORT)]));
    let client = proxy_client(executor.clone(), Authentication::None);
    let ticker = Ticker::new("AAPL").unwrap();

    let rows = client.quote_short(&ticker).await.unwrap();

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].symbol.as_str(), "AAPL");
    assert_eq!(rows[0].price, 331.85501);
    assert_eq!(rows[0].change, -6.33498);
    assert_eq!(rows[0].volume, 28_718_014.0);

    let requests = executor.requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].method(), HttpMethod::Get);
    assert_eq!(
        requests[0].expose_url().as_str(),
        "https://proxy.example/router/stable/quote-short?symbol=AAPL"
    );
}

#[tokio::test]
async fn client_preserves_empty_multiple_and_forward_compatible_arrays() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(QUOTE_SHORT_EMPTY),
        json_fixture(QUOTE_SHORT_MULTIPLE),
        json_fixture(QUOTE_SHORT_UNKNOWN),
    ]));
    let client = proxy_client(executor, Authentication::None);
    let ticker = Ticker::new("^VIX").unwrap();

    let empty = client.quote_short(&ticker).await.unwrap();
    let multiple = client.quote_short(ticker.clone()).await.unwrap();
    let future_shape = client.quote_short(ticker).await.unwrap();

    assert!(empty.is_empty());
    assert_eq!(multiple.len(), 2);
    assert_eq!(multiple[0].symbol.as_str(), "000001.SZ");
    assert_eq!(multiple[0].volume, 4_294_967_296.0);
    assert_eq!(multiple[1].symbol.as_str(), "^VIX");
    assert_eq!(future_shape.len(), 1);
    assert_eq!(future_shape[0].symbol.as_str(), "AAPL");
}

#[tokio::test]
async fn client_decodes_the_fractional_volume_observed_live_and_re_encodes_it_unchanged() {
    let executor = Arc::new(FixtureExecutor::new([json_fixture(
        QUOTE_SHORT_FRACTIONAL_VOLUME,
    )]));
    let client = proxy_client(executor, Authentication::None);

    let rows = client
        .quote_short(&Ticker::new("AAPL").unwrap())
        .await
        .unwrap();

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].price, 342.395);
    assert_eq!(rows[0].change, 3.415);
    assert_eq!(rows[0].volume, 20_201_922.827_33);
    let wire: serde_json::Value = serde_json::from_slice(QUOTE_SHORT_FRACTIONAL_VOLUME).unwrap();
    assert_eq!(serde_json::to_value(&rows).unwrap(), wire);
}

#[tokio::test]
async fn direct_fmp_header_and_query_auth_use_the_same_typed_endpoint() {
    for (authentication, expected_url, expected_header) in [
        (
            Authentication::fmp_header("header-secret"),
            "https://financialmodelingprep.com/stable/quote-short?symbol=AAPL",
            Some(("apikey", "header-secret")),
        ),
        (
            Authentication::fmp_query("query-secret"),
            "https://financialmodelingprep.com/stable/quote-short?symbol=AAPL&apikey=query-secret",
            None,
        ),
    ] {
        let executor = Arc::new(FixtureExecutor::new([json_fixture(QUOTE_SHORT)]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();

        client
            .quote_short(Ticker::new("AAPL").unwrap())
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
async fn proxy_auth_and_custom_routing_do_not_change_the_endpoint_contract() {
    let executor = Arc::new(FixtureExecutor::new([json_fixture(QUOTE_SHORT)]));
    let client = proxy_client(
        executor.clone(),
        Authentication::custom_header("x-router-token", Some("Token ".to_owned()), "proxy-secret"),
    );

    client
        .quote_short(Ticker::new("000001.SZ").unwrap())
        .await
        .unwrap();

    let requests = executor.requests();
    assert_eq!(
        requests[0].expose_url().as_str(),
        "https://proxy.example/router/stable/quote-short?symbol=000001.SZ"
    );
    assert_eq!(
        requests[0].expose_headers()["x-router-token"],
        "Token proxy-secret"
    );
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
