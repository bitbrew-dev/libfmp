mod support;

use std::sync::Arc;

use http::header::{CONTENT_DISPOSITION, CONTENT_TYPE, HeaderMap, HeaderValue};
use libfmp::{
    Client,
    config::Authentication,
    endpoints::{BinaryBody, BinaryResponse, EndpointSpec, QueryEncoder, QueryParameters},
    error::{ErrorCategory, MAX_SAFE_BODY_BYTES},
    responses::quote::QuoteShort,
    transport::HttpMethod,
    types::Ticker,
};

use support::{FixtureExecutor, fixture_response, json_fixture};

const QUOTE_SHORT: &[u8] = include_bytes!("fixtures/quote_short.json");
const QUOTE_SHORT_EMPTY: &[u8] = include_bytes!("fixtures/quote_short_empty.json");
const QUOTE_SHORT_MULTIPLE: &[u8] = include_bytes!("fixtures/quote_short_multiple.json");
const QUOTE_SHORT_UNKNOWN: &[u8] = include_bytes!("fixtures/quote_short_unknown.json");

#[derive(Debug)]
struct ExampleQuery {
    symbol: Ticker,
    limit: Option<u32>,
}

impl QueryParameters for ExampleQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.required("symbol", &self.symbol);
        encoder.optional("limit", self.limit);
    }
}

// Synthetic descriptor used only to exercise reusable required/optional query
// mechanics. The real quote-short endpoint, added in #7, documents `symbol` only.
fn contract_probe_descriptor(
    symbol: &str,
    limit: Option<u32>,
) -> EndpointSpec<ExampleQuery, Vec<QuoteShort>> {
    EndpointSpec::get(
        "endpoint-contract-test",
        "contract-probe",
        ExampleQuery {
            symbol: Ticker::new(symbol).unwrap(),
            limit,
        },
    )
}

fn mock_client(executor: Arc<FixtureExecutor>) -> Client {
    Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("stable")
        .executor(executor)
        .build()
        .unwrap()
}

#[tokio::test]
async fn get_descriptor_serializes_exact_queries_and_omits_none() {
    let executor = Arc::new(FixtureExecutor::new([json_fixture(QUOTE_SHORT)]));
    let client = mock_client(executor.clone());
    let endpoint = contract_probe_descriptor("^VIX", None);

    let rows = client.execute(&endpoint).await.unwrap();

    assert_eq!(endpoint.method(), HttpMethod::Get);
    assert_eq!(endpoint.relative_path(), "contract-probe");
    assert_eq!(rows.len(), 1);
    let requests = executor.requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].method(), HttpMethod::Get);
    assert_eq!(
        requests[0].expose_url().as_str(),
        "https://proxy.example/router/stable/contract-probe?symbol=%5EVIX"
    );
    assert!(!requests[0].expose_url().as_str().contains("limit"));
}

#[tokio::test]
async fn present_optional_queries_keep_provider_names_and_wire_order() {
    let executor = Arc::new(FixtureExecutor::new([json_fixture(QUOTE_SHORT)]));
    let client = mock_client(executor.clone());

    client
        .execute(&contract_probe_descriptor("AAPL", Some(100)))
        .await
        .unwrap();

    assert_eq!(
        executor.requests()[0].expose_url().query(),
        Some("symbol=AAPL&limit=100")
    );
}

#[tokio::test]
async fn json_contract_preserves_bare_array_shapes_and_ignores_unknown_fields() {
    let responses = [
        fixture_response(Some("application/json; charset=utf-8"), QUOTE_SHORT_EMPTY),
        fixture_response(
            Some("application/vnd.fmp.snapshot+json"),
            QUOTE_SHORT_MULTIPLE,
        ),
        json_fixture(QUOTE_SHORT_UNKNOWN),
    ];
    let client = mock_client(Arc::new(FixtureExecutor::new(responses)));

    let empty = client
        .execute(&contract_probe_descriptor("AAPL", None))
        .await
        .unwrap();
    let multiple = client
        .execute(&contract_probe_descriptor("AAPL", None))
        .await
        .unwrap();
    let future_shape = client
        .execute(&contract_probe_descriptor("AAPL", None))
        .await
        .unwrap();

    assert!(empty.is_empty());
    assert_eq!(multiple.len(), 2);
    assert_eq!(multiple[0].symbol.as_str(), "000001.SZ");
    assert_eq!(multiple[1].symbol.as_str(), "^VIX");
    assert_eq!(future_shape.len(), 1);
    assert_eq!(future_shape[0].symbol.as_str(), "AAPL");
}

#[tokio::test]
async fn successful_json_requires_a_matching_content_type() {
    for response in [
        fixture_response(Some("text/plain"), QUOTE_SHORT),
        fixture_response(None, QUOTE_SHORT),
    ] {
        let client = mock_client(Arc::new(FixtureExecutor::new([response])));
        let error = client
            .execute(&contract_probe_descriptor("AAPL", None))
            .await
            .unwrap_err();

        assert_eq!(error.category(), ErrorCategory::Decode);
        assert_eq!(error.status_code(), Some(200));
        assert_eq!(error.endpoint(), Some("endpoint-contract-test"));
    }
}

