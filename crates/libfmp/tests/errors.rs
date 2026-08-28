use libfmp::{
    Error,
    error::{
        ErrorCategory, MAX_SAFE_BODY_BYTES, REDACTED, Redactor, SafeBody, SecretNameError,
        SecretString, SecretUrl,
    },
    types::Ticker,
};
use serde::Deserialize;

#[test]
fn validation_and_configuration_have_stable_categories() {
    let validation = Error::from(Ticker::new(" ").unwrap_err());
    let configuration = Error::configuration("base URL must be absolute");

    assert_eq!(validation.category(), ErrorCategory::Validation);
    assert_eq!(validation.category().as_str(), "validation");
    assert_eq!(configuration.category(), ErrorCategory::Configuration);
    assert_eq!(configuration.endpoint(), None);
    assert_eq!(configuration.status_code(), None);
}

#[test]
fn status_and_decode_retain_only_safe_structured_context() {
    let mut redactor = Redactor::new();
    redactor.add_secret(&SecretString::new("very-secret"));
    let body = SafeBody::new("denied: very-secret", &redactor);
    let status = Error::status("/stable/quote-short", 401, Some(body));

    assert_eq!(status.category(), ErrorCategory::Status);
    assert_eq!(status.endpoint(), Some("/stable/quote-short"));
    assert_eq!(status.status_code(), Some(401));
    assert_eq!(status.body().unwrap().as_str(), "denied: [REDACTED]");
    assert!(!status.to_string().contains("very-secret"));

    let decode = Error::decode(
        Some("/stable/quote-short"),
        Some(200),
        status.body().cloned(),
        "invalid JSON",
    );
    assert_eq!(decode.category(), ErrorCategory::Decode);
    assert_eq!(decode.status_code(), Some(200));
}

#[test]
fn safe_body_is_redacted_and_bounded_in_utf8_bytes() {
    let secret = SecretString::new("token-value");
    let mut redactor = Redactor::new();
    redactor.add_secret(&secret);
    let body = format!(
        "https://example.test/report?apikey=abc&symbol=AAPL token-value {}",
        "界".repeat(MAX_SAFE_BODY_BYTES)
    );
    let safe = SafeBody::new(&body, &redactor);

    assert!(safe.is_truncated());
    assert!(safe.as_str().len() <= MAX_SAFE_BODY_BYTES);
    assert!(!safe.as_str().contains("abc"));
    assert!(!safe.as_str().contains("token-value"));
    assert!(safe.as_str().contains(REDACTED));
}

#[test]
fn authentication_values_and_secret_urls_never_format_in_cleartext() {
    let secret = SecretString::new("bearer-token");
    let url = SecretUrl::new(
        "https://financialmodelingprep.com/stable/financial-reports-json?apikey=key",
    );
    let mut redactor = Redactor::new();
    redactor.add_secret(&secret);
    redactor.add_secret_header_name("x-router-secret").unwrap();
    redactor.add_secret_query_name("router_token").unwrap();

    assert_eq!(secret.to_string(), REDACTED);
    assert!(!format!("{secret:?}").contains("bearer-token"));
    assert_eq!(url.to_string(), "[REDACTED URL]");
    assert!(!format!("{url:?}").contains("financial-reports"));
    assert_eq!(
        redactor.redact_header("Authorization", "Bearer x"),
        REDACTED
    );
    assert_eq!(
        redactor.redact_header("X-Router-Secret", "router-value"),
        REDACTED
    );
    assert_eq!(
        redactor.redact("/route?router_token=router-value&symbol=AAPL"),
        "/route?router_token=[REDACTED]&symbol=AAPL"
    );
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReportLinks {
    link_json: SecretUrl,
    link_xlsx: SecretUrl,
}

#[test]
fn secret_report_urls_deserialize_but_only_explicitly_expose_cleartext() {
    const JSON_URL: &str = "https://example.test/report.json?symbol=TEST&apikey=json-report-secret";
    const XLSX_URL: &str =
        "https://example.test/report.xlsx?symbol=TEST&signature=xlsx-report-secret";
    let wire = format!(r#"{{"linkJson":"{JSON_URL}","linkXlsx":"{XLSX_URL}"}}"#);

    let links: ReportLinks = serde_json::from_str(&wire).unwrap();
    assert_eq!(links.link_json.expose_secret(), JSON_URL);
    assert_eq!(links.link_xlsx.expose_secret(), XLSX_URL);
    assert_eq!(links.link_json.to_string(), "[REDACTED URL]");
    assert_eq!(links.link_xlsx.to_string(), "[REDACTED URL]");

    let debug = format!("{links:?}");
    assert!(!debug.contains("example.test"));
    assert!(!debug.contains("json-report-secret"));
    assert!(!debug.contains("xlsx-report-secret"));

    let safe_body = SafeBody::new(&wire, &Redactor::new());
    let error = Error::decode(
        Some("report-contract-test"),
        Some(200),
        Some(safe_body),
        "successful response could not be decoded",
    );
    let diagnostic = format!("{error:?} {error}");
    assert!(!diagnostic.contains("json-report-secret"));
    assert!(!diagnostic.contains("xlsx-report-secret"));
}

#[test]
fn custom_secret_names_reject_empty_and_unsafe_values() {
    let mut redactor = Redactor::new();

    assert_eq!(redactor.add_secret_header_name("x.router-secret"), Ok(()));
    assert_eq!(redactor.add_secret_query_name("router.token"), Ok(()));
    assert_eq!(
        redactor.add_secret_header_name(""),
        Err(SecretNameError::Empty)
    );
    assert_eq!(
        redactor.add_secret_query_name("api=key"),
        Err(SecretNameError::InvalidCharacter)
    );
    assert_eq!(
        redactor.add_secret_header_name("bad header"),
        Err(SecretNameError::InvalidCharacter)
    );
    assert_eq!(
        redactor.redact_header("X.Router-Secret", "hidden"),
        REDACTED
    );
    assert_eq!(
        redactor.redact("?router.token=hidden&x=visible"),
        "?router.token=[REDACTED]&x=visible"
    );
}

#[test]
fn empty_success_arrays_are_not_an_error_concept() {
    let rows: libfmp::Result<Vec<()>> = Ok(Vec::new());
    assert!(matches!(rows, Ok(values) if values.is_empty()));
}
