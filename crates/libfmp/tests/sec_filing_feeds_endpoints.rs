mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
        sec_filings::{
            Latest8kSecFilingsQuery, LatestSecFilingsQuery, SecFilingsByCikQuery,
            SecFilingsByFormTypeQuery, SecFilingsBySymbolQuery, latest_8k_sec_filings,
            latest_sec_filings, sec_filings_by_cik, sec_filings_by_form_type,
            sec_filings_by_symbol,
        },
    },
    error::ErrorCategory,
    responses::sec_filings::SecFiling,
    transport::HttpMethod,
    types::{Cik, Date, FormType, Limit, Page, Ticker},
};

use support::{FixtureExecutor, json_fixture};

const LATEST_8K: &[u8] = include_bytes!("fixtures/latest_8k_sec_filings.json");
const LATEST: &[u8] = include_bytes!("fixtures/latest_sec_filings.json");
const BY_FORM: &[u8] = include_bytes!("fixtures/sec_filings_by_form_type.json");
const BY_SYMBOL: &[u8] = include_bytes!("fixtures/sec_filings_by_symbol.json");
const BY_CIK: &[u8] = include_bytes!("fixtures/sec_filings_by_cik.json");

#[test]
fn descriptors_use_exact_paths_bare_rows_and_only_documented_metadata() {
    let from = date("2024-01-01");
    let to = date("2024-03-01");
    let latest_8k = latest_8k_sec_filings(Latest8kSecFilingsQuery::new(from, to));
    let latest = latest_sec_filings(LatestSecFilingsQuery::new(from, to));
    let by_form = sec_filings_by_form_type(SecFilingsByFormTypeQuery::new(
        FormType::new("8-K").unwrap(),
        from,
        to,
    ));
    let by_symbol = sec_filings_by_symbol(SecFilingsBySymbolQuery::new(
        Ticker::new("AAPL").unwrap(),
        from,
        to,
    ));
    let by_cik = sec_filings_by_cik(SecFilingsByCikQuery::new(
        Cik::new("0000320193").unwrap(),
        from,
        to,
    ));

    let latest_bounds = EndpointBounds::new()
        .with_response_rows(1_000)
        .with_page(100)
        .with_date_range_days(90);
    assert_facts(&latest_8k, "sec-filings-8k", latest_bounds);
    assert_facts(&latest, "sec-filings-financials", latest_bounds);

    let search_bounds = EndpointBounds::new()
        .with_response_rows(1_000)
        .with_page(100);
    assert_facts(&by_form, "sec-filings-search/form-type", search_bounds);
    assert_facts(&by_symbol, "sec-filings-search/symbol", search_bounds);
    assert_facts(&by_cik, "sec-filings-search/cik", search_bounds);

    assert_response_types(&latest_8k, &latest, &by_form, &by_symbol, &by_cik);
}

fn assert_facts<Q>(
    endpoint: &EndpointSpec<Q, Vec<SecFiling>>,
    path: &'static str,
    bounds: EndpointBounds,
) {
    assert_eq!(endpoint.method(), HttpMethod::Get);
    assert_eq!(endpoint.id(), path);
    assert_eq!(endpoint.relative_path(), path);
    assert_eq!(
        endpoint.metadata().geography(),
        GeographicAvailability::UsOnly
    );
    assert_eq!(endpoint.metadata().access(), AccessRequirement::Unspecified);
    assert_eq!(endpoint.metadata().bounds(), bounds);
    assert_eq!(bounds.limit(), None);
    assert_eq!(endpoint.metadata().conditional_plan(), None);
    assert_eq!(endpoint.metadata().realtime(), None);
}

fn assert_response_types(
    _: &EndpointSpec<Latest8kSecFilingsQuery, Vec<SecFiling>>,
    _: &EndpointSpec<LatestSecFilingsQuery, Vec<SecFiling>>,
    _: &EndpointSpec<SecFilingsByFormTypeQuery, Vec<SecFiling>>,
    _: &EndpointSpec<SecFilingsBySymbolQuery, Vec<SecFiling>>,
    _: &EndpointSpec<SecFilingsByCikQuery, Vec<SecFiling>>,
) {
}

