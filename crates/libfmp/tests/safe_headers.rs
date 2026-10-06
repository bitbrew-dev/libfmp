use std::time::{Duration, SystemTime, UNIX_EPOCH};

use libfmp::{
    error::{
        MAX_SAFE_HEADER_VALUE_BYTES, MAX_SAFE_HEADERS, Redactor, SafeHeaders, SecretString,
        is_retained_header_name,
    },
    http::{HeaderMap, HeaderName, HeaderValue},
};

fn header_map(pairs: &[(&str, &str)]) -> HeaderMap {
    let mut headers = HeaderMap::new();
    for (name, value) in pairs {
        headers.append(
            HeaderName::from_bytes(name.as_bytes()).unwrap(),
            HeaderValue::from_str(value).unwrap(),
        );
    }
    headers
}

#[test]
fn allowlist_is_case_insensitive_and_closed() {
    for name in [
        "Retry-After",
        "X-Proxy-Error",
        "x-proxy-anything",
        "X-RateLimit-Reset",
    ] {
        assert!(is_retained_header_name(name), "{name}");
    }
    for name in [
        "set-cookie",
        "authorization",
        "apikey",
        "cookie",
        "x-proxy",
        "retry-after-ms",
    ] {
        assert!(!is_retained_header_name(name), "{name}");
    }
}

fn retry_after(value: &str, now: SystemTime) -> Option<Duration> {
    SafeHeaders::from_header_map(&header_map(&[("Retry-After", value)]), &Redactor::new())
        .retry_after_at(now)
}

#[test]
fn retry_after_accepts_every_http_date_form() {
    let now = UNIX_EPOCH + Duration::from_secs(784_111_717);
    for date in [
        "Sun, 06 Nov 1994 08:49:37 GMT",
        "Sunday, 06-Nov-94 08:49:37 GMT",
        "Sun Nov  6 08:49:37 1994",
    ] {
        assert_eq!(
            retry_after(date, now),
            Some(Duration::from_secs(60)),
            "{date}"
        );
    }
}

#[test]
fn retry_after_in_the_past_is_zero_and_garbage_is_none() {
    let now = UNIX_EPOCH + Duration::from_secs(900_000_000);
    assert_eq!(
        retry_after("Sun, 06 Nov 1994 08:49:37 GMT", now),
        Some(Duration::ZERO)
    );
    assert_eq!(retry_after(" 7 ", now), Some(Duration::from_secs(7)));
    for value in ["soon", "-5", "1.5", ""] {
        assert_eq!(retry_after(value, now), None, "{value:?}");
    }
}

#[test]
fn retained_headers_are_redacted_and_bounded() {
    let mut redactor = Redactor::new();
    redactor.add_secret(&SecretString::new("vk_live_secret".to_owned()));
    let mut pairs = vec![
        ("X-Proxy-Key-Id", "vk_live_secret".to_owned()),
        ("X-Proxy-Long", "x".repeat(MAX_SAFE_HEADER_VALUE_BYTES + 1)),
    ];
    pairs.extend((0..MAX_SAFE_HEADERS + 4).map(|index| ("X-Proxy-Count", index.to_string())));
    let pairs: Vec<_> = pairs
        .iter()
        .map(|(name, value)| (*name, value.as_str()))
        .collect();

    let headers = SafeHeaders::from_header_map(&header_map(&pairs), &redactor);

    assert_eq!(headers.len(), MAX_SAFE_HEADERS);
    assert_eq!(headers.get("x-proxy-key-id"), Some("[REDACTED]"));
    assert_eq!(headers.get("x-proxy-long"), None);
}
