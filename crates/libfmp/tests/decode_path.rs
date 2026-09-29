mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::screener::CompanyScreenerQuery,
    error::{DecodeErrorKind, ErrorCategory},
    http::{HeaderMap, HeaderValue, header::CONTENT_TYPE},
    transport::TransportResponse,
};

use support::{FixtureExecutor, json_fixture};

const NULL_BETA: &[u8] = include_bytes!("fixtures/company_screener_null_beta_synthetic.json");
const SCREENER: &str = include_str!("fixtures/company_screener.json");
const SENTINEL: &str = "SENTINEL-beta-text";

fn json_body(body: String) -> TransportResponse {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    TransportResponse::new(200, headers, body)
}

async fn decode_error(response: TransportResponse) -> libfmp::Error {
    let executor = Arc::new(FixtureExecutor::new([response]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("stable")
        .authentication(Authentication::None)
        .executor(executor.clone())
        .build()
        .unwrap();
    let error = client
        .company_screener(CompanyScreenerQuery::new())
        .await
        .unwrap_err();
    assert_eq!(executor.requests().len(), 1);
    assert_eq!(error.category(), ErrorCategory::Decode);
    assert_eq!(error.endpoint(), Some("company-screener"));
    assert_eq!(error.message(), "successful response could not be decoded");
    error
}

#[tokio::test]
async fn null_member_names_the_row_index_and_member() {
    let error = decode_error(json_fixture(NULL_BETA)).await;

    assert_eq!(error.decode_path(), Some("[37].beta"));
    assert_eq!(error.decode_kind(), Some(DecodeErrorKind::Null));
    assert!(error.to_string().starts_with(
        "successful response could not be decoded: null value at [37].beta \
         (endpoint: company-screener): "
    ));
}

#[tokio::test]
async fn string_valued_member_never_reaches_the_error_text() {
    let body = String::from_utf8(NULL_BETA.to_vec()).unwrap().replacen(
        "\"beta\": null",
        &format!("\"beta\": \"{SENTINEL}\""),
        1,
    );
    let error = decode_error(json_body(body)).await;

    assert_eq!(error.decode_path(), Some("[37].beta"));
    assert_eq!(error.decode_kind(), Some(DecodeErrorKind::WrongType));
    assert!(error.body().unwrap().is_truncated());
    assert!(!error.to_string().contains(SENTINEL));
    assert!(!format!("{error:?}").contains(SENTINEL));
}

#[tokio::test]
async fn location_fields_omit_the_value_even_when_the_body_retains_it() {
    let body = SCREENER.replacen("1.097", &format!("\"{SENTINEL}\""), 1);
    let error = decode_error(json_body(body)).await;

    assert_eq!(error.decode_path(), Some("[0].beta"));
    assert_eq!(error.decode_kind(), Some(DecodeErrorKind::WrongType));
    assert!(error.body().unwrap().as_str().contains(SENTINEL));
    assert!(!error.message().contains(SENTINEL));
    assert!(!error.decode_path().unwrap().contains(SENTINEL));
}

#[tokio::test]
async fn missing_member_path_names_the_absent_member() {
    let body = SCREENER.replacen("\"beta\": 1.097,", "", 1);
    let error = decode_error(json_body(body)).await;

    assert_eq!(error.decode_path(), Some("[0].beta"));
    assert_eq!(error.decode_kind(), Some(DecodeErrorKind::MissingMember));
}

#[tokio::test]
async fn number_for_a_string_member_is_a_wrong_type_at_that_member() {
    let body = SCREENER.replacen("\"AAPL\"", "5", 1);
    let error = decode_error(json_body(body)).await;

    assert_eq!(error.decode_path(), Some("[0].symbol"));
    assert_eq!(error.decode_kind(), Some(DecodeErrorKind::WrongType));
}

#[tokio::test]
async fn rejected_string_value_is_an_invalid_value_without_its_text() {
    let body = SCREENER.replacen("\"AAPL\"", "\" \"", 1);
    let error = decode_error(json_body(body)).await;

    assert_eq!(error.decode_path(), Some("[0].symbol"));
    assert_eq!(error.decode_kind(), Some(DecodeErrorKind::InvalidValue));
}

#[tokio::test]
async fn root_shape_mismatch_has_a_kind_but_no_path() {
    let error = decode_error(json_body("{}".to_owned())).await;

    assert_eq!(error.decode_path(), None);
    assert_eq!(error.decode_kind(), Some(DecodeErrorKind::WrongType));
    assert!(
        error
            .to_string()
            .starts_with("successful response could not be decoded: wrong type (endpoint:")
    );
}

#[tokio::test]
async fn malformed_and_trailing_json_are_syntax_failures() {
    let malformed = decode_error(json_body("[{\"beta\": nope}]".to_owned())).await;
    let trailing = decode_error(json_body("[] []".to_owned())).await;

    assert_eq!(malformed.decode_kind(), Some(DecodeErrorKind::Syntax));
    assert_eq!(trailing.decode_kind(), Some(DecodeErrorKind::Syntax));
    assert_eq!(trailing.decode_path(), None);
}

#[tokio::test]
async fn non_json_and_empty_bodies_are_root_syntax_failures() {
    for body in ["not-json", ""] {
        let error = decode_error(json_body(body.to_owned())).await;

        assert_eq!(error.decode_kind(), Some(DecodeErrorKind::Syntax));
        assert_eq!(error.decode_path(), None);
    }
}

#[test]
fn decode_kind_exposes_stable_binding_values() {
    let kinds = [
        (DecodeErrorKind::Syntax, "syntax"),
        (DecodeErrorKind::Null, "null"),
        (DecodeErrorKind::MissingMember, "missing_member"),
        (DecodeErrorKind::WrongType, "wrong_type"),
        (DecodeErrorKind::InvalidValue, "invalid_value"),
    ];
    for (kind, value) in kinds {
        assert_eq!(kind.as_str(), value);
        assert_eq!(kind.to_string(), value);
    }
}