#[tokio::test]
async fn custom_proxy_preserves_paths_queries_headers_and_all_documented_fixtures() {
    let executor = Arc::new(FixtureExecutor::new(filing_fixtures()));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("gateway/stable")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "sec-filings")
        .executor(executor.clone())
        .build()
        .unwrap();
    let from = date("2024-01-01");
    let to = date("2024-03-01");

    let latest_8k = client
        .latest_8k_sec_filings(
            Latest8kSecFilingsQuery::new(from, to)
                .with_page(Page(0))
                .with_limit(Limit(u32::MAX)),
        )
        .await
        .unwrap();
    let latest = client
        .latest_sec_filings(LatestSecFilingsQuery::new(from, to))
        .await
        .unwrap();
    let by_form = client
        .sec_filings_by_form_type(
            SecFilingsByFormTypeQuery::new(FormType::new("8-K").unwrap(), from, to)
                .with_page(Page(0))
                .with_limit(Limit(0)),
        )
        .await
        .unwrap();
    let by_symbol = client
        .sec_filings_by_symbol(SecFilingsBySymbolQuery::new(
            Ticker::new("BRK.B / Class A").unwrap(),
            from,
            to,
        ))
        .await
        .unwrap();
    let by_cik = client
        .sec_filings_by_cik(
            SecFilingsByCikQuery::new(Cik::new("0000320193").unwrap(), from, to)
                .with_page(Page(u32::MAX))
                .with_limit(Limit(u32::MAX)),
        )
        .await
        .unwrap();

    assert_eq!(latest_8k[0].symbol.as_str(), "SUNE");
    assert_eq!(latest_8k[0].has_financials, None);
    assert_eq!(latest[0].symbol.as_str(), "DNN");
    assert_eq!(latest[0].has_financials, Some(true));
    assert_eq!(by_form[0].form_type.as_str(), "8-K");
    assert_eq!(by_form[0].has_financials, None);
    assert_eq!(by_symbol[0].symbol.as_str(), "AAPL");
    assert_eq!(by_symbol[0].has_financials, None);
    assert_eq!(by_cik[0].cik.as_str(), "0000320193");
    assert_eq!(by_cik[0].accepted_date.to_string(), "2024-03-01 18:36:45");

    let requests = executor.requests();
    assert!(
        requests
            .iter()
            .all(|request| request.method() == HttpMethod::Get)
    );
    assert!(requests.iter().all(|request| {
        request.expose_headers()["x-router-token"] == "Token proxy-secret"
            && request.expose_headers()["x-data-scope"] == "sec-filings"
    }));
    assert_eq!(
        request_urls(&requests),
        [
            "https://proxy.example/router/gateway/stable/sec-filings-8k?from=2024-01-01&to=2024-03-01&page=0&limit=4294967295",
            "https://proxy.example/router/gateway/stable/sec-filings-financials?from=2024-01-01&to=2024-03-01",
            "https://proxy.example/router/gateway/stable/sec-filings-search/form-type?formType=8-K&from=2024-01-01&to=2024-03-01&page=0&limit=0",
            "https://proxy.example/router/gateway/stable/sec-filings-search/symbol?symbol=BRK.B+%2F+Class+A&from=2024-01-01&to=2024-03-01",
            "https://proxy.example/router/gateway/stable/sec-filings-search/cik?cik=0000320193&from=2024-01-01&to=2024-03-01&page=4294967295&limit=4294967295",
        ]
    );
}

