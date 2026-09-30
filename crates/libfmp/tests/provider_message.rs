mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::economics::EconomicIndicatorsQuery,
    error::{DecodeErrorKind, ErrorCategory},
    http::{HeaderMap, HeaderValue, header::CONTENT_TYPE},
    query::EconomicIndicator,
    transport::TransportResponse,
};

use support::{FixtureExecutor, json_fixture};

const INDICATORS: &[u8] = include_bytes!("fixtures/economic_indicators.json");
const ERROR_MESSAGE: &str = "{\n  \"Error Message\": \"No Data for this symbol or invalid API call. \
     Please retry or visit our documentation at https://financialmodelingprep.com/developer/docs.\"\n}";

fn json_body(body: &str) -> TransportResponse {
    let mut headers = HeaderMap::new();
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/json; charset=utf-8"),
    );
    TransportResponse::new(200, headers, body.to_owned())
}

async fn economic_indicators(
    response: TransportResponse,
) -> libfmp::Result<Vec<libfmp::responses::economics::EconomicIndicatorObservation>> {
    let executor = Arc::new(FixtureExecutor::new([response]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("stable")
        .authentication(Authentication::None)
        .executor(executor.clone())
        .build()
        .unwrap();
    let result = client
        .economic_indicators(EconomicIndicatorsQuery::new(EconomicIndicator::Gdp))
        .await;
    assert_eq!(executor.requests().len(), 1);
    result
}

fn assert_provider_message(error: &libfmp::Error, body: &str) {
    assert_eq!(error.category(), ErrorCategory::Status);
    assert_eq!(error.status_code(), Some(200));
    assert_eq!(error.endpoint(), Some("economic-indicators"));
    assert_eq!(
        error.message(),
        "provider returned an error message with HTTP status 200"
    );
    assert_eq!(error.decode_kind(), None);
    assert_eq!(error.decode_path(), None);
    assert_eq!(error.body().unwrap().as_str(), body);
}

#[tokio::test]
async fn plain_text_success_body_is_a_provider_message() {
    let error = economic_indicators(json_body("Invalid name"))
        .await
        .unwrap_err();

    assert_provider_message(&error, "Invalid name");
    assert_eq!(
        error.to_string(),
        "provider returned an error message with HTTP status 200 \
         (endpoint: economic-indicators): Invalid name"
    );
}

#[tokio::test]
async fn error_message_object_is_a_provider_message() {
    let error = economic_indicators(json_body(ERROR_MESSAGE))
        .await
        .unwrap_err();

    assert_provider_message(&error, ERROR_MESSAGE);
}

#[tokio::test]
async fn documented_rows_still_decode() {
    let rows = economic_indicators(json_fixture(INDICATORS)).await.unwrap();

    assert!(!rows.is_empty());
}

#[tokio::test]
async fn other_malformed_bodies_stay_decode_errors() {
    for body in [
        "",
        "null",
        "42",
        "{\"Error Message\": \"x\", \"date\": \"2024-01-01\"}",
        "[\"Invalid name\"]",
        "Invalid\u{7}name",
    ] {
        let error = economic_indicators(json_body(body)).await.unwrap_err();

        assert_eq!(error.category(), ErrorCategory::Decode, "{body:?}");
        assert!(error.decode_kind().is_some(), "{body:?}");
    }
}

#[tokio::test]
async fn long_plain_text_body_stays_a_syntax_error() {
    let body = "x ".repeat(200);
    let error = economic_indicators(json_body(&body)).await.unwrap_err();

    assert_eq!(error.category(), ErrorCategory::Decode);
    assert_eq!(error.decode_kind(), Some(DecodeErrorKind::Syntax));
}