#[tokio::test]
async fn binary_contract_retains_bytes_and_safe_content_metadata() {
    const BINARY_TYPE: &str = "application/octet-stream";
    const CONTENT_TYPE_VALUE: &str = "Application/Octet-Stream; token=private-media-token";
    const DISPOSITION: &str = "attachment; filename=private-report-name.xlsx";
    const BINARY_BYTES: &[u8] = b"private-binary-payload";

    let endpoint: EndpointSpec<(), BinaryBody> = EndpointSpec::get_binary(
        "binary-contract-test",
        "future-download",
        (),
        &[BINARY_TYPE],
    );
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static(CONTENT_TYPE_VALUE));
    headers.insert(CONTENT_DISPOSITION, HeaderValue::from_static(DISPOSITION));
    let executor = Arc::new(FixtureExecutor::new([
        libfmp::transport::TransportResponse::new(200, headers, BINARY_BYTES),
    ]));

    let body = mock_client(executor).execute(&endpoint).await.unwrap();

    assert_eq!(body.as_bytes(), BINARY_BYTES);
    assert_eq!(body.content_type(), CONTENT_TYPE_VALUE);
    assert_eq!(body.content_disposition(), Some(DISPOSITION));

    let debug = format!("{body:?}");
    assert!(debug.contains("body_bytes: 22"));
    assert!(debug.contains("Application/Octet-Stream"));
    assert!(debug.contains("has_content_disposition: true"));
    assert!(!debug.contains("private-binary-payload"));
    assert!(!debug.contains("private-report-name"));
    assert!(!debug.contains("private-media-token"));

    let compatibility_name: BinaryBody = body.clone();
    let additive_name: BinaryResponse = compatibility_name;
    assert_eq!(additive_name.into_bytes(), BINARY_BYTES);

    let no_disposition = Arc::new(FixtureExecutor::new([fixture_response(
        Some(BINARY_TYPE),
        BINARY_BYTES,
    )]));
    let body = mock_client(no_disposition)
        .execute(&endpoint)
        .await
        .unwrap();
    assert_eq!(body.content_disposition(), None);
}

#[tokio::test]
async fn binary_response_equality_includes_bytes_and_exact_content_metadata() {
    const BINARY_TYPE: &str = "application/octet-stream";
    let endpoint: EndpointSpec<(), BinaryResponse> = EndpointSpec::get_binary(
        "binary-equality-test",
        "future-download",
        (),
        &[BINARY_TYPE],
    );
    let response =
        |content_type: &'static str, content_disposition: &'static str, bytes: &'static [u8]| {
            let mut headers = HeaderMap::new();
            headers.insert(CONTENT_TYPE, HeaderValue::from_static(content_type));
            headers.insert(
                CONTENT_DISPOSITION,
                HeaderValue::from_static(content_disposition),
            );
            libfmp::transport::TransportResponse::new(200, headers, bytes)
        };
    let executor = Arc::new(FixtureExecutor::new([
        response(BINARY_TYPE, "attachment; filename=one.xlsx", b"same"),
        response(BINARY_TYPE, "attachment; filename=one.xlsx", b"same"),
        response(
            "Application/Octet-Stream",
            "attachment; filename=one.xlsx",
            b"same",
        ),
        response(BINARY_TYPE, "attachment; filename=two.xlsx", b"same"),
        response(BINARY_TYPE, "attachment; filename=one.xlsx", b"different"),
    ]));
    let client = mock_client(executor);

    let baseline = client.execute(&endpoint).await.unwrap();
    let identical = client.execute(&endpoint).await.unwrap();
    let different_content_type = client.execute(&endpoint).await.unwrap();
    let different_disposition = client.execute(&endpoint).await.unwrap();
    let different_bytes = client.execute(&endpoint).await.unwrap();

    assert_eq!(baseline, identical);
    assert_ne!(baseline, different_content_type);
    assert_ne!(baseline, different_disposition);
    assert_ne!(baseline, different_bytes);
}

#[tokio::test]
async fn binary_contract_rejects_missing_and_wrong_content_types() {
    const BINARY_TYPE: &str = "application/octet-stream";
    const BINARY_BYTES: &[u8] = b"future-binary-response";
    let endpoint: EndpointSpec<(), BinaryBody> = EndpointSpec::get_binary(
        "binary-contract-test",
        "future-download",
        (),
        &[BINARY_TYPE],
    );

    for response in [
        json_fixture(BINARY_BYTES),
        fixture_response(None, BINARY_BYTES),
    ] {
        let error = mock_client(Arc::new(FixtureExecutor::new([response])))
            .execute(&endpoint)
            .await
            .unwrap_err();
        assert_eq!(error.category(), ErrorCategory::Decode);
        assert_eq!(error.endpoint(), Some("binary-contract-test"));
        assert_eq!(error.status_code(), Some(200));
    }
}

#[tokio::test]
async fn unexpected_binary_content_has_bounded_token_safe_diagnostics() {
    const AUTH_SECRET: &str = "transport-secret-token";
    const BODY_SECRET: &str = "body-secret-token";
    let endpoint: EndpointSpec<(), BinaryBody> = EndpointSpec::get_binary(
        "binary-contract-test",
        "future-download",
        (),
        &["application/octet-stream"],
    );
    let body = format!(
        "{{\"link\":\"https://example.test/report?apikey={BODY_SECRET}\",\"detail\":\"{AUTH_SECRET}{}\"}}",
        "x".repeat(MAX_SAFE_BODY_BYTES)
    );
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    let executor = Arc::new(FixtureExecutor::new([
        libfmp::transport::TransportResponse::new(200, headers, body.into_bytes()),
    ]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("stable")
        .authentication(Authentication::custom_query("router_token", AUTH_SECRET))
        .executor(executor)
        .build()
        .unwrap();

    let error = client.execute(&endpoint).await.unwrap_err();
    let safe_body = error.body().unwrap();
    let diagnostic = format!("{error:?} {error}");

    assert_eq!(error.category(), ErrorCategory::Decode);
    assert!(safe_body.is_truncated());
    assert!(safe_body.as_str().len() <= MAX_SAFE_BODY_BYTES);
    assert!(!diagnostic.contains(AUTH_SECRET));
    assert!(!diagnostic.contains(BODY_SECRET));
}