#[tokio::test]
async fn direct_header_and_query_auth_preserve_omission_zero_and_full_domains() {
    for (authentication, suffix, expected_header) in [
        (
            Authentication::fmp_header("header-secret"),
            "",
            Some("header-secret"),
        ),
        (
            Authentication::fmp_query("query-secret"),
            "&apikey=query-secret",
            None,
        ),
    ] {
        let executor = Arc::new(FixtureExecutor::new(filing_fixtures()));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();
        let from = date("2024-01-01");
        let to = date("2024-03-01");

        client
            .latest_8k_sec_filings(Latest8kSecFilingsQuery::new(from, to))
            .await
            .unwrap();
        client
            .latest_sec_filings(
                LatestSecFilingsQuery::new(from, to)
                    .with_page(Page(0))
                    .with_limit(Limit(0)),
            )
            .await
            .unwrap();
        client
            .sec_filings_by_form_type(SecFilingsByFormTypeQuery::new(
                FormType::new("10-Q/A").unwrap(),
                from,
                to,
            ))
            .await
            .unwrap();
        client
            .sec_filings_by_symbol(
                SecFilingsBySymbolQuery::new(Ticker::new("AAPL").unwrap(), from, to)
                    .with_page(Page(u32::MAX))
                    .with_limit(Limit(u32::MAX)),
            )
            .await
            .unwrap();
        client
            .sec_filings_by_cik(
                SecFilingsByCikQuery::new(Cik::new("0000320193").unwrap(), from, to)
                    .with_page(Page(0)),
            )
            .await
            .unwrap();

        let requests = executor.requests();
        assert_eq!(
            request_urls(&requests),
            [
                format!(
                    "https://financialmodelingprep.com/stable/sec-filings-8k?from=2024-01-01&to=2024-03-01{suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/sec-filings-financials?from=2024-01-01&to=2024-03-01&page=0&limit=0{suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/sec-filings-search/form-type?formType=10-Q%2FA&from=2024-01-01&to=2024-03-01{suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/sec-filings-search/symbol?symbol=AAPL&from=2024-01-01&to=2024-03-01&page=4294967295&limit=4294967295{suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/sec-filings-search/cik?cik=0000320193&from=2024-01-01&to=2024-03-01&page=0{suffix}"
                ),
            ]
        );
        for request in requests.iter() {
            match expected_header {
                Some(value) => assert_eq!(request.expose_headers()["apikey"], value),
                None => assert!(!request.expose_headers().contains_key("apikey")),
            }
        }
    }
}

#[tokio::test]
async fn malformed_non_array_responses_keep_each_endpoint_identity() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(b"{}"),
        json_fixture(b"{}"),
        json_fixture(b"{}"),
        json_fixture(b"{}"),
        json_fixture(b"{}"),
    ]));
    let client = Client::builder()
        .authentication(Authentication::fmp_header("secret"))
        .executor(executor)
        .build()
        .unwrap();
    let from = date("2024-01-01");
    let to = date("2024-03-01");

    let errors = [
        client
            .latest_8k_sec_filings(Latest8kSecFilingsQuery::new(from, to))
            .await
            .unwrap_err(),
        client
            .latest_sec_filings(LatestSecFilingsQuery::new(from, to))
            .await
            .unwrap_err(),
        client
            .sec_filings_by_form_type(SecFilingsByFormTypeQuery::new(
                FormType::new("8-K").unwrap(),
                from,
                to,
            ))
            .await
            .unwrap_err(),
        client
            .sec_filings_by_symbol(SecFilingsBySymbolQuery::new(
                Ticker::new("AAPL").unwrap(),
                from,
                to,
            ))
            .await
            .unwrap_err(),
        client
            .sec_filings_by_cik(SecFilingsByCikQuery::new(
                Cik::new("0000320193").unwrap(),
                from,
                to,
            ))
            .await
            .unwrap_err(),
    ];

    for (error, id) in errors.iter().zip([
        "sec-filings-8k",
        "sec-filings-financials",
        "sec-filings-search/form-type",
        "sec-filings-search/symbol",
        "sec-filings-search/cik",
    ]) {
        assert_eq!(error.category(), ErrorCategory::Decode);
        assert_eq!(error.endpoint(), Some(id));
        assert_eq!(error.status_code(), Some(200));
    }
}

fn filing_fixtures() -> [libfmp::transport::TransportResponse; 5] {
    [
        json_fixture(LATEST_8K),
        json_fixture(LATEST),
        json_fixture(BY_FORM),
        json_fixture(BY_SYMBOL),
        json_fixture(BY_CIK),
    ]
}

fn request_urls(requests: &[libfmp::transport::PreparedRequest]) -> Vec<String> {
    requests
        .iter()
        .map(|request| request.expose_url().as_str().to_owned())
        .collect()
}

fn date(value: &str) -> Date {
    Date::parse(value).unwrap()
}
