use std::{
    collections::VecDeque,
    fmt,
    sync::{Arc, Mutex},
    time::Duration,
};

use http::header::{HeaderMap, HeaderValue, LOCATION};
use libfmp::{
    Client,
    client::EndpointSpec,
    config::{Authentication, RedirectPolicy},
    error::{ConfigurationErrorKind, ErrorCategory},
    transport::{
        ExecutorError, ExecutorFuture, HttpExecutor, HttpMethod, PreparedRequest, TransportResponse,
    },
};
use serde::Deserialize;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    task::JoinHandle,
};

const OK_RESPONSE: &str = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 11\r\nConnection: close\r\n\r\n{\"ok\":true}";

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct Answer {
    ok: bool,
}

fn endpoint() -> EndpointSpec<[(&'static str, &'static str); 1], Answer> {
    EndpointSpec::new(
        HttpMethod::Get,
        "transport-test",
        "/echo path",
        [("term", "AAPL & ^VIX")],
    )
}

async fn serve(responses: Vec<String>) -> (String, JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let task = tokio::spawn(async move {
        let mut requests = Vec::new();
        for response in responses {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut bytes = Vec::new();
            let mut buffer = [0_u8; 1024];
            loop {
                let count = stream.read(&mut buffer).await.unwrap();
                if count == 0 {
                    break;
                }
                bytes.extend_from_slice(&buffer[..count]);
                if bytes.windows(4).any(|window| window == b"\r\n\r\n") {
                    break;
                }
            }
            requests.push(String::from_utf8(bytes).unwrap());
            stream.write_all(response.as_bytes()).await.unwrap();
            stream.shutdown().await.unwrap();
        }
        requests
    });
    (format!("http://{address}/gateway"), task)
}

fn configured(base_url: &str, authentication: Authentication) -> Client {
    Client::builder()
        .base_url(base_url)
        .path_prefix("/router/v1/")
        .authentication(authentication)
        .build()
        .unwrap()
}

#[tokio::test]
async fn all_authentication_configurations_share_one_transport() {
    let configurations = [
        (Authentication::None, None, None),
        (
            Authentication::fmp_header("header-secret"),
            Some("apikey: header-secret"),
            None,
        ),
        (
            Authentication::fmp_query("query-secret"),
            None,
            Some("apikey=query-secret"),
        ),
        (
            Authentication::bearer("bearer-secret"),
            Some("authorization: Bearer bearer-secret"),
            None,
        ),
        (
            Authentication::custom_header("x-router-key", None, "custom-secret"),
            Some("x-router-key: custom-secret"),
            None,
        ),
        (
            Authentication::custom_header(
                "x-router-token",
                Some("Token ".to_owned()),
                "prefixed-secret",
            ),
            Some("x-router-token: Token prefixed-secret"),
            None,
        ),
        (
            Authentication::custom_query("router_token", "router-secret"),
            None,
            Some("router_token=router-secret"),
        ),
    ];

    for (authentication, expected_header, expected_query) in configurations {
        let (base_url, server) = serve(vec![OK_RESPONSE.to_owned()]).await;
        let answer = configured(&base_url, authentication)
            .execute(&endpoint())
            .await
            .unwrap();
        assert_eq!(answer, Answer { ok: true });

        let request = server.await.unwrap().pop().unwrap().to_ascii_lowercase();
        assert!(request.starts_with("get /gateway/router/v1/echo%20path?term=aapl+%26+%5evix"));
        if let Some(expected) = expected_header {
            assert!(request.contains(&expected.to_ascii_lowercase()));
        }
        if let Some(expected) = expected_query {
            assert!(request.lines().next().unwrap().contains(expected));
        }
    }
}

#[tokio::test]
async fn later_defaults_win_but_transport_fields_remain_protected() {
    let (base_url, server) = serve(vec![OK_RESPONSE.to_owned()]).await;
    let client = Client::builder()
        .base_url(base_url)
        .path_prefix("")
        .authentication(Authentication::bearer("auth-secret"))
        .default_header("x-mode", "first")
        .default_header("X-Mode", "second")
        .build()
        .unwrap();

    client.execute(&endpoint()).await.unwrap();
    let request = server.await.unwrap().pop().unwrap().to_ascii_lowercase();
    assert!(request.contains("\r\nx-mode: second\r\n"));
    assert!(!request.contains("x-mode: first"));

    let error = Client::builder()
        .authentication(Authentication::fmp_header("secret"))
        .default_header("APIKEY", "replacement")
        .build()
        .unwrap_err();
    assert_eq!(
        error.configuration_kind(),
        Some(ConfigurationErrorKind::ProtectedFieldCollision)
    );

    for result in [
        Client::builder()
            .default_header("Authorization", "manual")
            .build(),
        Client::builder()
            .authentication(Authentication::fmp_query("secret"))
            .default_header("X-API-Key", "second-channel")
            .build(),
        Client::builder()
            .default_header("Host", "other.test")
            .build(),
        Client::builder()
            .default_header("Transfer-Encoding", "chunked")
            .build(),
    ] {
        assert_eq!(
            result.unwrap_err().configuration_kind(),
            Some(ConfigurationErrorKind::ProtectedFieldCollision)
        );
    }
}

#[tokio::test]
async fn default_origin_and_complex_url_encoding_are_deterministic() {
    let executor = Arc::new(ScriptedExecutor::new(vec![Ok(json_response())]));
    let endpoint: EndpointSpec<_, Answer> = EndpointSpec::new(
        HttpMethod::Get,
        "encoding",
        "/encoded path",
        [("symbols", "A&B = + 雪")],
    );
    let client = Client::builder()
        .authentication(Authentication::fmp_header("direct-secret"))
        .default_header("x-first", "one")
        .default_header("x-second", "two")
        .executor(executor.clone())
        .build()
        .unwrap();
    client.execute(&endpoint).await.unwrap();

    let request = executor.requests().pop().unwrap();
    assert_eq!(request.expose_url().scheme(), "https");
    assert_eq!(
        request.expose_url().host_str(),
        Some("financialmodelingprep.com")
    );
    assert_eq!(
        request.expose_url().as_str(),
        "https://financialmodelingprep.com/stable/encoded%20path?symbols=A%26B+%3D+%2B+%E9%9B%AA"
    );
    assert_eq!(request.expose_headers()["x-first"], "one");
    assert_eq!(request.expose_headers()["x-second"], "two");
}

#[tokio::test]
async fn endpoint_query_cannot_replace_query_authentication() {
    let executor = Arc::new(ScriptedExecutor::new(vec![]));
    let client = Client::builder()
        .authentication(Authentication::custom_query("ProxyToken", "secret"))
        .executor(executor.clone())
        .build()
        .unwrap();
    let endpoint: EndpointSpec<_, Answer> = EndpointSpec::new(
        HttpMethod::Get,
        "collision",
        "echo",
        [("proxytoken", "replacement")],
    );
    let error = client.execute(&endpoint).await.unwrap_err();
    assert_eq!(
        error.configuration_kind(),
        Some(ConfigurationErrorKind::ProtectedFieldCollision)
    );
    assert!(executor.requests().is_empty());
}

#[test]
fn invalid_configuration_has_typed_secret_free_failures() {
    let cases = [
        (
            Client::builder().build(),
            ConfigurationErrorKind::MissingCredential,
        ),
        (
            Client::builder().base_url("relative/path").build(),
            ConfigurationErrorKind::InvalidBaseUrl,
        ),
        (
            Client::builder().base_url("ftp://example.test").build(),
            ConfigurationErrorKind::InvalidBaseUrl,
        ),
        (
            Client::builder()
                .base_url("https://user:secret@example.test")
                .build(),
            ConfigurationErrorKind::UnsafeBaseUrl,
        ),
        (
            Client::builder()
                .base_url("https://example.test?token=secret")
                .build(),
            ConfigurationErrorKind::UnsafeBaseUrl,
        ),
        (
            Client::builder().path_prefix("../escape").build(),
            ConfigurationErrorKind::InvalidPath,
        ),
        (
            Client::builder().default_header("bad name", "x").build(),
            ConfigurationErrorKind::InvalidHeaderName,
        ),
        (
            Client::builder()
                .default_header("x-ok", "bad\nvalue")
                .build(),
            ConfigurationErrorKind::InvalidHeaderValue,
        ),
        (
            Client::builder()
                .authentication(Authentication::bearer(""))
                .build(),
            ConfigurationErrorKind::EmptyCredential,
        ),
        (
            Client::builder()
                .authentication(Authentication::custom_query("bad\nname", "secret"))
                .build(),
            ConfigurationErrorKind::InvalidQueryName,
        ),
        (
            Client::builder()
                .authentication(Authentication::custom_header(
                    "Proxy-Authorization",
                    None,
                    "secret",
                ))
                .build(),
            ConfigurationErrorKind::ProtectedFieldCollision,
        ),
    ];

    for (result, expected) in cases {
        let error = result.unwrap_err();
        assert_eq!(error.category(), ErrorCategory::Configuration);
        assert_eq!(error.configuration_kind(), Some(expected));
        let diagnostic = format!("{error:?} {error}");
        assert!(!diagnostic.contains("user:secret"));
        assert!(!diagnostic.contains("token=secret"));
        assert!(!diagnostic.contains("example.test"));
    }

    let error = Client::builder()
        .authentication(Authentication::bearer("first"))
        .authentication(Authentication::fmp_query("second"))
        .build()
        .unwrap_err();
    assert_eq!(
        error.configuration_kind(),
        Some(ConfigurationErrorKind::ConflictingAuthentication)
    );
}

#[tokio::test]
async fn same_origin_redirects_reapply_header_and_query_authentication() {
    for authentication in [
        Authentication::custom_header("x-secret", None, "header-secret"),
        Authentication::custom_query("secret_query", "query-secret"),
    ] {
        let redirect = "HTTP/1.1 302 Found\r\nLocation: /redirected?kept=yes\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
        let (base_url, server) = serve(vec![redirect.to_owned(), OK_RESPONSE.to_owned()]).await;
        configured(&base_url, authentication)
            .execute(&endpoint())
            .await
            .unwrap();
        let requests = server.await.unwrap();
        assert_eq!(requests.len(), 2);
        let redirected = requests[1].to_ascii_lowercase();
        assert!(redirected.starts_with("get /redirected?kept=yes"));
        assert!(
            redirected.contains("x-secret: header-secret")
                || redirected
                    .lines()
                    .next()
                    .unwrap()
                    .contains("secret_query=query-secret")
        );
    }
}

#[tokio::test]
async fn cross_origin_redirect_is_not_executed_or_given_secrets() {
    let mut redirect_headers = HeaderMap::new();
    redirect_headers.insert(
        LOCATION,
        HeaderValue::from_static("http://other.example.test/steal"),
    );
    let executor = Arc::new(ScriptedExecutor::new(vec![Ok(TransportResponse::new(
        302,
        redirect_headers,
        "token=redirect-secret",
    ))]));
    let client = Client::builder()
        .base_url("https://example.test:443/proxy")
        .authentication(Authentication::custom_header(
            "x-secret",
            None,
            "redirect-secret",
        ))
        .executor(executor.clone())
        .build()
        .unwrap();
    let error = client.execute(&endpoint()).await.unwrap_err();

    assert_eq!(error.category(), ErrorCategory::Status);
    assert_eq!(executor.requests().len(), 1);
    assert!(!format!("{error:?} {error}").contains("redirect-secret"));
}

#[tokio::test]
async fn query_secrets_never_cross_scheme_host_or_effective_port() {
    for destination in [
        "http://example.test/steal",
        "https://other.example.test/steal",
        "https://example.test:444/steal",
    ] {
        let mut headers = HeaderMap::new();
        headers.insert(LOCATION, HeaderValue::from_str(destination).unwrap());
        let executor = Arc::new(ScriptedExecutor::new(vec![Ok(TransportResponse::new(
            302,
            headers,
            "query-secret",
        ))]));
        let client = Client::builder()
            .base_url("https://example.test:443/root")
            .authentication(Authentication::custom_query("token", "query-secret"))
            .executor(executor.clone())
            .build()
            .unwrap();
        let error = client.execute(&endpoint()).await.unwrap_err();
        assert_eq!(error.category(), ErrorCategory::Status);
        assert_eq!(executor.requests().len(), 1);
        assert!(!format!("{error:?} {error}").contains("query-secret"));
    }

    let mut redirect = HeaderMap::new();
    redirect.insert(
        LOCATION,
        HeaderValue::from_static("https://example.test/next"),
    );
    let executor = Arc::new(ScriptedExecutor::new(vec![
        Ok(TransportResponse::new(302, redirect, "")),
        Ok(json_response()),
    ]));
    Client::builder()
        .base_url("https://example.test:443/root")
        .authentication(Authentication::custom_query("token", "query-secret"))
        .executor(executor.clone())
        .build()
        .unwrap()
        .execute(&endpoint())
        .await
        .unwrap();
    assert_eq!(executor.requests().len(), 2);
}

#[tokio::test]
async fn redirect_policy_and_hop_limit_are_enforced() {
    let mut headers = HeaderMap::new();
    headers.insert(LOCATION, HeaderValue::from_static("/again"));
    let redirect = TransportResponse::new(302, headers, "");

    let executor = Arc::new(ScriptedExecutor::new(vec![Ok(redirect.clone())]));
    let error = Client::builder()
        .base_url("https://example.test")
        .redirect_policy(RedirectPolicy::None)
        .executor(executor.clone())
        .build()
        .unwrap()
        .execute(&endpoint())
        .await
        .unwrap_err();
    assert_eq!(error.category(), ErrorCategory::Status);
    assert_eq!(executor.requests().len(), 1);

    let executor = Arc::new(ScriptedExecutor::new(
        (0..11).map(|_| Ok(redirect.clone())).collect(),
    ));
    let error = Client::builder()
        .base_url("https://example.test")
        .executor(executor.clone())
        .build()
        .unwrap()
        .execute(&endpoint())
        .await
        .unwrap_err();
    assert_eq!(error.category(), ErrorCategory::Transport);
    assert_eq!(executor.requests().len(), 11);
}

#[tokio::test]
async fn status_decode_connection_and_timeout_failures_are_safe() {
    let secret = "never-print-this-token";
    let status_body = format!("token={secret}&detail=denied");
    let status_response = format!(
        "HTTP/1.1 401 Unauthorized\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{status_body}",
        status_body.len()
    );
    let (base_url, server) = serve(vec![status_response]).await;
    let error = configured(&base_url, Authentication::bearer(secret))
        .execute(&endpoint())
        .await
        .unwrap_err();
    server.await.unwrap();
    assert_eq!(error.category(), ErrorCategory::Status);
    assert_eq!(error.status_code(), Some(401));
    assert!(!format!("{error:?} {error}").contains(secret));

    let invalid_json = format!("{{\"secret\":\"{secret}\"");
    let decode_response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{invalid_json}",
        invalid_json.len()
    );
    let (base_url, server) = serve(vec![decode_response]).await;
    let error = configured(&base_url, Authentication::fmp_query(secret))
        .execute(&endpoint())
        .await
        .unwrap_err();
    server.await.unwrap();
    assert_eq!(error.category(), ErrorCategory::Decode);
    assert!(!format!("{error:?} {error}").contains(secret));

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    drop(listener);
    let error = Client::builder()
        .base_url(format!("http://{address}"))
        .path_prefix("")
        .build()
        .unwrap()
        .execute(&endpoint())
        .await
        .unwrap_err();
    assert_eq!(error.category(), ErrorCategory::Transport);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let hanging = tokio::spawn(async move {
        let (_stream, _) = listener.accept().await.unwrap();
        tokio::time::sleep(Duration::from_millis(200)).await;
    });
    let error = Client::builder()
        .base_url(format!("http://{address}"))
        .path_prefix("")
        .timeout(Duration::from_millis(20))
        .connect_timeout(Duration::from_millis(20))
        .build()
        .unwrap()
        .execute(&endpoint())
        .await
        .unwrap_err();
    assert_eq!(error.category(), ErrorCategory::Transport);
    hanging.await.unwrap();
}

#[tokio::test]
async fn debug_output_never_contains_configured_secrets_or_urls() {
    let secret = "debug-secret-value";
    let builder = Client::builder()
        .base_url(format!("https://example.test/{secret}"))
        .authentication(Authentication::custom_query("token", secret))
        .default_header("x-private", secret);
    assert!(!format!("{builder:?}").contains(secret));

    let executor = Arc::new(ScriptedExecutor::new(vec![Ok(TransportResponse::new(
        200,
        HeaderMap::new(),
        OK_RESPONSE,
    ))]));
    let client = builder.executor(executor.clone()).build().unwrap();
    assert!(!format!("{client:?}").contains(secret));

    let _ = client.execute(&endpoint()).await;
    let request = executor.requests().pop().unwrap();
    let debug = format!("{request:?}");
    assert!(!debug.contains(secret));
    assert!(!debug.contains("example.test"));
}

struct ScriptedExecutor {
    responses: Mutex<VecDeque<std::result::Result<TransportResponse, ExecutorError>>>,
    requests: Mutex<Vec<PreparedRequest>>,
}

fn json_response() -> TransportResponse {
    TransportResponse::new(200, HeaderMap::new(), br#"{"ok":true}"#.as_slice())
}

impl ScriptedExecutor {
    fn new(responses: Vec<std::result::Result<TransportResponse, ExecutorError>>) -> Self {
        Self {
            responses: Mutex::new(responses.into()),
            requests: Mutex::new(Vec::new()),
        }
    }

    fn requests(&self) -> Vec<PreparedRequest> {
        self.requests.lock().unwrap().clone()
    }
}

impl fmt::Debug for ScriptedExecutor {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ScriptedExecutor(..)")
    }
}

impl HttpExecutor for ScriptedExecutor {
    fn execute(&self, request: PreparedRequest) -> ExecutorFuture<'_> {
        Box::pin(async move {
            self.requests.lock().unwrap().push(request);
            self.responses
                .lock()
                .unwrap()
                .pop_front()
                .unwrap_or(Err(ExecutorError::new()))
        })
    }
}
